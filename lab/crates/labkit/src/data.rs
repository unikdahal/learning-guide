//! Deterministic test data for Module 00.
//!
//! Sizes follow TPC-H at scale factor 1: 150,000 customers and 1,500,000 orders. The schema is
//! simplified so the running example in the guide stays readable:
//!
//! - `customer(id, name, segment, region)`: one customer in five is in the `retail` segment.
//! - `orders(id, cust_id, order_date, status, amount)`: `amount` is `Decimal128(12, 2)`.
//! - `trips(trip_id, hour, distance_km, fare)`: a small CSV for the embedding example.

use std::error::Error;
use std::fs::{File, create_dir_all, remove_dir_all, rename};
use std::path::Path;
use std::sync::Arc;

use datafusion::arrow::array::{
    ArrayRef, Date32Array, Decimal128Array, Float64Array, Int64Array, StringArray,
};
use datafusion::arrow::csv::WriterBuilder;
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::parquet::arrow::ArrowWriter;

use crate::rng::Rng;

pub const SEGMENTS: [&str; 5] = [
    "retail",
    "wholesale",
    "government",
    "education",
    "nonprofit",
];
pub const REGIONS: [&str; 5] = ["AFRICA", "AMERICA", "ASIA", "EUROPE", "MIDDLE EAST"];

/// Version 2 fixes the trips fare formula. Customer/orders rows are unchanged from v1.
pub const GENERATOR_VERSION: u32 = 2;
/// Rows per batch keep generation memory bounded.
const CHUNK: usize = 65_536;

pub fn row_counts(scale: f64) -> Result<(usize, usize)> {
    let customers = (150_000.0 * scale).floor();
    let orders = (1_500_000.0 * scale).floor();
    let limit = if usize::BITS < 64 {
        usize::MAX as f64
    } else {
        i64::MAX as f64
    };
    if !scale.is_finite()
        || scale <= 0.0
        || customers < 1.0
        || !orders.is_finite()
        || customers >= limit
        || orders >= limit
    {
        return Err("scale must be finite and positive, produce at least one customer, and fit integer row IDs".into());
    }
    Ok((customers as usize, orders as usize))
}

fn fare_for_distance(distance: f64) -> f64 {
    ((2.5 + distance * 1.8) * 100.0).round() / 100.0
}

type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn customer_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
        Field::new("segment", DataType::Utf8, false),
        Field::new("region", DataType::Utf8, false),
    ]))
}

pub fn orders_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("cust_id", DataType::Int64, false),
        Field::new("order_date", DataType::Date32, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("amount", DataType::Decimal128(12, 2), false),
    ]))
}

pub fn trips_schema() -> SchemaRef {
    Arc::new(Schema::new(vec![
        Field::new("trip_id", DataType::Int64, false),
        Field::new("hour", DataType::Int64, false),
        Field::new("distance_km", DataType::Float64, false),
        Field::new("fare", DataType::Float64, false),
    ]))
}

/// Customers `start..end` (ids are 1-based).
fn customer_batch(rng: &mut Rng, start: usize, end: usize) -> Result<RecordBatch> {
    let ids: Vec<i64> = (start..end).map(|i| i as i64 + 1).collect();
    let names: Vec<String> = ids.iter().map(|id| format!("Customer#{id:09}")).collect();
    let segments: Vec<&str> = ids
        .iter()
        .map(|_| SEGMENTS[rng.below(5) as usize])
        .collect();
    let regions: Vec<&str> = ids.iter().map(|_| REGIONS[rng.below(5) as usize]).collect();
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(ids)),
        Arc::new(StringArray::from(names)),
        Arc::new(StringArray::from(segments)),
        Arc::new(StringArray::from(regions)),
    ];
    Ok(RecordBatch::try_new(customer_schema(), cols)?)
}

/// Orders `start..end`, referencing customers `1..=customers`.
fn orders_batch(rng: &mut Rng, start: usize, end: usize, customers: usize) -> Result<RecordBatch> {
    // 2020-01-01 is day 18,262 since the Unix epoch; spread orders over five years.
    const DAY0: i32 = 18_262;
    let n = end - start;
    let ids: Vec<i64> = (start..end).map(|i| i as i64 + 1).collect();
    let cust: Vec<i64> = (0..n)
        .map(|_| rng.below(customers as u64) as i64 + 1)
        .collect();
    let dates: Vec<i32> = (0..n).map(|_| DAY0 + rng.below(5 * 365) as i32).collect();
    let status: Vec<&str> = (0..n)
        .map(|_| ["F", "O", "P"][rng.below(3) as usize])
        .collect();
    // Amounts between 1.00 and 5,000.00, stored as cents.
    let amounts: Vec<i128> = (0..n).map(|_| 100 + rng.below(499_901) as i128).collect();
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(ids)),
        Arc::new(Int64Array::from(cust)),
        Arc::new(Date32Array::from(dates)),
        Arc::new(StringArray::from(status)),
        Arc::new(Decimal128Array::from(amounts).with_precision_and_scale(12, 2)?),
    ];
    Ok(RecordBatch::try_new(orders_schema(), cols)?)
}

