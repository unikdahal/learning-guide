//! Article: "Apache Arrow: the columnar in-memory standard".
//! Builds Arrow arrays and prints the buffers behind them, slices one without copying, then
//! writes the same 10M rows as an Arrow IPC file and as CSV for python/read_arrow.py to read.
//!
//!     cargo run --release -p m03-representation --example arrow_ipc
//!     python3 crates/m03-representation/python/read_arrow.py

use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

use datafusion::arrow::array::{
    Array, ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray,
};
use datafusion::arrow::csv::Writer as CsvWriter;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::ipc::reader::FileReader;
use datafusion::arrow::ipc::writer::FileWriter;

fn show_buffers(name: &str, a: &dyn Array) {
    let d = a.to_data();
    println!(
        "{name}: {} values, type {:?}, offset {}",
        d.len(),
        d.data_type(),
        d.offset()
    );
    match d.nulls() {
        Some(n) => {
            let bits: String = (0..d.len())
                .map(|i| if n.is_valid(i) { '1' } else { '0' })
                .collect();
            println!("  validity bitmap: {bits}  ({} null)", n.null_count());
        }
        None => println!("  validity bitmap: none (no nulls)"),
    }
    for (i, b) in d.buffers().iter().enumerate() {
        println!("  buffer {i}: {} bytes at {:p}", b.len(), b.as_ptr());
    }
}

fn main() {
    println!("== The buffers behind three small arrays ==\n");
    let id = Int64Array::from(vec![1, 2, 3, 4, 5]);
    let name = StringArray::from(vec![
        Some("Asha"),
        None,
        Some("Chen"),
        Some("Dara"),
        Some("Emeka"),
    ]);
    let amount = Float64Array::from(vec![250.0, 40.0, 90.0, 610.0, 75.0]);
    show_buffers("id", &id);
    show_buffers("name", &name);
    let offsets: Vec<i32> = name.value_offsets().to_vec();
    let bytes = std::str::from_utf8(name.value_data()).unwrap();
    println!("  name offsets: {offsets:?}");
    println!("  name bytes:   {bytes:?}");
    show_buffers("amount", &amount);

    println!("\n== Slicing copies nothing ==\n");
    let sliced = name.slice(2, 3);
    println!(
        "name.slice(2, 3) = {:?}",
        (0..sliced.len())
            .map(|i| sliced.value(i))
            .collect::<Vec<_>>()
    );
    show_buffers("slice", &sliced);
    let same = sliced.to_data().buffers()[1].as_ptr() == name.to_data().buffers()[1].as_ptr();
    println!("  same string bytes as the original array: {same}");

    println!("\n== Writing 10M rows as Arrow IPC and as CSV ==\n");
    let n = 10_000_000i64;
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("value", DataType::Float64, false),
    ]));
    let ids: ArrayRef = Arc::new(Int64Array::from_iter_values(0..n));
    let vals: ArrayRef = Arc::new(Float64Array::from_iter_values(
        (0..n).map(|i| (i % 1000) as f64 * 0.5),
    ));
    let batch = RecordBatch::try_new(schema.clone(), vec![ids, vals]).unwrap();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    std::fs::create_dir_all(&dir).unwrap();

    let t = Instant::now();
    let mut w =
        FileWriter::try_new(File::create(dir.join("m03-demo.arrow")).unwrap(), &schema).unwrap();
    w.write(&batch).unwrap();
    w.finish().unwrap();
    let ipc_s = t.elapsed().as_secs_f64();
    let t = Instant::now();
    let mut w = CsvWriter::new(File::create(dir.join("m03-demo.csv")).unwrap());
    w.write(&batch).unwrap();
    drop(w);
    let csv_s = t.elapsed().as_secs_f64();
    let size = |f: &str| std::fs::metadata(dir.join(f)).unwrap().len() as f64 / 1e6;
    println!(
        "Arrow IPC: {:>7.1} MB, written in {:>6.0} ms",
        size("m03-demo.arrow"),
        ipc_s * 1e3
    );
    println!(
        "CSV:       {:>7.1} MB, written in {:>6.0} ms",
        size("m03-demo.csv"),
        csv_s * 1e3
    );

    let t = Instant::now();
    let reader =
        FileReader::try_new(File::open(dir.join("m03-demo.arrow")).unwrap(), None).unwrap();
    let rows: usize = reader.map(|b| b.unwrap().num_rows()).sum();
    println!(
        "\nRust reads the IPC file back: {rows} rows in {:.0} ms",
        t.elapsed().as_secs_f64() * 1e3
    );
}
