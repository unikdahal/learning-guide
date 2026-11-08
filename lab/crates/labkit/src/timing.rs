//! Measuring time honestly: wall-clock and CPU time, repeated runs, and summary statistics.
//! Module 00, "Measuring things honestly", explains why each piece exists.

use std::fmt;
use std::time::{Duration, Instant};

/// CPU time consumed by this process so far, summed over all of its threads.
///
/// Uses `CLOCK_PROCESS_CPUTIME_ID`, available on Linux and macOS.
pub fn process_cpu_time() -> Duration {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: `ts` is a valid, writable timespec and the clock id is a constant libc provides.
    let rc = unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts) };
    assert_eq!(rc, 0, "clock_gettime(CLOCK_PROCESS_CPUTIME_ID) failed");
    Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
}

/// Wall-clock and CPU time of one run.
#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub wall: Duration,
    pub cpu: Duration,
}

impl Sample {
    /// CPU time divided by wall time. About 1.0 for single-threaded CPU-bound work, near the
    /// thread count for well-parallelized work, and well below 1.0 when the work waits on I/O.
    /// Returns 0 when wall time is below clock resolution; no ratio can be inferred.
    pub fn parallelism(&self) -> f64 {
        if self.wall.is_zero() {
            0.0
        } else {
            self.cpu.as_secs_f64() / self.wall.as_secs_f64()
        }
    }
}

/// Runs `f` once and records its wall and CPU time.
pub fn measure<R>(f: impl FnOnce() -> R) -> (R, Sample) {
    let cpu0 = process_cpu_time();
    let t0 = Instant::now();
    let out = f();
    let wall = t0.elapsed();
    let cpu = process_cpu_time() - cpu0;
    (out, Sample { wall, cpu })
}

/// Summary statistics over repeated wall-clock samples.
#[derive(Clone, Debug)]
pub struct Summary {
    pub runs: usize,
    pub min: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub max: Duration,
    pub mean: Duration,
    /// Standard deviation divided by the mean: the noise level as a fraction.
    pub cv: f64,
}

impl Summary {
    /// # Panics
    /// Panics when samples is empty. Zero-duration samples have zero reported variation.
    pub fn of(samples: &[Duration]) -> Summary {
        assert!(!samples.is_empty(), "no samples");
        let mut s = samples.to_vec();
        s.sort();
        let n = s.len();
        let secs: Vec<f64> = s.iter().map(Duration::as_secs_f64).collect();
        let mean = secs.iter().sum::<f64>() / n as f64;
        let var = secs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n.max(2) - 1) as f64;
        // Nearest-rank percentile.
        let rank = |p: f64| s[((p * n as f64).ceil() as usize).clamp(1, n) - 1];
        Summary {
            runs: n,
            min: s[0],
            median: if n % 2 == 1 {
                s[n / 2]
            } else {
                s[n / 2 - 1] + (s[n / 2] - s[n / 2 - 1]) / 2
            },
            p95: rank(0.95),
            max: s[n - 1],
            mean: Duration::from_secs_f64(mean),
            cv: if mean == 0.0 { 0.0 } else { var.sqrt() / mean },
        }
    }
}

impl fmt::Display for Summary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "runs {:>3}  min {:>9.3?}  median {:>9.3?}  p95 {:>9.3?}  max {:>9.3?}  noise ±{:.1}%",
            self.runs,
            self.min,
            self.median,
            self.p95,
            self.max,
            self.cv * 100.0
        )
    }
}

/// Runs `f` `warmup` times without recording, then `runs` times, returning every wall sample.
pub fn repeat<R>(warmup: usize, runs: usize, mut f: impl FnMut() -> R) -> Vec<Duration> {
    for _ in 0..warmup {
        std::hint::black_box(f());
    }
    let samples = (0..runs)
        .map(|_| {
            let t0 = Instant::now();
            std::hint::black_box(f());
            t0.elapsed()
        })
        .collect::<Vec<_>>();
    record_samples(&samples);
    samples
}

/// Like [`repeat`], for async work such as running a query.
pub async fn repeat_async<R, F, Fut>(warmup: usize, runs: usize, mut f: F) -> Vec<Duration>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = R>,
{
    for _ in 0..warmup {
        std::hint::black_box(f().await);
    }
    let mut out = Vec::with_capacity(runs);
    for _ in 0..runs {
        let t0 = Instant::now();
        std::hint::black_box(f().await);
        out.push(t0.elapsed());
    }
    record_samples(&out);
    out
}

/// Exports raw samples only when explicitly requested by the benchmark recorder.
/// Called after timing, never within the measured operation.
pub fn record_samples(samples: &[Duration]) {
    use std::io::Write;
    if let Some(path) = std::env::var_os("QE_BENCH_SAMPLES") {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("open raw benchmark samples");
        let values = samples
            .iter()
            .map(|s| s.as_nanos().to_string())
            .collect::<Vec<_>>()
            .join(",");
        writeln!(file, "{{\"samples_ns\":[{values}]}}").expect("write raw benchmark samples");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degenerate_samples_are_finite() {
        let zero = Summary::of(&[Duration::ZERO; 2]);
        assert_eq!(zero.cv, 0.0);
        assert_eq!(zero.median, Duration::ZERO);
        assert_eq!(
            Sample {
                wall: Duration::ZERO,
                cpu: Duration::ZERO
            }
            .parallelism(),
            0.0
        );
        let even = Summary::of(&[Duration::from_nanos(2), Duration::from_nanos(4)]);
        assert_eq!(even.median, Duration::from_nanos(3));
        let same = Summary::of(&[Duration::from_millis(4); 3]);
        assert_eq!(same.cv, 0.0);
    }

    #[test]
    #[should_panic(expected = "no samples")]
    fn empty_samples_are_rejected() {
        Summary::of(&[]);
    }

    #[test]
    fn summary_basics() {
        let ms = |v: u64| Duration::from_millis(v);
        let s = Summary::of(&[ms(5), ms(1), ms(3), ms(2), ms(4)]);
        assert_eq!(s.min, ms(1));
        assert_eq!(s.median, ms(3));
        assert_eq!(s.max, ms(5));
        assert_eq!(s.p95, ms(5));
    }

    #[test]
    fn cpu_time_advances_with_work() {
        let (_, sample) = measure(|| (0..20_000_000u64).fold(0u64, |a, b| a.wrapping_add(b * b)));
        assert!(sample.cpu > Duration::ZERO);
    }
}
