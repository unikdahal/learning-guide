//! Article: "Instruction-level parallelism".
//! The same number of multiply-adds arranged as 1 to 12 independent dependency chains. The work
//! is identical; only the dependencies change, and with them how many operations the core can
//! overlap. Everything stays in registers, so memory plays no part.
//!
//!     cargo run --release -p m02-hardware --example ilp
//!
//! Check that the loops stayed scalar (one `imul` and one `add` per step, no vector registers):
//!     objdump -d --no-show-raw-insn -C target/release/examples/ilp | grep -A40 'chains::<4>'

use labkit::timing::{Summary, repeat};
use m02_hardware::keep;

/// `n` multiply-adds spread over `K` independent chains: x_k = x_k * a + c. The constants come in
/// through `keep`, so the compiler can't precompute several steps at once. (With literal
/// constants it folds eight steps into one multiply-add, which wrapping integer arithmetic
/// allows; the article shows that machine code.)
#[inline(never)]
fn chains<const K: usize>(n: usize, a: u64, c: u64) -> u64 {
    let mut x = [0u64; K];
    for (k, v) in x.iter_mut().enumerate() {
        *v = k as u64 + 1;
    }
    for _ in 0..n / K {
        for v in x.iter_mut() {
            *v = v.wrapping_mul(a).wrapping_add(c);
        }
    }
    x.iter().fold(0, |s, &v| s ^ v)
}

fn main() {
    let n = 240_000_000;
    let (a, c) = (keep(6364136223846793005u64), keep(1442695040888963407u64));
    println!("== 240M multiply-adds, split over K independent chains ==\n");
    println!("{:>7}  {:>10}  {:>9}", "chains", "ns per op", "speedup");
    let mut base = 0.0;
    let mut csv = Vec::new();
    macro_rules! run {
        ($k:literal) => {{
            let s = Summary::of(&repeat(1, 5, || chains::<$k>(n, a, c)));
            let ns = s.median.as_secs_f64() * 1e9 / n as f64;
            if $k == 1 {
                base = ns;
            }
            println!(
                "{:>7}  {ns:>10.3}  {:>8.2}x   noise ±{:.1}%",
                $k,
                base / ns,
                s.cv * 100.0
            );
            csv.push(format!("{},{ns:.4}", $k));
        }};
    }
    run!(1);
    run!(2);
    run!(3);
    run!(4);
    run!(6);
    run!(8);
    run!(12);
    println!("\nchains csv: {}", csv.join(";"));
}
