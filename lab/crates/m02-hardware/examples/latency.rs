//! Article: "The memory hierarchy, from registers to object storage".
//! Measures the latency of one dependent memory access, and the bandwidth of a sequential scan,
//! as the working set grows from a few kilobytes to a gigabyte.
//!
//!     cargo run --release -p m02-hardware --example latency

use labkit::timing::{Summary, repeat};
use m02_hardware::{LINE, chase, keep, random_cycle, sum_sequential};

fn human(bytes: usize) -> String {
    match bytes {
        b if b >= 1 << 30 => format!("{} GiB", b >> 30),
        b if b >= 1 << 20 => format!("{} MiB", b >> 20),
        b => format!("{} KiB", b >> 10),
    }
}

fn main() {
    let sizes: Vec<usize> = (12..=30).map(|p| 1usize << p).collect(); // 4 KiB .. 1 GiB

    println!("== Latency: one dependent load at a time (pointer chasing) ==\n");
    println!("{:>9}  {:>10}  {:>8}", "working set", "ns / load", "noise");
    let mut csv = Vec::new();
    for &size in &sizes {
        let n = size / LINE;
        let nodes = random_cycle(n, 42);
        let steps = n.max(20_000_000);
        let runs = if size >= 1 << 28 { 3 } else { 7 };
        let samples = repeat(1, runs, || keep(chase(&nodes, steps)));
        let s = Summary::of(&samples);
        let ns = s.median.as_secs_f64() * 1e9 / steps as f64;
        println!("{:>11}  {:>10.2}  {:>7.1}%", human(size), ns, s.cv * 100.0);
        csv.push(format!("{size},{ns:.3}"));
    }
    println!("\nlatency csv: {}\n", csv.join(";"));

    println!("== Bandwidth: one thread summing a buffer in order ==\n");
    println!("{:>9}  {:>10}  {:>8}", "working set", "GB / s", "noise");
    let mut csv = Vec::new();
    for &size in sizes.iter().filter(|&&s| s >= 16 << 10) {
        let data: Vec<u64> = (0..(size / 8) as u64).collect();
        // Repeat the pass so each sample reads at least 2 GiB.
        let passes = ((2usize << 30) / size).max(1);
        let samples = repeat(1, 5, || {
            let mut t = 0u64;
            for _ in 0..passes {
                t = t.wrapping_add(sum_sequential(keep(&data)));
            }
            t
        });
        let s = Summary::of(&samples);
        let gbs = (size * passes) as f64 / s.median.as_secs_f64() / 1e9;
        println!("{:>11}  {:>10.1}  {:>7.1}%", human(size), gbs, s.cv * 100.0);
        csv.push(format!("{size},{gbs:.2}"));
    }
    println!("\nbandwidth csv: {}", csv.join(";"));
}
