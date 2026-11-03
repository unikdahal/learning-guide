//! Article: "Lightweight encodings".
//! Five 10M-value Int64 columns with different shapes, each encoded with bit-packing, frame of
//! reference (whole column and per 1024-value block), delta and run-length encoding: the bits per
//! value each needs, and how fast the best one decodes.
//!
//!     cargo run --release -p m03-representation --example encodings

use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use m03_representation::encodings::{BitPacked, BlockFor, Delta, ForPacked, Rle};
use std::hint::black_box;

fn ms<R>(f: impl FnMut() -> R) -> f64 {
    Summary::of(&repeat(1, 7, f)).median.as_secs_f64() * 1e3
}

fn main() {
    let n = 10_000_000usize;
    let mut rng = Rng::new(41);
    let mut ts = 1_700_000_000_000i64;
    let mut status = 0i64;
    let columns: Vec<(&str, Vec<i64>)> = vec![
        (
            "event time, sorted ms",
            (0..n)
                .map(|_| {
                    ts += rng.below(1000) as i64;
                    ts
                })
                .collect(),
        ),
        (
            "quantity, 1 to 100",
            (0..n).map(|_| 1 + rng.below(100) as i64).collect(),
        ),
        (
            "price in cents, 10000-10500",
            (0..n).map(|_| 10_000 + rng.below(501) as i64).collect(),
        ),
        (
            "status, runs of ~1000",
            (0..n)
                .map(|_| {
                    if rng.below(1000) == 0 {
                        status = rng.below(4) as i64;
                    }
                    status
                })
                .collect(),
        ),
        (
            "hash, random 64-bit",
            (0..n).map(|_| rng.next_u64() as i64).collect(),
        ),
    ];

    println!("bits per value ({n} values each; plain Int64 is 64)\n");
    println!(
        "{:<28} {:>9} {:>9} {:>10} {:>9} {:>9}",
        "column", "bitpack", "FOR", "FOR/1024", "delta", "RLE"
    );
    let mut csv = Vec::new();
    for (name, v) in &columns {
        let bpv = |bytes: usize| bytes as f64 * 8.0 / n as f64;
        let bp = if v.iter().all(|&x| x >= 0) {
            format!(
                "{:.2}",
                bpv(BitPacked::pack(&v.iter().map(|&x| x as u64).collect::<Vec<_>>()).bytes())
            )
        } else {
            "n/a".into()
        };
        let f = bpv(ForPacked::encode(v).bytes());
        let fb = bpv(BlockFor::encode(v).bytes());
        let d = bpv(Delta::encode(v).bytes());
        let r = bpv(Rle::encode(v).bytes());
        println!("{name:<28} {bp:>9} {f:>9.2} {fb:>10.2} {d:>9.2} {r:>9.2}");
        csv.push(format!("{name},{bp},{f:.2},{fb:.2},{d:.2},{r:.2}"));
    }
    println!("\nbits csv: {}", csv.join(";"));

    println!("\n== Decoding 10M values back to Int64 ==\n");
    let mut out = vec![0i64; n];
    let copy = ms(|| {
        out.copy_from_slice(&columns[1].1);
        out[0]
    });
    println!(
        "{:<40} {copy:>7.2} ms  (memcpy of plain Int64, for scale)",
        "plain copy"
    );
    type Decoder = Box<dyn Fn(&mut [i64])>;
    let cases: Vec<(&str, Decoder)> = vec![
        ("FOR/1024, quantity (7 bits)", {
            let e = BlockFor::encode(&columns[1].1);
            Box::new(move |o: &mut [i64]| e.decode_into(o))
        }),
        ("FOR/1024, price (9 bits)", {
            let e = BlockFor::encode(&columns[2].1);
            Box::new(move |o: &mut [i64]| e.decode_into(o))
        }),
        ("delta, event time (10 bits)", {
            let e = Delta::encode(&columns[0].1);
            Box::new(move |o: &mut [i64]| e.decode_into(o))
        }),
        ("RLE, status", {
            let e = Rle::encode(&columns[3].1);
            Box::new(move |o: &mut [i64]| e.decode_into(o))
        }),
    ];
    for (name, dec) in &cases {
        let t = ms(|| {
            dec(black_box(&mut out));
            out[n - 1]
        });
        println!(
            "{name:<40} {t:>7.2} ms  {:>5.2} ns/value  {:>5.1} GB/s of Int64 out",
            t * 1e6 / n as f64,
            n as f64 * 8.0 / (t * 1e-3) / 1e9
        );
    }

    println!("\n== Summing without decoding ==\n");
    let rle = Rle::encode(&columns[3].1);
    let plain_sum = ms(|| columns[3].1.iter().sum::<i64>());
    let rle_sum = ms(|| rle.runs.iter().map(|&(v, k)| v * k as i64).sum::<i64>());
    println!(
        "status: plain sum {plain_sum:.2} ms, sum over {} runs {rle_sum:.4} ms",
        rle.runs.len()
    );
}
