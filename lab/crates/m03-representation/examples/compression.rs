//! Article: "General-purpose compression".
//! Compresses four kinds of data with LZ4, Snappy, Zstd at several levels and gzip, measuring
//! ratio and speed in both directions. Then writes the same table as Parquet with each codec and
//! measures file size and the time to read it all back.
//!
//!     cargo run --release -p m03-representation --example compression

use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Instant;

use datafusion::arrow::array::{ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::parquet::arrow::ArrowWriter;
use datafusion::parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use datafusion::parquet::basic::{Compression, GzipLevel, ZstdLevel};
use datafusion::parquet::file::properties::WriterProperties;
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use std::hint::black_box;

type Codec = (
    &'static str,
    fn(&[u8]) -> Vec<u8>,
    fn(&[u8], usize) -> Vec<u8>,
);

fn codecs() -> Vec<Codec> {
    vec![
        (
            "lz4",
            |d| lz4_flex::block::compress(d),
            |c, n| lz4_flex::block::decompress(c, n).unwrap(),
        ),
        (
            "snappy",
            |d| snap::raw::Encoder::new().compress_vec(d).unwrap(),
            |c, _| snap::raw::Decoder::new().decompress_vec(c).unwrap(),
        ),
        (
            "zstd -1",
            |d| zstd::bulk::compress(d, 1).unwrap(),
            |c, n| zstd::bulk::decompress(c, n).unwrap(),
        ),
        (
            "zstd -3",
            |d| zstd::bulk::compress(d, 3).unwrap(),
            |c, n| zstd::bulk::decompress(c, n).unwrap(),
        ),
        (
            "zstd -9",
            |d| zstd::bulk::compress(d, 9).unwrap(),
            |c, n| zstd::bulk::decompress(c, n).unwrap(),
        ),
        (
            "zstd -19",
            |d| zstd::bulk::compress(d, 19).unwrap(),
            |c, n| zstd::bulk::decompress(c, n).unwrap(),
        ),
        (
            "gzip -6",
            |d| {
                let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(6));
                e.write_all(d).unwrap();
                e.finish().unwrap()
            },
            |c, n| {
                let mut out = Vec::with_capacity(n);
                flate2::read::GzDecoder::new(c)
                    .read_to_end(&mut out)
                    .unwrap();
                out
            },
        ),
    ]
}

fn main() {
    let n = 2_000_000usize;
    let regions = ["AFRICA", "AMERICA", "ASIA", "EUROPE", "MIDDLE EAST"];
    let mut rng = Rng::new(61);
    let mut ts = 1_700_000_000_000i64;
    let times: Vec<i64> = (0..n)
        .map(|_| {
            ts += rng.below(1000) as i64;
            ts
        })
        .collect();
    let region: Vec<&str> = (0..n).map(|_| regions[rng.below(5) as usize]).collect();
    let amount: Vec<f64> = (0..n).map(|_| rng.below(100_000) as f64 / 100.0).collect();

    let csv: Vec<u8> = (0..n)
        .flat_map(|i| format!("{},{},{},{:.2}\n", i, times[i], region[i], amount[i]).into_bytes())
        .collect();
    let ts_bytes: Vec<u8> = times.iter().flat_map(|t| t.to_le_bytes()).collect();
    let region_bytes: Vec<u8> = region.iter().flat_map(|r| r.bytes()).collect();
    let random: Vec<u8> = (0..n * 2)
        .flat_map(|_| rng.next_u64().to_le_bytes())
        .collect();
    let inputs: [(&str, &[u8]); 4] = [
        ("orders as CSV text", &csv),
        ("sorted Int64 timestamps", &ts_bytes),
        ("region strings, run together", &region_bytes),
        ("random bytes", &random),
    ];

    let mut csvout = Vec::new();
    for (name, data) in inputs {
        let mb = data.len() as f64 / 1e6;
        println!("\n== {name}: {mb:.1} MB ==\n");
        println!(
            "{:<10} {:>7} {:>14} {:>16}",
            "codec", "ratio", "compress MB/s", "decompress MB/s"
        );
        for (cname, comp, decomp) in codecs() {
            let t = Instant::now();
            let c = comp(data);
            let mut ct = t.elapsed().as_secs_f64();
            if ct < 1.0 {
                ct = Summary::of(&repeat(0, 3, || comp(black_box(data))))
                    .median
                    .as_secs_f64();
            }
            let dt = Summary::of(&repeat(1, 5, || decomp(black_box(&c), data.len())))
                .median
                .as_secs_f64();
            assert_eq!(decomp(&c, data.len()), data);
            let ratio = data.len() as f64 / c.len() as f64;
            println!(
                "{cname:<10} {ratio:>7.2} {:>14.0} {:>16.0}",
                mb / ct,
                mb / dt
            );
            csvout.push(format!(
                "{name}|{cname},{ratio:.2},{:.0},{:.0}",
                mb / ct,
                mb / dt
            ));
        }
    }
    println!("\ncodec csv: {}", csvout.join(";"));

    println!("\n== The same table as Parquet, by codec: {n} rows ==\n");
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("ts", DataType::Int64, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("amount", DataType::Float64, false),
    ]));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(0..n as i64)),
        Arc::new(Int64Array::from(times.clone())),
        Arc::new(StringArray::from(region.clone())),
        Arc::new(Float64Array::from(amount.clone())),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();
    println!(
        "{:<12} {:>9} {:>10} {:>10}",
        "codec", "file MB", "write ms", "read ms"
    );
    let pq: [(&str, Compression); 6] = [
        ("none", Compression::UNCOMPRESSED),
        ("snappy", Compression::SNAPPY),
        ("lz4_raw", Compression::LZ4_RAW),
        ("zstd(1)", Compression::ZSTD(ZstdLevel::try_new(1).unwrap())),
        ("zstd(9)", Compression::ZSTD(ZstdLevel::try_new(9).unwrap())),
        ("gzip(6)", Compression::GZIP(GzipLevel::try_new(6).unwrap())),
    ];
    let mut pqcsv = Vec::new();
    for (name, codec) in pq {
        let path = dir.join("m03-codec.parquet");
        let props = WriterProperties::builder().set_compression(codec).build();
        let t = Instant::now();
        let mut w = ArrowWriter::try_new(File::create(&path).unwrap(), schema.clone(), Some(props))
            .unwrap();
        w.write(&batch).unwrap();
        w.close().unwrap();
        let wt = t.elapsed().as_secs_f64() * 1e3;
        let size = std::fs::metadata(&path).unwrap().len() as f64 / 1e6;
        let rt = Summary::of(&repeat(1, 5, || {
            let r = ParquetRecordBatchReaderBuilder::try_new(File::open(&path).unwrap())
                .unwrap()
                .build()
                .unwrap();
            r.map(|b| b.unwrap().num_rows()).sum::<usize>()
        }))
        .median
        .as_secs_f64()
            * 1e3;
        println!("{name:<12} {size:>9.2} {wt:>10.0} {rt:>10.1}");
        pqcsv.push(format!("{name},{size:.2},{wt:.0},{rt:.1}"));
        std::fs::remove_file(&path).unwrap();
    }
    println!("\nparquet csv: {}", pqcsv.join(";"));
}