fn trips_batch(rng: &mut Rng, start: usize, end: usize) -> Result<RecordBatch> {
    let n = end - start;
    let ids: Vec<i64> = (start..end).map(|i| i as i64 + 1).collect();
    // Two daily peaks: more trips around 8:00 and 18:00.
    let hours: Vec<i64> = (0..n)
        .map(|_| {
            let r = rng.unit();
            if r < 0.25 {
                7 + rng.below(3) as i64
            } else if r < 0.5 {
                17 + rng.below(3) as i64
            } else {
                rng.below(24) as i64
            }
        })
        .collect();
    let dist: Vec<f64> = (0..n).map(|_| 0.5 + rng.unit() * 20.0).collect();
    let fare: Vec<f64> = dist.iter().map(|d| fare_for_distance(*d)).collect();
    let cols: Vec<ArrayRef> = vec![
        Arc::new(Int64Array::from(ids)),
        Arc::new(Int64Array::from(hours)),
        Arc::new(Float64Array::from(dist)),
        Arc::new(Float64Array::from(fare)),
    ];
    Ok(RecordBatch::try_new(trips_schema(), cols)?)
}

fn write_parquet<F>(path: &Path, schema: SchemaRef, rows: usize, mut make: F) -> Result<()>
where
    F: FnMut(usize, usize) -> Result<RecordBatch>,
{
    let mut writer = ArrowWriter::try_new(File::create(path)?, schema, None)?;
    for start in (0..rows).step_by(CHUNK) {
        writer.write(&make(start, (start + CHUNK).min(rows))?)?;
    }
    writer.close()?;
    Ok(())
}

/// Writes `customer.parquet`, `orders.parquet` and `trips.csv` into `dir`.
/// `scale` multiplies the TPC-H scale-factor-1 row counts.
pub fn generate(dir: &Path, scale: f64) -> Result<()> {
    let (customers, orders) = row_counts(scale)?; // Validate before touching existing files.
    create_dir_all(dir)?;
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    let staging = dir.join(format!(".generate-{}-{nonce}", std::process::id()));
    create_dir_all(&staging)?;
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(staging.clone());
    generate_into(&staging, customers, orders)?;
    std::fs::write(
        staging.join("generator.json"),
        format!(
            "{{\"version\":{GENERATOR_VERSION},\"scale\":{scale},\"customers\":{customers},\"orders\":{orders},\"trips\":200000,\"chunk\":{CHUNK}}}\n"
        ),
    )?;
    // Publish only after every writer has finished. Writer errors leave existing files intact.
    // Renames are per file; an I/O failure during publication can still require a rerun.
    for name in [
        "customer.parquet",
        "orders.parquet",
        "trips.csv",
        "generator.json",
    ] {
        rename(staging.join(name), dir.join(name))?;
    }
    Ok(())
}

fn generate_into(dir: &Path, customers: usize, orders: usize) -> Result<()> {
    let mut rng = Rng::new(42);
    write_parquet(
        &dir.join("customer.parquet"),
        customer_schema(),
        customers,
        |s, e| customer_batch(&mut rng, s, e),
    )?;
    let mut rng = Rng::new(43);
    write_parquet(
        &dir.join("orders.parquet"),
        orders_schema(),
        orders,
        |s, e| orders_batch(&mut rng, s, e, customers),
    )?;

    let mut rng = Rng::new(44);
    let mut csv = WriterBuilder::new()
        .with_header(true)
        .build(File::create(dir.join("trips.csv"))?);
    let trips = 200_000;
    for start in (0..trips).step_by(CHUNK) {
        csv.write(&trips_batch(&mut rng, start, (start + CHUNK).min(trips))?)?;
    }
    Ok(())
}

/// Absolute path of a file in `lab/data/`, so examples work from any working directory.
pub fn data_file(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../data")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

/// Checks the scale-1 revenue map outside the measured interval, ignoring row order.
pub fn assert_revenue(batches: &[RecordBatch]) {
    use datafusion::arrow::util::display::array_value_to_string;
    use std::collections::BTreeMap;
    let mut actual = BTreeMap::new();
    for batch in batches {
        for i in 0..batch.num_rows() {
            let region = array_value_to_string(batch.column(0), i).unwrap();
            let value = array_value_to_string(batch.column(1), i).unwrap();
            assert!(
                actual.insert(region, value).is_none(),
                "duplicate revenue group"
            );
        }
    }
    let expected = [
        ("AFRICA", "146087670.17"),
        ("AMERICA", "149691110.90"),
        ("ASIA", "150049799.10"),
        ("EUROPE", "150911354.33"),
        ("MIDDLE EAST", "149423247.54"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .collect();
    assert_eq!(
        actual, expected,
        "revenue evidence requires generator scale 1; wrong keys or decimal sums"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scale_bounds_protect_foreign_keys() {
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e300, 0.000001] {
            assert!(row_counts(scale).is_err());
        }
        assert_eq!(row_counts(1.0).unwrap(), (150_000, 1_500_000));
        assert_eq!(row_counts(0.00001).unwrap(), (1, 15));
    }
    #[test]
    fn invalid_scale_does_not_touch_the_output() {
        let path = std::env::temp_dir().join(format!("qe-invalid-scale-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        assert!(generate(&path, f64::NAN).is_err());
        assert!(!path.exists());
    }
    #[test]
    fn logical_rows_and_fares_are_deterministic() {
        assert_eq!(fare_for_distance(1.0), 4.30);
        assert_eq!(fare_for_distance(10.0), 20.50);
        let a = customer_batch(&mut Rng::new(42), 0, 8).unwrap();
        let b = customer_batch(&mut Rng::new(42), 0, 8).unwrap();
        assert_eq!(a, b);
        let orders = orders_batch(&mut Rng::new(43), 0, 15, 1).unwrap();
        let keys = orders
            .column(1)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap();
        assert!(keys.values().iter().all(|key| *key == 1));
    }
}
