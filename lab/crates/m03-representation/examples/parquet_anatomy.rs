//! Article: "Anatomy of an Apache Parquet file".
//! Writes a small orders file with arrow-rs's Parquet writer, then takes it apart: the magic
//! bytes, the footer length, the row groups and column chunks the footer describes, and the pages
//! inside one column chunk. Ends with a byte map of the whole file for the article's visual.
//!
//!     cargo run --release -p m03-representation --example parquet_anatomy

use std::fs::File;
use std::sync::Arc;

use datafusion::arrow::array::{ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::parquet::arrow::ArrowWriter;
use datafusion::parquet::basic::{Compression, ZstdLevel};
use datafusion::parquet::file::properties::WriterProperties;
use datafusion::parquet::file::reader::FileReader;
use datafusion::parquet::file::serialized_reader::{ReadOptionsBuilder, SerializedFileReader};
use datafusion::parquet::file::statistics::Statistics;
use labkit::rng::Rng;

fn stats(s: Option<&Statistics>) -> String {
    match s {
        Some(Statistics::Int64(v)) => format!(
            "min {:?} max {:?} nulls {:?}",
            v.min_opt(),
            v.max_opt(),
            v.null_count_opt()
        ),
        Some(Statistics::Double(v)) => format!(
            "min {:?} max {:?} nulls {:?}",
            v.min_opt(),
            v.max_opt(),
            v.null_count_opt()
        ),
        Some(Statistics::ByteArray(v)) => format!(
            "min {:?} max {:?} nulls {:?}",
            v.min_opt()
                .map(|b| String::from_utf8_lossy(b.data()).to_string()),
            v.max_opt()
                .map(|b| String::from_utf8_lossy(b.data()).to_string()),
            v.null_count_opt()
        ),
        Some(other) => format!("{other:?}"),
        None => "none".into(),
    }
}

fn main() {
    let n = 300_000usize;
    let regions = ["AFRICA", "AMERICA", "ASIA", "EUROPE", "MIDDLE EAST"];
    let mut rng = Rng::new(51);
    let schema = Arc::new(Schema::new(vec![
        Field::new("order_id", DataType::Int64, false),
        Field::new("region", DataType::Utf8, false),
        Field::new("amount", DataType::Float64, false),
    ]));
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from_iter_values(0..n as i64)),
        Arc::new(StringArray::from_iter_values(
            (0..n).map(|_| regions[rng.below(5) as usize]),
        )),
        Arc::new(Float64Array::from_iter_values(
            (0..n).map(|_| (rng.below(100_000) as f64) / 100.0),
        )),
    ];
    let batch = RecordBatch::try_new(schema.clone(), cols).unwrap();

    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("m03-orders.parquet");
    let props = WriterProperties::builder()
        .set_max_row_group_row_count(Some(100_000))
        .set_compression(Compression::ZSTD(ZstdLevel::try_new(1).unwrap()))
        .build();
    let mut w = ArrowWriter::try_new(File::create(&path).unwrap(), schema, Some(props)).unwrap();
    w.write(&batch).unwrap();
    w.close().unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let len = bytes.len();
    println!("== The file: {len} bytes, {n} rows ==\n");
    println!("first 4 bytes: {:?}", String::from_utf8_lossy(&bytes[..4]));
    let footer_len = u32::from_le_bytes(bytes[len - 8..len - 4].try_into().unwrap()) as usize;
    println!(
        "last 8 bytes:  footer length {footer_len} (little-endian u32), then {:?}",
        String::from_utf8_lossy(&bytes[len - 4..])
    );
    println!(
        "footer (Thrift-encoded metadata) at bytes {}..{}",
        len - 8 - footer_len,
        len - 8
    );

    let reader = SerializedFileReader::new_with_options(
        File::open(&path).unwrap(),
        ReadOptionsBuilder::new().with_page_index().build(),
    )
    .unwrap();
    let md = reader.metadata();
    let fm = md.file_metadata();
    println!(
        "\nfile metadata: version {}, {} rows, created by {:?}",
        fm.version(),
        fm.num_rows(),
        fm.created_by().unwrap_or("?")
    );
    println!(
        "schema: {}",
        fm.schema_descr()
            .columns()
            .iter()
            .map(|c| format!("{} {:?}", c.name(), c.physical_type()))
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!("\n== Row groups and column chunks ==");
    for (g, rg) in md.row_groups().iter().enumerate() {
        println!(
            "\nrow group {g}: {} rows, {} bytes compressed",
            rg.num_rows(),
            rg.compressed_size()
        );
        for c in rg.columns() {
            println!(
                "  {:<9} {:?} encodings {:?}\n            dictionary page at {:?}, data pages at {}, {} -> {} bytes\n            {}",
                c.column_path().string(),
                c.compression(),
                c.encodings().collect::<Vec<_>>(),
                c.dictionary_page_offset(),
                c.data_page_offset(),
                c.uncompressed_size(),
                c.compressed_size(),
                stats(c.statistics())
            );
        }
    }

    println!("\n== Pages inside row group 0, column region ==\n");
    let rg = reader.get_row_group(0).unwrap();
    let mut pages = rg.get_column_page_reader(1).unwrap();
    while let Some(p) = pages.get_next_page().unwrap() {
        println!(
            "{:?}: {} values, encoding {:?}, {} bytes after decompression",
            p.page_type(),
            p.num_values(),
            p.encoding(),
            p.buffer().len()
        );
    }

    println!("\n== Byte map ==\n");
    let mut map = vec![format!("0,4,magic")];
    let oi = md.offset_index().unwrap();
    for (g, rg) in md.row_groups().iter().enumerate() {
        for (c, col) in rg.columns().iter().enumerate() {
            let name = col.column_path().string();
            if let Some(d) = col.dictionary_page_offset() {
                map.push(format!(
                    "{d},{},rg{g} {name} dict",
                    col.data_page_offset() - d
                ));
            }
            for (k, loc) in oi[g][c].page_locations().iter().enumerate() {
                map.push(format!(
                    "{},{},rg{g} {name} page{k}",
                    loc.offset, loc.compressed_page_size
                ));
            }
        }
    }
    let data_end = md
        .row_groups()
        .iter()
        .flat_map(|rg| rg.columns())
        .map(|c| c.dictionary_page_offset().unwrap_or(c.data_page_offset()) + c.compressed_size())
        .max()
        .unwrap();
    map.push(format!(
        "{data_end},{},page indexes",
        len - 8 - footer_len - data_end as usize
    ));
    map.push(format!("{},{footer_len},footer", len - 8 - footer_len));
    map.push(format!("{},4,footer length", len - 8));
    map.push(format!("{},4,magic", len - 4));
    println!("map csv: {}", map.join(";"));
}
