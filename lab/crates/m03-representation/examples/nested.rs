//! Article: "Arrow nested types".
//! Prints the buffers behind list, large list, struct, map and union arrays, then measures three
//! ways to sum the elements of a million lists.
//!
//!     cargo run --release -p m03-representation --example nested

use std::sync::Arc;

use datafusion::arrow::array::{
    Array, ArrayRef, Int32Array, Int64Array, LargeListArray, ListArray, MapArray, StringArray,
    StructArray, UnionArray,
};
use datafusion::arrow::buffer::ScalarBuffer;
use datafusion::arrow::datatypes::{DataType, Field, Int32Type, Int64Type, UnionFields};
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use std::hint::black_box;

fn ms(f: impl FnMut() -> i64) -> f64 {
    Summary::of(&repeat(1, 7, f)).median.as_secs_f64() * 1e3
}

fn main() {
    println!("== List<Int32>: [[1, 2], [], null, [3, 4, 5]] ==\n");
    let l = ListArray::from_iter_primitive::<Int32Type, _, _>(vec![
        Some(vec![Some(1), Some(2)]),
        Some(vec![]),
        None,
        Some(vec![Some(3), Some(4), Some(5)]),
    ]);
    println!("offsets:  {:?}", l.value_offsets());
    println!(
        "validity: {:?}",
        (0..l.len())
            .map(|i| l.is_valid(i) as u8)
            .collect::<Vec<_>>()
    );
    let child = l.values().as_any().downcast_ref::<Int32Array>().unwrap();
    println!("child:    {:?}  (one flat Int32 array)", child.values());
    let ll = LargeListArray::from_iter_primitive::<Int32Type, _, _>(vec![Some(vec![Some(1)])]);
    println!(
        "offset width: List {} bytes, LargeList {} bytes",
        l.to_data().buffers()[0].len() / (l.len() + 1),
        ll.to_data().buffers()[0].len() / (ll.len() + 1)
    );

    println!("\n== Struct<name: Utf8, qty: Int32> ==\n");
    let s = StructArray::from(vec![
        (
            Arc::new(Field::new("name", DataType::Utf8, false)),
            Arc::new(StringArray::from(vec!["bolt", "nut", "gear"])) as ArrayRef,
        ),
        (
            Arc::new(Field::new("qty", DataType::Int32, false)),
            Arc::new(Int32Array::from(vec![10, 250, 3])) as ArrayRef,
        ),
    ]);
    for (f, c) in s.fields().iter().zip(s.columns()) {
        println!(
            "child {:<4} {:?}: {} values, own buffers",
            f.name(),
            c.data_type(),
            c.len()
        );
    }
    let qty = s.column_by_name("qty").unwrap();
    println!(
        "projecting qty shares its buffer: {}",
        qty.to_data().buffers()[0].as_ptr() == s.column(1).to_data().buffers()[0].as_ptr()
    );

    println!("\n== Map<Utf8, Int32>: [{{a: 1, b: 2}}, {{}}, {{c: 3}}] ==\n");
    let m = MapArray::new_from_strings(
        ["a", "b", "c"].into_iter(),
        &Int32Array::from(vec![1, 2, 3]),
        &[0, 2, 2, 3],
    )
    .unwrap();
    println!("offsets: {:?}", m.value_offsets());
    println!("entries: {:?}", m.entries().data_type());
    println!(
        "keys:    {:?}",
        m.keys()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .iter()
            .flatten()
            .collect::<Vec<_>>()
    );

    println!("\n== Dense union of Int32 | Utf8: [5, \"x\", 7, \"yz\"] ==\n");
    let fields = UnionFields::try_new(
        vec![0, 1],
        vec![
            Field::new("i", DataType::Int32, false),
            Field::new("s", DataType::Utf8, false),
        ],
    )
    .unwrap();
    let u = UnionArray::try_new(
        fields,
        ScalarBuffer::from(vec![0i8, 1, 0, 1]),
        Some(ScalarBuffer::from(vec![0i32, 0, 1, 1])),
        vec![
            Arc::new(Int32Array::from(vec![5, 7])),
            Arc::new(StringArray::from(vec!["x", "yz"])),
        ],
    )
    .unwrap();
    println!("type ids: {:?}", u.type_ids());
    println!(
        "offsets:  {:?}  (index into the chosen child)",
        u.offsets().unwrap()
    );
    println!(
        "children: Int32 {:?}, Utf8 [\"x\", \"yz\"]",
        Int32Array::from(u.child(0).to_data()).values()
    );

    println!("\n== Summing the elements of 1M lists (average 10 elements) ==\n");
    let mut rng = Rng::new(5);
    let big = ListArray::from_iter_primitive::<Int64Type, _, _>((0..1_000_000).map(|_| {
        let k = rng.below(21) as i64;
        Some((0..k).map(|j| Some(j % 7)).collect::<Vec<_>>())
    }));
    let vals = big
        .values()
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .values()
        .clone();
    let offs = big.value_offsets().to_vec();
    println!("{} elements in one child array", vals.len());

    let flat = ms(|| {
        let o = &offs;
        vals[o[0] as usize..o[o.len() - 1] as usize].iter().sum()
    });
    let windows = ms(|| {
        offs.windows(2)
            .map(|w| vals[w[0] as usize..w[1] as usize].iter().sum::<i64>())
            .sum()
    });
    let per_list = ms(|| {
        (0..big.len())
            .map(|i| {
                let a = big.value(i);
                a.as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .values()
                    .iter()
                    .sum::<i64>()
            })
            .sum()
    });
    let check = (black_box(&big), vals.iter().sum::<i64>());
    println!("one pass over the child array:      {flat:>7.2} ms");
    println!("per list, slicing by offsets:       {windows:>7.2} ms");
    println!("per list, list.value(i) as ArrayRef: {per_list:>6.2} ms");
    println!("(total {})", check.1);
}
