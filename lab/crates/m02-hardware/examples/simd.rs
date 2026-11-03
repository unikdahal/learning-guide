//! Articles: "SIMD: one instruction, many values" and "SIMD in Rust".
//! Counts the u32 values below a threshold five ways: forced scalar, auto-vectorized for the
//! default x86-64 target (SSE2), the same Rust compiled for AVX2 and for AVX-512 with
//! `#[target_feature]`, and hand-written AVX2 intrinsics. Then a float sum, which the compiler
//! may only vectorize if we give it permission to reorder the additions.
//!
//!     cargo run --release -p m02-hardware --example simd
//!
//! The functions are `#[no_mangle]` so their machine code is easy to find:
//!     objdump -d --no-show-raw-insn target/release/examples/simd | less

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use labkit::timing::{Summary, repeat};
use m02_hardware::{keep, percent_data};

/// One value at a time. `read_volatile` stops the compiler from vectorizing the loop, giving a
/// true scalar baseline: one load, one compare, one add per value.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn count_scalar(data: &[u32], t: u32) -> u64 {
    let mut n = 0u64;
    for i in 0..data.len() {
        // SAFETY: i < data.len().
        let x = unsafe { std::ptr::read_volatile(data.as_ptr().add(i)) };
        n += (x < t) as u64;
    }
    n
}

/// Plain Rust. Built for the default x86-64 target, LLVM may only use SSE2: 128-bit registers,
/// four u32 lanes.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn count_auto(data: &[u32], t: u32) -> u64 {
    data.iter().map(|&x| (x < t) as u32).sum::<u32>() as u64
}

/// The same source, compiled for AVX2: 256-bit registers, eight lanes. Only safe to call on a
/// CPU that has AVX2, which `main` checks at runtime.
///
/// # Safety
/// The CPU must support the features this function enables; check with
/// `is_x86_feature_detected!` before calling.
#[unsafe(no_mangle)]
#[inline(never)]
#[target_feature(enable = "avx2")]
pub fn count_avx2(data: &[u32], t: u32) -> u64 {
    data.iter().map(|&x| (x < t) as u32).sum::<u32>() as u64
}

/// The same source again, for AVX-512: 512-bit registers, sixteen lanes.
///
/// # Safety
/// The CPU must support the features this function enables; check with
/// `is_x86_feature_detected!` before calling.
#[unsafe(no_mangle)]
#[inline(never)]
#[target_feature(enable = "avx512f,avx512bw,avx512vl")]
pub fn count_avx512(data: &[u32], t: u32) -> u64 {
    data.iter().map(|&x| (x < t) as u32).sum::<u32>() as u64
}

/// Hand-written AVX2. A lane-wise compare yields all-ones (-1) where x < t, so subtracting the
/// mask from a vector of counters adds 1 in exactly the lanes that passed.
///
/// # Safety
/// The CPU must support the features this function enables; check with
/// `is_x86_feature_detected!` before calling.
#[unsafe(no_mangle)]
#[inline(never)]
#[target_feature(enable = "avx2")]
pub fn count_intrinsics(data: &[u32], t: u32) -> u64 {
    // Values here are 0..100, so a signed compare is correct.
    let thr = _mm256_set1_epi32(t as i32);
    let mut acc = _mm256_setzero_si256();
    let chunks = data.chunks_exact(8);
    let rest = chunks.remainder();
    for c in chunks {
        // SAFETY: c has exactly 8 u32 = 32 bytes; loadu has no alignment requirement.
        let v = unsafe { _mm256_loadu_si256(c.as_ptr() as *const __m256i) };
        let lt = _mm256_cmpgt_epi32(thr, v); // t > x  ⇔  x < t
        acc = _mm256_sub_epi32(acc, lt);
    }
    let mut lanes = [0u32; 8];
    // SAFETY: lanes is 32 bytes.
    unsafe { _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, acc) };
    lanes.iter().map(|&x| x as u64).sum::<u64>() + rest.iter().filter(|&&x| x < t).count() as u64
}

/// Float addition isn't associative, so the compiler must add strictly left to right, one value
/// at a time, even when vector instructions are available.
///
/// # Safety
/// The CPU must support the features this function enables; check with
/// `is_x86_feature_detected!` before calling.
#[unsafe(no_mangle)]
#[inline(never)]
#[target_feature(enable = "avx2")]
pub fn fsum_in_order(data: &[f32]) -> f32 {
    data.iter().sum()
}

