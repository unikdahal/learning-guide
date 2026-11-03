//! Article: "Branch prediction and why branches hurt".
//! A filter with an `if` against the same filter without a branch, at every selectivity, on
//! random and on sorted input.
//!
//!     cargo run --release -p m02-hardware --example branches

use labkit::timing::{Summary, repeat};
use m02_hardware::{filter_branchless, filter_branchy, keep, percent_data};

fn main() {
    let n = 10_000_000;
    let data = percent_data(n, 1);
    let mut sorted = data.clone();
    sorted.sort_unstable();
    let mut out = Vec::with_capacity(n + 1);

    println!("== Filtering 10M random values: ns per row ==\n");
    println!(
        "{:>11}  {:>8}  {:>10}  {:>14}",
        "selectivity", "kept", "with if", "without branch"
    );
    let mut csv = Vec::new();
    for t in (0..=100).step_by(5) {
        let a = Summary::of(&repeat(2, 7, || {
            filter_branchy(keep(&data), t, &mut out);
            out.len()
        }));
        let kept = out.len();
        let b = Summary::of(&repeat(2, 7, || {
            filter_branchless(keep(&data), t, &mut out);
            out.len()
        }));
        let na = a.median.as_secs_f64() * 1e9 / n as f64;
        let nb = b.median.as_secs_f64() * 1e9 / n as f64;
        println!("{:>10}%  {kept:>8}  {na:>10.2}  {nb:>14.2}", t);
        csv.push(format!("{t},{na:.3},{nb:.3}"));
    }
    println!("\nfilter csv: {}\n", csv.join(";"));

    println!("== The same 50% filter on sorted input ==\n");
    for (name, d) in [("random order", &data), ("sorted", &sorted)] {
        let a = Summary::of(&repeat(2, 7, || {
            filter_branchy(keep(d), 50, &mut out);
            out.len()
        }));
        let b = Summary::of(&repeat(2, 7, || {
            filter_branchless(keep(d), 50, &mut out);
            out.len()
        }));
        println!(
            "{name:<13} with if {:>6.2} ns/row   without branch {:>6.2} ns/row",
            a.median.as_secs_f64() * 1e9 / n as f64,
            b.median.as_secs_f64() * 1e9 / n as f64
        );
    }
}
