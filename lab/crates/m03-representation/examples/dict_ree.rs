//! Article: "Dictionary and run-end encoded arrays".
//! A low-cardinality string column stored plain and dictionary encoded, and a sorted column stored
//! plain and run-end encoded: their sizes, a filter, a count by value, decoding, and random access.
//!
//!     cargo run --release -p m03-representation --example dict_ree

use std::collections::HashMap;

use datafusion::arrow::array::{
    Array, DictionaryArray, Int8Array, Int32Array, Int64Array, RunArray, Scalar, StringArray,
};
use datafusion::arrow::compute::kernels::cmp::eq;
use datafusion::arrow::compute::{cast, sum};
use datafusion::arrow::datatypes::{DataType, Int8Type, Int32Type};
use labkit::rng::Rng;
use labkit::timing::{Summary, repeat};
use std::hint::black_box;

fn ms<R>(f: impl FnMut() -> R) -> f64 {
    Summary::of(&repeat(1, 7, f)).median.as_secs_f64() * 1e3
}

fn mb(a: &dyn Array) -> f64 {
    a.get_array_memory_size() as f64 / 1e6
}

fn main() {
    let n = 10_000_000usize;
    let regions = ["AFRICA", "AMERICA", "ASIA", "EUROPE", "MIDDLE EAST"];
    let mut rng = Rng::new(11);
    let picks: Vec<&str> = (0..n).map(|_| regions[rng.below(5) as usize]).collect();

    println!("== A region column, {n} rows, 5 distinct values ==\n");
    let plain = StringArray::from(picks.clone());
    let dict32: DictionaryArray<Int32Type> = picks.iter().copied().collect();
    let dict8: DictionaryArray<Int8Type> = picks.iter().copied().collect();
    println!("plain Utf8:            {:>7.2} MB", mb(&plain));
    println!("dictionary, Int32 keys:{:>7.2} MB", mb(&dict32));
    println!("dictionary, Int8 keys: {:>7.2} MB", mb(&dict8));
    let small = dict32.slice(0, 6);
    let small = small
        .as_any()
        .downcast_ref::<DictionaryArray<Int32Type>>()
        .unwrap();
    println!(
        "first 6 keys {:?} index into values {:?}",
        small.keys().values(),
        small
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .iter()
            .flatten()
            .collect::<Vec<_>>()
    );

    println!("\n== Filter: region = 'ASIA' ==\n");
    let target = Scalar::new(StringArray::from(vec!["ASIA"]));
    let t_plain = ms(|| eq(black_box(&plain), &target).unwrap());
    let t_dict = ms(|| eq(black_box(&dict32), &target).unwrap());
    let t_codes = ms(|| {
        // Look the constant up in the dictionary once, then compare integer codes.
        let values = dict32
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let code = values.iter().position(|v| v == Some("ASIA")).unwrap() as i32;
        eq(dict32.keys(), &Scalar::new(Int32Array::from(vec![code]))).unwrap()
    });
    let t_codes8 = ms(|| {
        let values = dict8
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        let code = values.iter().position(|v| v == Some("ASIA")).unwrap() as i8;
        eq(dict8.keys(), &Scalar::new(Int8Array::from(vec![code]))).unwrap()
    });
    let hits = eq(&plain, &target).unwrap().true_count();
    println!("plain strings, eq kernel:          {t_plain:>6.2} ms");
    println!("dictionary, eq kernel:             {t_dict:>6.2} ms");
    println!("dictionary, compare Int32 codes:   {t_codes:>6.2} ms");
    println!("dictionary, compare Int8 codes:    {t_codes8:>6.2} ms");
    println!("({hits} matches)");

    println!("\n== Count rows per region ==\n");
    let t_hash = ms(|| {
        let mut m: HashMap<&str, u64> = HashMap::new();
        for v in plain.iter().flatten() {
            *m.entry(v).or_default() += 1;
        }
        m.len()
    });
    let t_keys = ms(|| {
        let mut c = [0u64; 5];
        for &k in dict32.keys().values() {
            c[k as usize] += 1;
        }
        c[0]
    });
    println!("plain, hash map on strings:   {t_hash:>6.2} ms");
    println!("dictionary, array of counters:{t_keys:>6.2} ms");

    println!("\n== Decoding the dictionary back to plain strings ==\n");
    let t_cast = ms(|| cast(black_box(&dict32), &DataType::Utf8).unwrap());
    println!("cast Dictionary(Int32, Utf8) -> Utf8: {t_cast:.2} ms");

    println!("\n== A sorted date column, {n} rows, 1000 distinct days ==\n");
    let per = n / 1000;
    let days = Int64Array::from_iter_values((0..n).map(|i| 19_000 + (i / per) as i64));
    let run_ends = Int32Array::from_iter_values((1..=1000).map(|r| (r * per) as i32));
    let run_values = Int64Array::from_iter_values((0..1000).map(|r| 19_000 + r as i64));
    let ree = RunArray::<Int32Type>::try_new(&run_ends, &run_values).unwrap();
    println!("plain Int64:      {:>9.3} MB", mb(&days));
    println!(
        "run-end encoded:  {:>9.3} MB  ({} runs, logical length {})",
        mb(&ree),
        ree.run_ends().values().len(),
        ree.len()
    );
    let tiny = RunArray::<Int32Type>::try_new(
        &Int32Array::from(vec![3, 4, 7]),
        &Int64Array::from(vec![10, 20, 30]),
    )
    .unwrap();
    println!(
        "example: run_ends {:?}, values {:?} = [10, 10, 10, 20, 30, 30, 30]",
        tiny.run_ends().values(),
        tiny.values()
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .values()
    );

    let t_sum_plain = ms(|| sum(black_box(&days)));
    let t_sum_ree = ms(|| {
        let ends = ree.run_ends().values();
        let vals = run_values.values();
        let mut prev = 0i64;
        let mut s = 0i64;
        for (e, v) in ends.iter().zip(vals.iter()) {
            s += v * (*e as i64 - prev);
            prev = *e as i64;
        }
        s
    });
    println!("\nsum, plain:                       {t_sum_plain:>8.3} ms");
    println!("sum, run-end encoded (per run):   {t_sum_ree:>8.3} ms");

    let mut rng = Rng::new(12);
    let probes: Vec<usize> = (0..1_000_000)
        .map(|_| rng.below(n as u64) as usize)
        .collect();
    let t_get_plain = ms(|| probes.iter().map(|&i| days.value(i)).sum::<i64>());
    let t_get_ree = ms(|| {
        probes
            .iter()
            .map(|&i| run_values.value(ree.get_physical_index(i)))
            .sum::<i64>()
    });
    let t_decode = ms(|| cast(black_box(&ree), &DataType::Int64).unwrap());
    println!("1M random reads, plain:           {t_get_plain:>8.2} ms");
    println!(
        "1M random reads, run-end encoded: {t_get_ree:>8.2} ms  (binary search over run ends)"
    );
    println!("decode run-end encoded -> Int64:  {t_decode:>8.2} ms");
}
