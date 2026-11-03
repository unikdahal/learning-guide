//! Articles: "The memory hierarchy, from registers to object storage", and later "TLBs, pages
//! and huge pages". The same pointer chase as `latency`, on ordinary 4 KiB pages and on 2 MiB
//! transparent huge pages, to separate cache misses from address-translation misses.
//!
//!     cargo run --release -p m02-hardware --example tlb

use labkit::timing::{Summary, repeat};
use m02_hardware::{HugeNodes, LINE, anon_huge_kib, chase, keep, link_random_cycle, random_cycle};

fn main() {
    println!("== Pointer chase: 4 KiB pages vs 2 MiB huge pages ==\n");
    println!(
        "{:>11}  {:>12}  {:>12}  {:>14}",
        "working set", "4 KiB pages", "2 MiB pages", "huge-backed"
    );
    let mut csv = Vec::new();
    for p in 20..=30 {
        let size = 1usize << p;
        let n = size / LINE;
        let steps = n.max(20_000_000);
        let runs = if size >= 1 << 28 { 3 } else { 5 };

        let small = random_cycle(n, 42);
        let a = Summary::of(&repeat(1, runs, || keep(chase(&small, steps))));
        drop(small);

        let before = anon_huge_kib();
        let mut huge = HugeNodes::new(n);
        link_random_cycle(huge.as_mut_slice(), 42);
        let backed = anon_huge_kib().saturating_sub(before);
        let b = Summary::of(&repeat(1, runs, || keep(chase(huge.as_slice(), steps))));
        drop(huge);

        let na = a.median.as_secs_f64() * 1e9 / steps as f64;
        let nb = b.median.as_secs_f64() * 1e9 / steps as f64;
        let label = if size >= 1 << 30 {
            format!("{} GiB", size >> 30)
        } else {
            format!("{} MiB", size >> 20)
        };
        let pct = 100.0 * backed as f64 / (size as f64 / 1024.0);
        println!("{label:>11}  {na:>9.1} ns  {nb:>9.1} ns  {pct:>13.0}%");
        csv.push(format!("{size},{na:.2},{nb:.2}"));
    }
    println!("\ntlb csv: {}", csv.join(";"));
}
