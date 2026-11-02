//! Article: "OLTP vs OLAP workloads". Summing one field from rows vs from a column.
//!
//!     cargo run --release -p m00-orientation --example row_vs_column

use std::hint::black_box;
use std::time::Instant;

use m00_orientation::orders;

fn main() {
    const N: usize = 8_000_000;
    let rows = orders(N);
    // The same data, column-wise: just the amounts.
    let amounts: Vec<i64> = rows.iter().map(|o| o.amount).collect();

    for round in 1..=5 {
        let t = Instant::now();
        let total_rows: i64 = black_box(&rows).iter().map(|o| o.amount).sum();
        let row_time = t.elapsed();

        let t = Instant::now();
        let total_cols: i64 = black_box(&amounts).iter().sum();
        let col_time = t.elapsed();

        assert_eq!(total_rows, total_cols);
        println!(
            "round {round}: row layout {row_time:>10.2?}   column layout {col_time:>10.2?}   ratio {:.1}x",
            row_time.as_secs_f64() / col_time.as_secs_f64()
        );
    }
}
