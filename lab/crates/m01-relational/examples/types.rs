//! Article: "Data types and their traps".
//! Integer overflow, decimal precision and scale, float equality, NaN, string comparison and
//! time zones, first in plain Rust, then in DataFusion.
//!
//!     cargo run --release -p m01-relational --example types

use datafusion::prelude::*;
use m01_relational::sql::{exec, show};

#[tokio::main]
async fn main() {
    println!("== Rust: the machine's view ==\n");
    let big: i32 = i32::MAX;
    println!("i32::MAX                  = {big}");
    println!("i32::MAX.checked_add(1)   = {:?}", big.checked_add(1));
    println!("i32::MAX.wrapping_add(1)  = {}", big.wrapping_add(1));
    println!("0.1 + 0.2                 = {:.17}", 0.1_f64 + 0.2);
    println!("0.1 + 0.2 == 0.3          = {}", 0.1_f64 + 0.2 == 0.3);
    let nan = std::hint::black_box(f64::NAN);
    println!("NaN == NaN                = {}", nan.eq(&nan));
    let mut sum32 = 0.0_f32;
    for _ in 0..10_000_000 {
        sum32 += 0.1;
    }
    println!("ten million 0.1s, f32 sum = {sum32}");
    let mut sum64 = 0.0_f64;
    for _ in 0..10_000_000 {
        sum64 += 0.1;
    }
    println!("ten million 0.1s, f64 sum = {sum64}");
    println!();

    let ctx = SessionContext::new();

    println!("== Integers ==\n");
    show(
        &ctx,
        "SELECT CAST(2147483647 AS INT) + CAST(1 AS INT) AS overflow",
    )
    .await;
    show(
        &ctx,
        "SELECT arrow_typeof(CAST(1 AS INT) + CAST(1 AS INT)) AS int_plus_int",
    )
    .await;
    show(
        &ctx,
        "SELECT CAST(9223372036854775807 AS BIGINT) + 1 AS overflow",
    )
    .await;
    show(
        &ctx,
        "SELECT 7 / 2 AS int_div, 7.0 / 2 AS float_div, -7 / 2 AS neg_div, -7 % 2 AS neg_mod",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE raw (s VARCHAR) AS VALUES ('3000000000')",
    )
    .await;
    show(&ctx, "SELECT CAST(s AS INT) AS too_big FROM raw").await;
    exec(
        &ctx,
        "CREATE TABLE sales (qty INT) AS VALUES (2000000000), (2000000000)",
    )
    .await;
    show(
        &ctx,
        "SELECT sum(qty) AS total, arrow_typeof(sum(qty)) AS t FROM sales",
    )
    .await;

    println!("== Decimals ==\n");
    show(&ctx, "SELECT CAST(1.005 AS DECIMAL(10,2)) AS from_float_literal, CAST('1.005' AS DECIMAL(10,2)) AS from_string, arrow_typeof(1.005) AS literal_t").await;
    show(&ctx, "SELECT CAST(123.456 AS DECIMAL(5,2)) AS rounds").await;
    exec(&ctx, "CREATE TABLE raw_d (s VARCHAR) AS VALUES ('12345.6')").await;
    show(&ctx, "SELECT CAST(s AS DECIMAL(5,2)) AS too_big FROM raw_d").await;
    show(
        &ctx,
        "SELECT arrow_typeof(CAST(1 AS DECIMAL(10,2)) + CAST(1 AS DECIMAL(5,4))) AS add_t, \
         arrow_typeof(CAST(1 AS DECIMAL(10,2)) * CAST(1 AS DECIMAL(5,4))) AS mul_t, \
         arrow_typeof(CAST(1 AS DECIMAL(10,2)) / CAST(3 AS DECIMAL(5,4))) AS div_t",
    )
    .await;
    show(&ctx, "SELECT CAST(1 AS DECIMAL(10,2)) / CAST(3 AS DECIMAL(10,2)) AS third, CAST(1 AS DECIMAL(10,2)) / CAST(3 AS DECIMAL(10,2)) * CAST(3 AS DECIMAL(10,2)) AS times_three").await;
    show(&ctx, "SELECT CAST(2 AS DECIMAL(10,2)) / CAST(3 AS DECIMAL(10,2)) AS two_thirds, round(2.0 / 3.0, 6) AS float_rounded").await;
    show(&ctx, "SELECT CAST(0.1 AS DECIMAL(10,1)) + CAST(0.2 AS DECIMAL(10,1)) = CAST(0.3 AS DECIMAL(10,1)) AS exact").await;
    show(
        &ctx,
        "SELECT arrow_typeof(0.1) AS literal_t, 0.1 + 0.2 = 0.3 AS float_eq",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE ledger AS SELECT CAST(0.1 AS DOUBLE) AS f, CAST(0.1 AS DECIMAL(20,2)) AS d FROM generate_series(1, 1000000)",
    )
    .await;
    show(
        &ctx,
        "SELECT sum(f) AS float_sum, sum(d) AS decimal_sum, arrow_typeof(sum(d)) AS t FROM ledger",
    )
    .await;
    show(
        &ctx,
        "SELECT arrow_typeof(99999999999999999999999999999999999999) AS big_literal_t",
    )
    .await;
    exec(&ctx, "CREATE TABLE dmax AS SELECT CAST('99999999999999999999999999999999999999' AS DECIMAL(38,0)) AS d FROM generate_series(1, 3)").await;
    show(
        &ctx,
        "SELECT d * CAST(10 AS DECIMAL(38,0)) AS times_ten FROM dmax LIMIT 1",
    )
    .await;
    show(
        &ctx,
        "SELECT sum(d) AS total, arrow_typeof(sum(d)) AS t FROM dmax",
    )
    .await;

    println!("== Floats ==\n");
    show(&ctx, "SELECT CAST('NaN' AS DOUBLE) = CAST('NaN' AS DOUBLE) AS nan_eq, CAST('NaN' AS DOUBLE) > 1e308 AS nan_gt").await;
    exec(
        &ctx,
        "CREATE TABLE fl (x DOUBLE) AS VALUES (1.0), (CAST('NaN' AS DOUBLE)), (CAST('Infinity' AS DOUBLE)), (NULL), (-0.0), (0.0)",
    )
    .await;
    show(&ctx, "SELECT x FROM fl ORDER BY x").await;
    show(
        &ctx,
        "SELECT x, count(*) AS n FROM fl GROUP BY x ORDER BY x",
    )
    .await;
    show(&ctx, "SELECT max(x) AS mx, min(x) AS mn FROM fl").await;
    show(&ctx, "SELECT 1.0 / 0.0 AS div_zero_float").await;
    show(&ctx, "SELECT 1 / 0 AS div_zero_int").await;

    println!("== Strings ==\n");
    exec(
        &ctx,
        "CREATE TABLE names (n VARCHAR) AS VALUES ('banana'), ('Apple'), ('apple'), ('Zebra'), ('éclair'), ('10'), ('9')",
    )
    .await;
    show(&ctx, "SELECT n FROM names ORDER BY n").await;
    show(&ctx, "SELECT 'abc' = 'abc ' AS trailing_space, 'abc' = 'ABC' AS case_eq, lower('ABC') = 'abc' AS lowered").await;
    show(
        &ctx,
        "SELECT length('éclair') AS chars, octet_length('éclair') AS bytes",
    )
    .await;
    show(&ctx, "SELECT CAST('abc' AS CHAR(5)) = 'abc' AS char_pad, arrow_typeof(CAST('abc' AS CHAR(5))) AS t").await;
    show(&ctx, "SELECT '10' < '9' AS string_cmp, 10 < 9 AS int_cmp").await;

    println!("== Dates and time zones ==\n");
    show(&ctx, "SELECT CAST('2026-03-08 02:30:00' AS TIMESTAMP) AS naive, arrow_typeof(CAST('2026-03-08 02:30:00' AS TIMESTAMP)) AS t").await;
    show(
        &ctx,
        "SELECT CAST('2026-11-03T06:00:00Z' AS TIMESTAMP) AS from_utc",
    )
    .await;
    show(&ctx, "SELECT arrow_cast('2026-11-03T06:00:00Z', 'Timestamp(Nanosecond, Some(\"+05:45\"))') AS kathmandu").await;
    show(&ctx, "SELECT arrow_cast(arrow_cast('2026-03-29T00:30:00Z', 'Timestamp(Second, Some(\"UTC\"))'), 'Timestamp(Second, Some(\"Europe/London\"))') AS before_bst, arrow_cast(arrow_cast('2026-03-29T01:30:00Z', 'Timestamp(Second, Some(\"UTC\"))'), 'Timestamp(Second, Some(\"Europe/London\"))') AS after_bst").await;
    show(&ctx, "SELECT DATE '2026-01-31' + INTERVAL '1 month' AS jan31_plus_month, DATE '2024-02-29' + INTERVAL '1 year' AS leap_plus_year").await;
    show(
        &ctx,
        "SELECT DATE '2026-01-31' + INTERVAL '1 month' - INTERVAL '1 month' AS round_trip",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE raw_t (s VARCHAR) AS VALUES ('2026-02-30')",
    )
    .await;
    show(&ctx, "SELECT CAST(s AS DATE) AS bad_date FROM raw_t").await;
    show(&ctx, "SELECT date_trunc('day', arrow_cast('2026-11-03T20:00:00Z', 'Timestamp(Nanosecond, Some(\"+05:45\"))')) AS trunc_local").await;
}
