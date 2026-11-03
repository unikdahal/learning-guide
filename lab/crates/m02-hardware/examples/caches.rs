//! Article: "CPU caches, cache lines and locality".
//! Strided reads, row- vs column-order traversal, and gathers, each showing that the unit of
//! memory traffic is the 64-byte cache line, not the value you asked for.
//!
//!     cargo run --release -p m02-hardware --example caches

use labkit::timing::{Summary, repeat};
use m02_hardware::{gather, keep, shuffle, sum_col_major, sum_row_major, touch_strided};

fn main() {
    println!("== Reading one byte every `stride` bytes of a 1 GiB buffer ==\n");
    let buf: Vec<u8> = (0..1usize << 30).map(|i| i as u8).collect();
    println!(
        "{:>7}  {:>12}  {:>10}  {:>12}",
        "stride", "bytes read", "total ms", "ns / access"
    );
    let mut csv = Vec::new();
    for stride in [1usize, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 4096] {
        let mut n = 0;
        let samples = repeat(1, 5, || {
            let (s, k) = touch_strided(keep(&buf), stride);
            n = k;
            s
        });
        let s = Summary::of(&samples);
        let ms = s.median.as_secs_f64() * 1e3;
        let ns = s.median.as_secs_f64() * 1e9 / n as f64;
        println!("{stride:>7}  {n:>12}  {ms:>10.1}  {ns:>12.2}");
        csv.push(format!("{stride},{ms:.2},{ns:.3}"));
    }
    println!("\nstride csv: {}\n", csv.join(";"));
    drop(buf);

    println!("== Summing a 8192 x 8192 matrix of u32 (256 MiB), stored row by row ==\n");
    let (rows, cols) = (8192usize, 8192usize);
    let m: Vec<u32> = (0..(rows * cols) as u32).collect();
    for (name, f) in [
        (
            "row by row",
            sum_row_major as fn(&[u32], usize, usize) -> u64,
        ),
        ("column by column", sum_col_major),
    ] {
        let s = Summary::of(&repeat(1, 5, || f(keep(&m), rows, cols)));
        println!("{name:<18} {s}");
    }
    drop(m);

    println!("\n== Gathering 64M values by index ==\n");
    let n = 1usize << 26; // 64M u32 = 256 MiB
    let data: Vec<u32> = (0..n as u32).collect();
    let sequential: Vec<u32> = (0..n as u32).collect();
    let mut random_small: Vec<u32> = (0..n as u32).map(|i| i % (1 << 18)).collect(); // within 1 MiB
    shuffle(&mut random_small, 9);
    let mut random_l3ish: Vec<u32> = (0..n as u32).map(|i| i % (1 << 23)).collect(); // within 32 MiB
    shuffle(&mut random_l3ish, 10);
    let mut random_all = sequential.clone();
    shuffle(&mut random_all, 11);
    for (name, idx) in [
        ("in order", &sequential),
        ("random, within 1 MiB", &random_small),
        ("random, within 32 MiB", &random_l3ish),
        ("random, all 256 MiB", &random_all),
    ] {
        let s = Summary::of(&repeat(1, 5, || gather(keep(&data), keep(idx))));
        let ns = s.median.as_secs_f64() * 1e9 / n as f64;
        println!("{name:<24} {:>7.2} ns / value   {s}", ns);
    }
}
