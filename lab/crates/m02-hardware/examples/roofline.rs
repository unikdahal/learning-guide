//! Article: "The roofline model for query operators".
//! Measures this core's two ceilings, peak floating-point throughput and memory bandwidth, then
//! sweeps arithmetic intensity with a polynomial kernel (out[i] = p(in[i]) of degree K, 2K flops
//! per 8 bytes moved) on data in cache and in main memory, to place real points on a roofline.
//!
//!     cargo run --release -p m02-hardware --example roofline

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use labkit::timing::{Summary, repeat};
use m02_hardware::{keep, sum_sequential};

/// Peak: 16 independent 512-bit FMA chains in registers, 16 f32 lanes each, no memory traffic.
///
/// # Safety
/// The CPU must support AVX-512F.
#[target_feature(enable = "avx512f")]
unsafe fn fma_peak(iters: usize) -> f32 {
    // Opaque inputs: with literal constants, 1.0 is a fixed point of x * a + b, and the
    // compiler proved the loop never changes it and deleted it.
    let a = _mm512_set1_ps(keep(0.999_9));
    let b = _mm512_set1_ps(keep(0.000_1));
    let mut acc = [_mm512_set1_ps(keep(1.5)); 16];
    for _ in 0..iters {
        for x in acc.iter_mut() {
            *x = _mm512_fmadd_ps(*x, a, b);
        }
    }
    let mut s = _mm512_setzero_ps();
    for x in acc {
        s = _mm512_add_ps(s, x);
    }
    _mm512_reduce_add_ps(s)
}

/// out[i] = p(in[i]), a degree-K polynomial by Horner's rule: K fused multiply-adds, 2K flops,
/// per element, for 4 bytes read and 4 written.
///
/// # Safety
/// The CPU must support AVX-512F and FMA.
#[target_feature(enable = "avx512f,fma")]
unsafe fn poly<const K: usize>(inp: &[f32], out: &mut [f32]) {
    let (a, b) = (keep(0.999_9f32), keep(0.000_1f32));
    for (o, &x) in out.iter_mut().zip(inp) {
        let mut acc = x;
        for _ in 0..K {
            acc = acc.mul_add(a, b);
        }
        *o = acc;
    }
}

fn main() {
    assert!(is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("fma"));

    println!("== Ceiling 1: peak floating-point throughput, one core ==\n");
    let iters = 50_000_000;
    // SAFETY: AVX-512F detected above.
    let s = Summary::of(&repeat(1, 5, || unsafe { fma_peak(keep(iters)) }));
    let flops = (iters * 16 * 16 * 2) as f64;
    let peak = flops / s.median.as_secs_f64() / 1e9;
    println!("AVX-512 FMA, 16 chains: {peak:.1} GFLOP/s (f32)\n");

    println!("== Ceiling 2: memory bandwidth, one core ==\n");
    let words = (1usize << 30) / 8;
    let buf: Vec<u64> = (0..words as u64).collect();
    let s = Summary::of(&repeat(1, 5, || sum_sequential(keep(&buf))));
    let bw = (words * 8) as f64 / s.median.as_secs_f64() / 1e9;
    println!("sequential read, 1 GiB: {bw:.1} GB/s");
    drop(buf);
    let small: Vec<u64> = (0..(256 << 10) / 8).collect();
    let s = Summary::of(&repeat(1, 7, || {
        let mut t = 0u64;
        for _ in 0..4000 {
            t = t.wrapping_add(sum_sequential(keep(&small)));
        }
        t
    }));
    let bw_l2 = (small.len() * 8 * 4000) as f64 / s.median.as_secs_f64() / 1e9;
    println!("sequential read, 256 KiB (L2): {bw_l2:.1} GB/s\n");

    println!("== The sweep: out[i] = p(in[i]), degree K ==\n");
    println!(
        "{:>4}  {:>10}  {:>16}  {:>16}",
        "K", "flops/byte", "in L2: GFLOP/s", "in DRAM: GFLOP/s"
    );
    println!("(DRAM: 256 MiB in, 256 MiB out)\n");
    let mut csv = Vec::new();
    for (n, reps, label) in [
        (16 * 1024usize, 4000usize, "l2"),
        (64 * 1024 * 1024usize, 1usize, "dram"),
    ] {
        let inp: Vec<f32> = (0..n).map(|i| (i % 100) as f32 * 0.01).collect();
        let mut out = vec![0f32; n];
        macro_rules! k {
            ($k:literal) => {{
                let s = Summary::of(&repeat(1, 5, || {
                    for _ in 0..reps {
                        // SAFETY: features detected above.
                        unsafe { poly::<$k>(keep(&inp), &mut out) };
                    }
                    keep(out[n / 2])
                }));
                let gf = (2 * $k * n * reps) as f64 / s.median.as_secs_f64() / 1e9;
                csv.push(format!("{label},{},{:.3},{gf:.2}", $k, $k as f64 / 4.0));
            }};
        }
        // K = 0 is a plain copy: no arithmetic, only memory traffic.
        let s = Summary::of(&repeat(1, 5, || {
            for _ in 0..reps {
                // SAFETY: features detected above.
                unsafe { poly::<0>(keep(&inp), &mut out) };
            }
            keep(out[n / 2])
        }));
        let gbs = (8 * n * reps) as f64 / s.median.as_secs_f64() / 1e9;
        println!("{label}: copy (K = 0) moves {gbs:.1} GB/s (read + write)");
        csv.push(format!("{label}-copy,{gbs:.2}"));
        k!(1);
        k!(2);
        k!(4);
        k!(8);
        k!(16);
        k!(32);
        k!(64);
        k!(128);
    }
    for k in [1, 2, 4, 8, 16, 32, 64, 128] {
        let get = |l: &str| {
            csv.iter()
                .find(|r| r.starts_with(&format!("{l},{k},")))
                .map(|r| r.rsplit(',').next().unwrap().to_string())
                .unwrap()
        };
        println!(
            "{k:>4}  {:>10.2}  {:>16}  {:>16}",
            k as f64 / 4.0,
            get("l2"),
            get("dram")
        );
    }
    println!(
        "\nroofline csv: peak={peak:.1};bw={bw:.2};bw_l2={bw_l2:.1};{}",
        csv.join(";")
    );
}
