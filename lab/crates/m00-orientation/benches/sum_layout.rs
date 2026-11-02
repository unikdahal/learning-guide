//! Article: "Measuring things honestly". The row-vs-column sum, measured with Criterion:
//! warm-up, many samples, confidence intervals, and comparison with the previous run.
//!
//!     cargo bench -p m00-orientation --bench sum_layout

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use m00_orientation::orders;

fn sum_layout(c: &mut Criterion) {
    let n = 1_000_000;
    let rows = orders(n);
    let amounts: Vec<i64> = rows.iter().map(|o| o.amount).collect();

    let mut group = c.benchmark_group("sum_amount");
    group.throughput(Throughput::Elements(n as u64));
    group.bench_function("row_layout", |b| {
        b.iter(|| black_box(&rows).iter().map(|o| o.amount).sum::<i64>())
    });
    group.bench_function("column_layout", |b| {
        b.iter(|| black_box(&amounts).iter().sum::<i64>())
    });
    group.finish();
}

criterion_group!(benches, sum_layout);
criterion_main!(benches);