/// Sixteen independent partial sums, written out: a different order of additions, chosen by us,
/// which the compiler can map onto vector registers.
///
/// # Safety
/// The CPU must support the features this function enables; check with
/// `is_x86_feature_detected!` before calling.
#[unsafe(no_mangle)]
#[inline(never)]
#[target_feature(enable = "avx2")]
pub fn fsum_lanes(data: &[f32]) -> f32 {
    let mut acc = [0.0f32; 16];
    let chunks = data.chunks_exact(16);
    let rest = chunks.remainder();
    for c in chunks {
        for k in 0..16 {
            acc[k] += c[k];
        }
    }
    acc.iter().sum::<f32>() + rest.iter().sum::<f32>()
}

fn time_count(name: &str, data: &[u32], reps: usize, f: &dyn Fn(&[u32]) -> u64) -> f64 {
    let s = Summary::of(&repeat(2, 7, || {
        let mut t = 0;
        for _ in 0..reps {
            t += f(keep(data));
        }
        t
    }));
    let ns = s.median.as_secs_f64() * 1e9 / (data.len() * reps) as f64;
    let gbs = (data.len() * reps * 4) as f64 / s.median.as_secs_f64() / 1e9;
    println!(
        "{name:<28} {ns:>7.3} ns/value  {gbs:>7.1} GB/s   noise ±{:.1}%",
        s.cv * 100.0
    );
    ns
}

fn main() {
    assert!(is_x86_feature_detected!("avx2"), "this example needs AVX2");
    let has512 = is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw");
    println!("CPU features: avx2 = true, avx512f+bw = {has512}\n");

    for (label, n, reps) in [
        (
            "64 KiB of u32, in L2, repeated 4,000 times",
            16 * 1024,
            4000,
        ),
        ("256 MiB of u32, from main memory", 64 * 1024 * 1024, 2),
    ] {
        let data = percent_data(n, 3);
        let expect = count_scalar(&data, 50);
        // SAFETY (all unsafe calls below): the features were detected at runtime above.
        assert_eq!(count_auto(&data, 50), expect);
        assert_eq!(unsafe { count_avx2(&data, 50) }, expect);
        assert_eq!(unsafe { count_intrinsics(&data, 50) }, expect);
        if has512 {
            assert_eq!(unsafe { count_avx512(&data, 50) }, expect);
        }
        println!("== Counting values < 50: {label} ==\n");
        let mut csv = Vec::new();
        let base = time_count("scalar (no SIMD)", &data, reps, &|d| count_scalar(d, 50));
        csv.push(format!("scalar,{base:.4}"));
        let a = time_count("auto, default target (SSE2)", &data, reps, &|d| {
            count_auto(d, 50)
        });
        csv.push(format!("sse2,{a:.4}"));
        let b = time_count("auto, AVX2", &data, reps, &|d| unsafe { count_avx2(d, 50) });
        csv.push(format!("avx2,{b:.4}"));
        if has512 {
            let c = time_count("auto, AVX-512", &data, reps, &|d| unsafe {
                count_avx512(d, 50)
            });
            csv.push(format!("avx512,{c:.4}"));
        }
        let d = time_count("intrinsics, AVX2", &data, reps, &|d| unsafe {
            count_intrinsics(d, 50)
        });
        csv.push(format!("intrinsics,{d:.4}"));
        println!("\ncsv: {}\n", csv.join(";"));
    }

    println!("== Summing 64 KiB of f32, repeated 4,000 times ==\n");
    let f: Vec<f32> = (0..16 * 1024).map(|i| (i % 7) as f32 * 0.25).collect();
    for (name, g) in [
        (
            "in order (as written)",
            fsum_in_order as unsafe fn(&[f32]) -> f32,
        ),
        ("16 partial sums", fsum_lanes),
    ] {
        let mut total = 0.0;
        let s = Summary::of(&repeat(2, 7, || {
            let mut t = 0.0;
            for _ in 0..4000 {
                // SAFETY: AVX2 detected above.
                t += unsafe { g(keep(&f)) };
            }
            total = t;
            t
        }));
        println!(
            "{name:<24} {:>7.3} ns/value   result {total}",
            s.median.as_secs_f64() * 1e9 / (f.len() * 4000) as f64
        );
    }
}
