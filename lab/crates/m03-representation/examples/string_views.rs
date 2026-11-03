//! Article: "String layouts: offsets vs views".
//! Prints the 16-byte views behind a StringViewArray, then compares it with the offset-based
//! StringArray on size, equality filters, take and sort.
//!
//!     cargo run --release -p m03-representation --example string_views

use datafusion::arrow::array::{Array, Scalar, StringArray, StringViewArray, UInt32Array};
use datafusion::arrow::compute::kernels::cmp::eq;
use datafusion::arrow::compute::{sort_to_indices, take};
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use std::hint::black_box;

fn ms<R>(f: impl FnMut() -> R) -> f64 {
    Summary::of(&repeat(1, 7, f)).median.as_secs_f64() * 1e3
}

fn show_view(v: u128) {
    let b = v.to_le_bytes();
    let len = u32::from_le_bytes(b[0..4].try_into().unwrap());
    let hex: Vec<String> = b.iter().map(|x| format!("{x:02x}")).collect();
    println!("  bytes: {}", hex.join(" "));
    if len <= 12 {
        println!(
            "  length {len}, inline data {:?}",
            String::from_utf8_lossy(&b[4..4 + len as usize])
        );
    } else {
        println!(
            "  length {len}, prefix {:?}, buffer {}, offset {}",
            String::from_utf8_lossy(&b[4..8]),
            u32::from_le_bytes(b[8..12].try_into().unwrap()),
            u32::from_le_bytes(b[12..16].try_into().unwrap())
        );
    }
}

fn main() {
    println!("== Views ==\n");
    let sv = StringViewArray::from(vec!["hash", "a string longer than twelve", "sort"]);
    for (i, &v) in sv.views().iter().enumerate() {
        println!("value {i}: {:?}", sv.value(i));
        show_view(v);
    }
    println!(
        "data buffers: {} ({} bytes: only the long string)",
        sv.data_buffers().len(),
        sv.data_buffers().iter().map(|b| b.len()).sum::<usize>()
    );

    let n = 5_000_000usize;
    let mut rng = Rng::new(21);
    let sets: [(&str, Vec<String>, String); 3] = [
        (
            "short codes (8 bytes)",
            (0..n)
                .map(|_| format!("SKU{:05}", rng.below(100_000)))
                .collect(),
            "SKU04242".into(),
        ),
        (
            "long, distinct prefixes",
            (0..n)
                .map(|_| format!("{:05}-customer-account", rng.below(100_000)))
                .collect(),
            "04242-customer-account".into(),
        ),
        (
            "long, shared prefix",
            (0..n)
                .map(|_| format!("customer-account-{:05}", rng.below(100_000)))
                .collect(),
            "customer-account-04242".into(),
        ),
    ];
    let mut rng = Rng::new(22);
    let idx = UInt32Array::from_iter_values((0..n / 2).map(|_| rng.below(n as u64) as u32));

    for (name, data, target) in &sets {
        println!("\n== {n} strings: {name} ==\n");
        let off = StringArray::from_iter_values(data.iter());
        let view = StringViewArray::from_iter_values(data.iter());
        let mb = |a: &dyn Array| a.get_array_memory_size() as f64 / 1e6;
        println!(
            "size:     offsets {:>7.1} MB   views {:>7.1} MB",
            mb(&off),
            mb(&view)
        );
        let so = Scalar::new(StringArray::from(vec![target.as_str()]));
        let sw = Scalar::new(StringViewArray::from(vec![target.as_str()]));
        let a = ms(|| eq(black_box(&off), &so).unwrap());
        let b = ms(|| eq(black_box(&view), &sw).unwrap());
        println!("eq:       offsets {a:>7.2} ms   views {b:>7.2} ms");
        let a = ms(|| take(black_box(&off), &idx, None).unwrap());
        let b = ms(|| take(black_box(&view), &idx, None).unwrap());
        println!("take n/2: offsets {a:>7.2} ms   views {b:>7.2} ms");
        let a = ms(|| sort_to_indices(black_box(&off), None, None).unwrap());
        let b = ms(|| sort_to_indices(black_box(&view), None, None).unwrap());
        println!("sort:     offsets {a:>7.0} ms   views {b:>7.0} ms");
    }
}
