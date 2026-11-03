//! Article: "Multicore, threads and false sharing".
//! How a compute-bound loop scales from one core to two, what happens when two cores write to
//! the same cache line (even to different variables in it), and per-thread versus shared state.
//!
//!     cargo run --release -p m02-hardware --example multicore

use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Instant;

use labkit::timing::{Summary, repeat};
use m02_hardware::keep;

/// Independent multiply-add chains: pure computation, no shared memory.
fn work(n: usize, seed: u64) -> u64 {
    let (a, c) = (keep(6364136223846793005u64), keep(1442695040888963407u64));
    let mut x = [seed, seed + 1, seed + 2, seed + 3];
    for _ in 0..n / 4 {
        for v in x.iter_mut() {
            *v = v.wrapping_mul(a).wrapping_add(c);
        }
    }
    x.iter().fold(0, |s, &v| s ^ v)
}

/// 256 bytes of counters, 64-byte aligned, so we control exactly which line each one is on.
#[repr(C, align(64))]
struct Counters([u64; 32]);

/// Two threads each increment their own counter `n` times with a volatile read and write, so
/// every increment really goes to memory. The counters are `gap` bytes apart.
fn two_counters(gap: usize, n: usize) -> f64 {
    let mut c = Counters([0; 32]);
    let base = c.0.as_mut_ptr() as usize;
    let (pa, pb) = (base, base + gap);
    let t0 = Instant::now();
    thread::scope(|s| {
        for p in [pa, pb] {
            s.spawn(move || {
                let p = p as *mut u64;
                for _ in 0..n {
                    // SAFETY: each thread owns a distinct u64 inside `c`, which outlives the scope.
                    unsafe { std::ptr::write_volatile(p, std::ptr::read_volatile(p) + 1) };
                }
            });
        }
    });
    let ns = t0.elapsed().as_secs_f64() * 1e9 / n as f64;
    assert_eq!(c.0[0], n as u64);
    ns
}

/// The same, with atomic read-modify-writes (`fetch_add`). An atomic RMW needs the cache line in
/// exclusive state in this core's cache, so two cores updating the same line take turns owning it.
#[repr(C, align(64))]
struct AtomicCounters([AtomicU64; 32]);

fn two_atomic_counters(gap: usize, n: usize) -> f64 {
    let c = AtomicCounters(std::array::from_fn(|_| AtomicU64::new(0)));
    let (ia, ib) = (0, gap / 8);
    let t0 = Instant::now();
    thread::scope(|s| {
        for i in [ia, ib] {
            let c = &c;
            s.spawn(move || {
                for _ in 0..n {
                    c.0[i].fetch_add(1, Ordering::Relaxed);
                }
            });
        }
    });
    let ns = t0.elapsed().as_secs_f64() * 1e9 / n as f64;
    assert_eq!(c.0[ia].load(Ordering::Relaxed), n as u64);
    ns
}

fn one_counter(n: usize) -> f64 {
    let mut c = Counters([0; 32]);
    let p = c.0.as_mut_ptr();
    let t0 = Instant::now();
    for _ in 0..n {
        // SAFETY: p points into `c`.
        unsafe { std::ptr::write_volatile(p, std::ptr::read_volatile(p) + 1) };
    }
    let ns = t0.elapsed().as_secs_f64() * 1e9 / n as f64;
    keep(c.0[0]);
    ns
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn main() {
    let cores = thread::available_parallelism().map_or(1, |n| n.get());
    println!("available parallelism: {cores}\n");

    println!("== A compute-bound loop on 1 and 2 threads ==\n");
    let n = 400_000_000;
    let one = Summary::of(&repeat(1, 5, || work(n, 1)));
    let two = Summary::of(&repeat(1, 5, || {
        thread::scope(|s| {
            let h = s.spawn(|| work(n / 2, 1));
            work(n / 2, 2) ^ h.join().unwrap()
        })
    }));
    println!("1 thread : {:>8.1} ms", one.median.as_secs_f64() * 1e3);
    println!(
        "2 threads: {:>8.1} ms   speedup {:.2}x\n",
        two.median.as_secs_f64() * 1e3,
        one.median.as_secs_f64() / two.median.as_secs_f64()
    );

    println!("== Two threads, each incrementing its own counter 100M times ==\n");
    let n = 100_000_000;
    println!(
        "one thread alone        {:>6.2} ns per increment",
        median((0..5).map(|_| one_counter(n)).collect())
    );
    let mut csv = Vec::new();
    for (label, gap) in [
        ("counters 8 bytes apart ", 8usize),
        ("counters 32 bytes apart", 32),
        ("counters 64 bytes apart", 64),
        ("counters 128 bytes apart", 128),
    ] {
        let ns = median((0..5).map(|_| two_counters(gap, n)).collect());
        let same_line = gap < 64;
        println!(
            "{label:<24} {ns:>6.2} ns per increment   ({})",
            if same_line {
                "same cache line"
            } else {
                "different lines"
            }
        );
        csv.push(format!("{gap},{ns:.3}"));
    }
    println!("\nfalse sharing csv: {}\n", csv.join(";"));

    println!("== The same, with atomic fetch_add on each thread's own counter, 50M each ==\n");
    let n = 50_000_000;
    let mut csv = Vec::new();
    for (label, gap) in [
        ("counters 8 bytes apart ", 8usize),
        ("counters 32 bytes apart", 32),
        ("counters 64 bytes apart", 64),
        ("counters 128 bytes apart", 128),
    ] {
        let ns = median((0..5).map(|_| two_atomic_counters(gap, n)).collect());
        let same_line = gap < 64;
        println!(
            "{label:<24} {ns:>6.2} ns per increment   ({})",
            if same_line {
                "same cache line"
            } else {
                "different lines"
            }
        );
        csv.push(format!("{gap},{ns:.3}"));
    }
    println!("\natomic false sharing csv: {}\n", csv.join(";"));

    println!("== Counting 200M events on 2 threads: one shared atomic vs per-thread counts ==\n");
    let n = 200_000_000;
    let shared = AtomicU64::new(0);
    let t = Summary::of(&repeat(1, 5, || {
        shared.store(0, Ordering::Relaxed);
        thread::scope(|s| {
            for _ in 0..2 {
                s.spawn(|| {
                    for _ in 0..n / 2 {
                        shared.fetch_add(1, Ordering::Relaxed);
                    }
                });
            }
        });
        shared.load(Ordering::Relaxed)
    }));
    assert_eq!(shared.load(Ordering::Relaxed), n as u64);
    println!(
        "shared AtomicU64::fetch_add   {:>8.1} ms  ({:.2} ns per event)",
        t.median.as_secs_f64() * 1e3,
        t.median.as_secs_f64() * 1e9 / n as f64
    );
    let t = Summary::of(&repeat(1, 5, || {
        thread::scope(|s| {
            let hs: Vec<_> = (0..2)
                .map(|_| {
                    s.spawn(|| {
                        let mut local = 0u64;
                        for _ in 0..n / 2 {
                            local = keep(local + 1);
                        }
                        local
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).sum::<u64>()
        })
    }));
    println!(
        "per-thread counters, summed   {:>8.1} ms  ({:.2} ns per event)",
        t.median.as_secs_f64() * 1e3,
        t.median.as_secs_f64() * 1e9 / n as f64
    );
}
