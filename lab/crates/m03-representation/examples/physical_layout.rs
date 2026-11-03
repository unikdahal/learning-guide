//! Article: "Arrow physical layout".
//! Prints the bytes of validity bitmaps, value buffers and offsets; checks alignment and padding;
//! slices at an offset that isn't a multiple of 8; and measures what nulls cost a sum.
//!
//!     cargo run --release -p m03-representation --example physical_layout

use datafusion::arrow::array::{Array, BooleanArray, Int32Array, Int64Array, StringArray};
use datafusion::arrow::buffer::Buffer;
use datafusion::arrow::compute::sum;
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use std::hint::black_box;

fn bits(b: &[u8]) -> String {
    b.iter()
        .map(|x| format!("{x:08b}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn describe(name: &str, b: &Buffer) {
    println!(
        "  {name}: {} bytes used, {} allocated, address mod 64 = {}",
        b.len(),
        b.capacity(),
        b.as_ptr() as usize % 64
    );
}

fn ms(f: impl FnMut() -> Option<i64>) -> f64 {
    Summary::of(&repeat(2, 11, f)).median.as_secs_f64() * 1e3
}

fn main() {
    println!("== A nullable Int32 array ==\n");
    let a = Int32Array::from(vec![
        Some(7),
        None,
        Some(-1),
        Some(42),
        None,
        Some(5),
        Some(6),
        Some(8),
        Some(9),
        None,
    ]);
    let d = a.to_data();
    let nulls = a.nulls().unwrap();
    println!("values (logical): {:?}", a.iter().collect::<Vec<_>>());
    println!(
        "validity bytes:   {}   (bit i of byte i/8 is value i; read each byte right to left)",
        bits(nulls.buffer().as_slice())
    );
    let raw: Vec<i32> = d.buffers()[0]
        .as_slice()
        .chunks(4)
        .map(|c| i32::from_le_bytes(c.try_into().unwrap()))
        .collect();
    println!("value buffer:     {raw:?}   (null slots hold whatever the builder wrote)");
    describe("validity", nulls.buffer());
    describe("values", &d.buffers()[0]);
    println!("  null_count stored with the array: {}", a.null_count());

    println!("\n== Booleans are bits too ==\n");
    let b = BooleanArray::from(vec![
        true, false, true, true, false, false, false, true, true, true,
    ]);
    println!(
        "10 booleans: values buffer {} bytes = {}",
        b.values().inner().len(),
        bits(b.values().inner().as_slice())
    );

    println!("\n== A string array ==\n");
    let s = StringArray::from(vec![
        Some("join"),
        Some(""),
        None,
        Some("sort"),
        Some("hash"),
    ]);
    let sd = s.to_data();
    println!("offsets: {:?}", s.value_offsets());
    println!(
        "data:    {:?}",
        std::str::from_utf8(s.value_data()).unwrap()
    );
    describe("offsets", &sd.buffers()[0]);
    describe("data", &sd.buffers()[1]);
    println!(
        "  empty string \"\" and null both have zero-length ranges; only the bitmap tells them apart"
    );

    println!("\n== Slicing at offset 3 ==\n");
    let sl = a.slice(3, 5);
    let sn = sl.nulls().unwrap();
    println!("slice(3, 5) = {:?}", sl.iter().collect::<Vec<_>>());
    println!(
        "  validity: same bytes, bit offset {} (shared: {})",
        sn.offset(),
        sn.buffer().as_ptr() == nulls.buffer().as_ptr()
    );
    println!(
        "  values start {} bytes into the original buffer",
        sl.values().inner().as_ptr() as usize - a.values().inner().as_ptr() as usize
    );
    println!(
        "  null_count of the slice: {} (recounted from the bits)",
        sl.null_count()
    );

    println!("\n== What nulls cost: sum of 10M Int64 ==\n");
    let n = 10_000_000usize;
    let mut csv = Vec::new();
    for pct in [0u64, 1, 10, 50] {
        let mut rng = Rng::new(9);
        let arr: Int64Array = (0..n)
            .map(|i| {
                if pct > 0 && rng.below(100) < pct {
                    None
                } else {
                    Some(i as i64 % 1000)
                }
            })
            .collect();
        let has_bitmap = arr.nulls().is_some();
        let bytes = arr.get_array_memory_size();
        let t = ms(|| sum(black_box(&arr)));
        println!(
            "{pct:>3}% nulls: {t:>6.2} ms   bitmap: {has_bitmap:<5}  memory {:>6.2} MB",
            bytes as f64 / 1e6
        );
        csv.push(format!("{pct},{t:.2}"));
    }
    println!("\nsum csv: {}", csv.join(";"));
}
