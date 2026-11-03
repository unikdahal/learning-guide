//! Article: "Dictionary encoding in storage".
//! Writes string columns of increasing cardinality to Parquet with dictionary encoding on and off,
//! shows each column chunk's dictionary page and the page where the writer falls back to PLAIN,
//! then reads a low-cardinality column back as plain strings and as an Arrow dictionary.
//!
//!     cargo run --release -p m03-representation --example parquet_dict

use std::fs::File;
use std::sync::Arc;

use datafusion::arrow::array::{Array, ArrayRef, AsArray, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::parquet::arrow::ArrowWriter;
use datafusion::parquet::arrow::arrow_reader::{
    ArrowReaderOptions, ParquetRecordBatchReaderBuilder,
};
use datafusion::parquet::basic::{Encoding, PageType};
use datafusion::parquet::file::properties::WriterProperties;
use datafusion::parquet::file::reader::FileReader;
use datafusion::parquet::file::serialized_reader::SerializedFileReader;
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};

fn main() {
    let n = 2_000_000usize;
    let mut rng = Rng::new(71);
    let cards = [5usize, 1_000, 100_000, 2_000_000];
    let fields: Vec<Field> = cards
        .iter()
        .map(|c| Field::new(format!("card_{c}"), DataType::Utf8, false))
        .collect();
    let schema = Arc::new(Schema::new(fields));
    let cols: Vec<ArrayRef> = cards
        .iter()
        .map(|&c| {
            Arc::new(StringArray::from_iter_values((0..n).map(|i| {
                let k = if c == n {
                    i
                } else {
                    rng.below(c as u64) as usize
                };
                format!("customer-{k:07}")
            }))) as ArrayRef
        })
        .collect();
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();

    let mut sizes = Vec::new();
    for dict in [true, false] {
        let path = dir.join(format!("m03-dict-{dict}.parquet"));
        let props = WriterProperties::builder()
            .set_dictionary_enabled(dict)
            .set_max_row_group_row_count(Some(n))
            .build();
        let mut w = ArrowWriter::try_new(File::create(&path).unwrap(), schema.clone(), Some(props))
            .unwrap();
        w.write(&batch).unwrap();
        w.close().unwrap();
        let r = SerializedFileReader::new(File::open(&path).unwrap()).unwrap();
        let rg = r.metadata().row_group(0);
        sizes.push(
            rg.columns()
                .iter()
                .map(|c| c.compressed_size())
                .collect::<Vec<_>>(),
        );
    }

    println!("{n} strings of 16 bytes per column, one row group, no compression\n");
    println!(
        "{:<15} {:>12} {:>12} {:>8}",
        "column", "dict on MB", "dict off MB", "ratio"
    );
    for (i, c) in cards.iter().enumerate() {
        let (a, b) = (sizes[0][i] as f64 / 1e6, sizes[1][i] as f64 / 1e6);
        println!("card_{c:<10} {a:>12.2} {b:>12.2} {:>7.1}x", b / a);
    }

    println!("\n== Pages in each column chunk, dictionary on ==");
    let path = dir.join("m03-dict-true.parquet");
    let r = SerializedFileReader::new(File::open(&path).unwrap()).unwrap();
    let rg = r.get_row_group(0).unwrap();
    for (i, c) in cards.iter().enumerate() {
        let meta = rg.metadata().column(i);
        println!(
            "\ncard_{c}: encodings {:?}",
            meta.encodings().collect::<Vec<_>>()
        );
        let mut pages = rg.get_column_page_reader(i).unwrap();
        let (mut dict_pages, mut rle, mut plain, mut first_plain) = (0, 0, 0, None);
        let mut dict_bytes = 0;
        let mut k = 0;
        while let Some(p) = pages.get_next_page().unwrap() {
            match (p.page_type(), p.encoding()) {
                (PageType::DICTIONARY_PAGE, _) => {
                    dict_pages += 1;
                    dict_bytes = p.buffer().len();
                    println!(
                        "  dictionary page: {} distinct values, {} bytes",
                        p.num_values(),
                        dict_bytes
                    );
                }
                (_, Encoding::RLE_DICTIONARY | Encoding::PLAIN_DICTIONARY) => {
                    rle += p.num_values();
                }
                (_, Encoding::PLAIN) => {
                    if first_plain.is_none() {
                        first_plain = Some(k);
                    }
                    plain += p.num_values();
                }
                (t, e) => println!("  other page {t:?} {e:?}"),
            }
            k += 1;
        }
        println!(
            "  values in dictionary-coded pages: {rle}, in PLAIN pages: {plain}{}",
            first_plain
                .map(|p| format!(" (fallback from page {p})"))
                .unwrap_or_default()
        );
        let _ = (dict_pages, dict_bytes);
    }

    println!("\n== Reading card_1000 back ==\n");
    let plain_read = || {
        let b = ParquetRecordBatchReaderBuilder::try_new(File::open(&path).unwrap()).unwrap();
        let mask = datafusion::parquet::arrow::ProjectionMask::roots(b.parquet_schema(), [1]);
        let r = b.with_projection(mask).build().unwrap();
        r.map(|b| b.unwrap().column(0).get_array_memory_size())
            .sum::<usize>()
    };
    let dict_schema = Arc::new(Schema::new(
        cards
            .iter()
            .map(|c| {
                Field::new(
                    format!("card_{c}"),
                    DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
                    false,
                )
            })
            .collect::<Vec<_>>(),
    ));
    let dict_read = || {
        let opts = ArrowReaderOptions::new().with_schema(dict_schema.clone());
        let b =
            ParquetRecordBatchReaderBuilder::try_new_with_options(File::open(&path).unwrap(), opts)
                .unwrap();
        let mask = datafusion::parquet::arrow::ProjectionMask::roots(b.parquet_schema(), [1]);
        let r = b.with_projection(mask).build().unwrap();
        // Batches share one dictionary, so count each batch's keys and the dictionary once.
        let mut dict_bytes = 0;
        let keys: usize = r
            .map(|b| {
                let b = b.unwrap();
                let d = b.column(0).as_any_dictionary();
                dict_bytes = d.values().get_array_memory_size();
                d.keys().get_array_memory_size()
            })
            .sum();
        keys + dict_bytes
    };
    let tp = Summary::of(&repeat(1, 5, plain_read)).median.as_secs_f64() * 1e3;
    let td = Summary::of(&repeat(1, 5, dict_read)).median.as_secs_f64() * 1e3;
    println!(
        "as Utf8 (decoded):          {tp:>6.1} ms, {:>6.1} MB in memory",
        plain_read() as f64 / 1e6
    );
    println!(
        "as Dictionary(Int32, Utf8): {td:>6.1} ms, {:>6.1} MB in memory",
        dict_read() as f64 / 1e6
    );
}
