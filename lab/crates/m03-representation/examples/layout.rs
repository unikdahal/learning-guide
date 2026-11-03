//! Article: "Row vs column layout".
//! The same 2 million records of 16 fields, stored row by row and column by column: scanning
//! k of the 16 fields, and fetching whole records by position.
//!
//!     cargo run --release -p m03-representation --example layout

use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use m03_representation::{Columns, FIELDS, Rows};
use std::hint::black_box;

fn main() {
    let n = 2_000_000;
    let rows = Rows::generate(n);
    let cols = Columns::from_rows(&rows);
    println!(
        "{n} records x {FIELDS} fields of 8 bytes = {} MiB in each layout\n",
        (n * FIELDS * 8) >> 20
    );

    println!("== Scan: sum k of the 16 fields over every record ==\n");
    println!(
        "{:>3}  {:>10}  {:>10}  {:>14}",
        "k", "rows ms", "columns ms", "rows / columns"
    );
    let mut csv = Vec::new();
    for k in [1usize, 2, 4, 8, 16] {
        let a = Summary::of(&repeat(1, 5, || rows.sum_fields(black_box(k))));
        let b = Summary::of(&repeat(1, 5, || cols.sum_fields(black_box(k))));
        let (ma, mb) = (a.median.as_secs_f64() * 1e3, b.median.as_secs_f64() * 1e3);
        println!("{k:>3}  {ma:>10.1}  {mb:>10.1}  {:>13.1}x", ma / mb);
        csv.push(format!("{k},{ma:.2},{mb:.2}"));
    }
    println!("\nscan csv: {}\n", csv.join(";"));

    println!("== Point lookups: read all 16 fields of 1M random records ==\n");
    let mut rng = Rng::new(3);
    let ids: Vec<u32> = (0..1_000_000).map(|_| rng.below(n as u64) as u32).collect();
    let a = Summary::of(&repeat(1, 5, || rows.fetch(black_box(&ids))));
    let b = Summary::of(&repeat(1, 5, || cols.fetch(black_box(&ids))));
    println!(
        "rows    {:>7.1} ms  ({:.1} ns per record)",
        a.median.as_secs_f64() * 1e3,
        a.median.as_secs_f64() * 1e9 / ids.len() as f64
    );
    println!(
        "columns {:>7.1} ms  ({:.1} ns per record)",
        b.median.as_secs_f64() * 1e3,
        b.median.as_secs_f64() * 1e9 / ids.len() as f64
    );
}
