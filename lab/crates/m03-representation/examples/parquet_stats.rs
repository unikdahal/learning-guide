//! Article: "Parquet row groups, pages and statistics".
//! Writes the same 10M orders sorted and shuffled by time, with different row group sizes, then
//! asks DataFusion for one hour of orders and reports how many row groups its statistics pruned,
//! how many bytes it read and how long the query took.
//!
//!     cargo run --release -p m03-representation --example parquet_stats

use std::fs::File;
use std::sync::Arc;

use datafusion::arrow::array::{ArrayRef, Float64Array, Int64Array, RecordBatch};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::parquet::arrow::ArrowWriter;
use datafusion::parquet::file::properties::WriterProperties;
use datafusion::parquet::file::reader::FileReader;
use datafusion::parquet::file::serialized_reader::SerializedFileReader;
use datafusion::parquet::file::statistics::Statistics;
use datafusion::prelude::*;
use labkit::rng::Rng;
use std::time::Instant;

const N: usize = 10_000_000;
const T0: i64 = 1_700_000_000_000;

fn write(path: &std::path::Path, ts: &[i64], amount: &[f64], rows_per_group: usize) {
    let schema = Arc::new(Schema::new(vec![
        Field::new("order_id", DataType::Int64, false),
        Field::new("ts", DataType::Int64, false),
        Field::new("amount", DataType::Float64, false),
    ]));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(0..N as i64)),
        Arc::new(Int64Array::from(ts.to_vec())),
        Arc::new(Float64Array::from(amount.to_vec())),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();
    let props = WriterProperties::builder()
        .set_max_row_group_row_count(Some(rows_per_group))
        .build();
    let mut w = ArrowWriter::try_new(File::create(path).unwrap(), schema, Some(props)).unwrap();
    for start in (0..N).step_by(1 << 20) {
        w.write(&batch.slice(start, (1 << 20).min(N - start)))
            .unwrap();
    }
    w.close().unwrap();
}

fn metric(plan: &str, name: &str) -> String {
    plan.split([',', '[', ']'])
        .map(str::trim)
        .find(|s| s.starts_with(&format!("{name}=")))
        .map(|s| s[name.len() + 1..].to_string())
        .unwrap_or_else(|| "?".into())
}

#[tokio::main]
async fn main() {
    let mut rng = Rng::new(81);
    // One order every ~0.6 s on average for ~70 days, in time order.
    let mut t = T0;
    let sorted: Vec<i64> = (0..N)
        .map(|_| {
            t += rng.below(1200) as i64;
            t
        })
        .collect();
    let amount: Vec<f64> = (0..N).map(|_| rng.below(100_000) as f64 / 100.0).collect();
    let mut shuffled = sorted.clone();
    for i in (1..N).rev() {
        shuffled.swap(i, rng.below(i as u64 + 1) as usize);
    }
    let lo = sorted[N / 2];
    let hi = lo + 3_600_000;

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();
    let cases = [
        ("sorted, 100k rows/group", &sorted, 100_000usize),
        ("shuffled, 100k rows/group", &shuffled, 100_000),
        ("sorted, 1M rows/group", &sorted, 1_000_000),
        ("sorted, 10k rows/group", &sorted, 10_000),
    ];
    println!("{N} orders; query: one hour of ts, sum(amount)\n");
    for (name, ts, rpg) in cases {
        let path = dir.join("m03-stats.parquet");
        write(&path, ts, &amount, rpg);
        let r = SerializedFileReader::new(File::open(&path).unwrap()).unwrap();
        let md = r.metadata();
        let bytes = std::fs::read(&path).unwrap();
        let len = bytes.len();
        let footer = u32::from_le_bytes(bytes[len - 8..len - 4].try_into().unwrap());
        if name.starts_with("shuffled") || rpg == 100_000 {
            let show: Vec<String> = md.row_groups()[..3]
                .iter()
                .map(|rg| match rg.column(1).statistics() {
                    Some(Statistics::Int64(s)) => format!(
                        "[{}, {}]",
                        (s.min_opt().unwrap() - T0) / 1000,
                        (s.max_opt().unwrap() - T0) / 1000
                    ),
                    _ => "?".into(),
                })
                .collect();
            println!(
                "{name}: ts min/max of the first row groups, seconds from start: {}",
                show.join(" ")
            );
        }

        let ctx = SessionContext::new();
        ctx.register_parquet("o", path.to_str().unwrap(), ParquetReadOptions::default())
            .await
            .unwrap();
        let sql = format!("SELECT count(*), sum(amount) FROM o WHERE ts BETWEEN {lo} AND {hi}");
        let mut times = Vec::new();
        for _ in 0..5 {
            let t = Instant::now();
            ctx.sql(&sql).await.unwrap().collect().await.unwrap();
            times.push(t.elapsed().as_secs_f64() * 1e3);
        }
        times.sort_by(f64::total_cmp);
        let res = ctx.sql(&sql).await.unwrap().collect().await.unwrap();
        let plan = ctx
            .sql(&format!("EXPLAIN ANALYZE {sql}"))
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
        let plan = pretty_format_batches(&plan).unwrap().to_string();
        println!(
            "{name:<26} {:>4} row groups, footer {:>7} B | pruned by stats {:>10} | page-index rows pruned {:>12} | bytes scanned {:>10} | {:>6.1} ms | {}",
            md.num_row_groups(),
            footer,
            metric(&plan, "row_groups_pruned_statistics"),
            metric(&plan, "page_index_rows_pruned"),
            metric(&plan, "bytes_scanned"),
            times[2],
            pretty_format_batches(&res)
                .unwrap()
                .to_string()
                .lines()
                .nth(3)
                .unwrap_or("")
                .trim()
        );
        std::fs::remove_file(&path).unwrap();
    }
}
