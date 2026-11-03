//! Article: "Selection vectors vs filtered copies".
//! A 10M-row batch of four columns and a filter mask at several selectivities. Measures making a
//! filtered copy of the batch, turning the mask into a selection vector of indices, and computing
//! sum(a) for the selected rows from a copy, from indices, and straight from the bitmap.
//!
//!     cargo run --release -p m03-representation --example selection

use std::sync::Arc;

use datafusion::arrow::array::{
    Array, ArrayRef, BooleanArray, Int64Array, RecordBatch, StringArray, UInt32Array,
};
use datafusion::arrow::compute::{filter, filter_record_batch, sum, take};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use std::hint::black_box;

fn ms<R>(f: impl FnMut() -> R) -> f64 {
    Summary::of(&repeat(1, 7, f)).median.as_secs_f64() * 1e3
}

/// Minor page faults so far for this process (field 10 of /proc/self/stat; Linux only).
fn minor_faults() -> u64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let after = s.rsplit(')').next().unwrap_or("");
    after
        .split_whitespace()
        .nth(7)
        .and_then(|x| x.parse().ok())
        .unwrap_or(0)
}

fn main() {
    let n = 10_000_000usize;
    let mut rng = Rng::new(31);
    let a = Int64Array::from_iter_values((0..n).map(|i| (i % 1000) as i64));
    let b = Int64Array::from_iter_values((0..n).map(|_| rng.below(1 << 20) as i64));
    let c = Int64Array::from_iter_values((0..n).map(|i| i as i64));
    let d = StringArray::from_iter_values((0..n).map(|i| format!("order-{:07}", i % 1_000_000)));
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Int64, false),
        Field::new("b", DataType::Int64, false),
        Field::new("c", DataType::Int64, false),
        Field::new("d", DataType::Utf8, false),
    ]));
    let cols: Vec<ArrayRef> = vec![Arc::new(a.clone()), Arc::new(b), Arc::new(c), Arc::new(d)];
    let batch = RecordBatch::try_new(schema, cols).unwrap();
    println!("batch: {n} rows, columns a, b, c (Int64) and d (Utf8)\n");
    println!(
        "{:>7}  {:>11}  {:>10}  {:>10}  {:>11}  {:>11}  {:>11}",
        "select", "copy batch", "mask->idx", "take a", "copy a+sum", "idx+sum a", "bitmap sum"
    );
    let mut csv = Vec::new();
    let mut faults_log = Vec::new();
    for p in [0.001f64, 0.01, 0.1, 0.5, 0.9] {
        let mut rng = Rng::new(32);
        let mask = BooleanArray::from_iter((0..n).map(|_| Some(rng.unit() < p)));
        let copy_batch = ms(|| filter_record_batch(black_box(&batch), &mask).unwrap());
        let f0 = minor_faults();
        let out = filter_record_batch(&batch, &mask).unwrap();
        let faults = minor_faults() - f0;
        let out_mb = out.get_array_memory_size() as f64 / 1e6;
        drop(out);
        let indices =
            || UInt32Array::from_iter_values(mask.values().set_indices().map(|i| i as u32));
        let to_idx = ms(indices);
        let idx = indices();
        let take_a = ms(|| take(black_box(&a), &idx, None).unwrap());
        let copy_sum = ms(|| {
            let f = filter(black_box(&a), &mask).unwrap();
            sum(f.as_any().downcast_ref::<Int64Array>().unwrap())
        });
        let idx_sum = ms(|| {
            let v = a.values();
            idx.values().iter().map(|&i| v[i as usize]).sum::<i64>()
        });
        let bitmap_sum = ms(|| {
            // Branch-free: each 64-bit word of the mask selects lanes of a 64-value chunk.
            let v = a.values();
            let words = mask.values().inner().typed_data::<u64>();
            let mut s = 0i64;
            for (w, chunk) in words.iter().zip(v.chunks(64)) {
                for (j, x) in chunk.iter().enumerate() {
                    s += x & -(((w >> j) & 1) as i64);
                }
            }
            s
        });
        println!(
            "{:>6}%  {copy_batch:>8.2} ms  {to_idx:>7.2} ms  {take_a:>7.2} ms  {copy_sum:>8.2} ms  {idx_sum:>8.2} ms  {bitmap_sum:>8.2} ms",
            p * 100.0
        );
        faults_log.push((p, out_mb, faults));
        csv.push(format!(
            "{p},{copy_batch:.2},{to_idx:.2},{take_a:.2},{copy_sum:.2},{idx_sum:.2},{bitmap_sum:.2}"
        ));
    }
    println!("\nselection csv: {}", csv.join(";"));
    println!("\n== Page faults while making one filtered copy of the batch ==\n");
    for (p, mb, f) in &faults_log {
        println!(
            "{:>6}%  copy holds {mb:>7.1} MB   minor page faults {f:>7}",
            p * 100.0
        );
    }
}
