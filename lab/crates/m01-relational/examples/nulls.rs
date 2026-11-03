//! Article: "NULL and three-valued logic".
//! Checks the tiny engine's truth tables against DataFusion, then shows NULL in comparisons,
//! WHERE, NOT IN, grouping, joins and sorting.
//!
//!     cargo run --release -p m01-relational --example nulls

use datafusion::arrow::array::{Array, BooleanArray};
use datafusion::prelude::*;
use m01_relational::logic::Truth;
use m01_relational::sql::{exec, show};

fn to_truth(a: &BooleanArray, i: usize) -> Truth {
    if a.is_null(i) {
        Truth::Unknown
    } else {
        Truth::from(a.value(i))
    }
}

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();

    println!("== Truth tables: tiny engine vs DataFusion ==\n");
    let lit = |t: Truth| match t {
        Truth::True => "TRUE",
        Truth::False => "FALSE",
        Truth::Unknown => "CAST(NULL AS BOOLEAN)",
    };
    let mut rows = Vec::new();
    for a in Truth::ALL {
        for b in Truth::ALL {
            rows.push(format!("({}, {})", lit(a), lit(b)));
        }
    }
    exec(
        &ctx,
        &format!(
            "CREATE TABLE t (a BOOLEAN, b BOOLEAN) AS VALUES {}",
            rows.join(", ")
        ),
    )
    .await;
    let batches = ctx
        .sql("SELECT a, b, a AND b, a OR b, NOT a FROM t")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let mut checked = 0;
    println!(
        "{:<8} {:<8} {:<8} {:<8} {:<8}",
        "a", "b", "a AND b", "a OR b", "NOT a"
    );
    for batch in &batches {
        let col = |i: usize| {
            batch
                .column(i)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap()
        };
        for r in 0..batch.num_rows() {
            let (a, b) = (to_truth(col(0), r), to_truth(col(1), r));
            let (and, or, not) = (
                to_truth(col(2), r),
                to_truth(col(3), r),
                to_truth(col(4), r),
            );
            assert_eq!(and, a.and(b), "AND disagrees for {a} {b}");
            assert_eq!(or, a.or(b), "OR disagrees for {a} {b}");
            assert_eq!(not, a.not(), "NOT disagrees for {a}");
            println!(
                "{:<8} {:<8} {:<8} {:<8} {:<8}",
                a.to_string(),
                b.to_string(),
                and.to_string(),
                or.to_string(),
                not.to_string()
            );
            checked += 1;
        }
    }
    println!("\nAll {checked} combinations agree with the tiny engine.\n");

    println!("== Comparisons with NULL ==\n");
    show(
        &ctx,
        "SELECT NULL = NULL AS eq, NULL <> 1 AS ne, NULL IS NULL AS is_null, \
         NULL IS NOT DISTINCT FROM NULL AS not_distinct",
    )
    .await;

    println!("== WHERE keeps only TRUE ==\n");
    exec(
        &ctx,
        "CREATE TABLE customer (id INT, name VARCHAR, segment VARCHAR) AS VALUES \
         (1, 'Asha', 'retail'), (2, 'Bruno', 'wholesale'), (3, 'Chen', NULL), \
         (4, 'Dara', 'retail'), (5, 'Emeka', NULL)",
    )
    .await;
    show(&ctx, "SELECT count(*) AS everyone FROM customer").await;
    show(
        &ctx,
        "SELECT count(*) AS retail FROM customer WHERE segment = 'retail'",
    )
    .await;
    show(
        &ctx,
        "SELECT count(*) AS not_retail FROM customer WHERE NOT (segment = 'retail')",
    )
    .await;

    println!("== The NOT IN trap ==\n");
    exec(
        &ctx,
        "CREATE TABLE orders (order_id INT, cust_id INT) AS VALUES (101, 1), (102, 4), (103, NULL)",
    )
    .await;
    show(
        &ctx,
        "SELECT name FROM customer WHERE id NOT IN (SELECT cust_id FROM orders)",
    )
    .await;
    show(
        &ctx,
        "SELECT name FROM customer c WHERE NOT EXISTS \
         (SELECT 1 FROM orders o WHERE o.cust_id = c.id) ORDER BY name",
    )
    .await;
    let df = ctx
        .sql("SELECT name FROM customer WHERE id NOT IN (SELECT cust_id FROM orders)")
        .await
        .unwrap();
    println!("{}\n", df.into_optimized_plan().unwrap().display_indent());

    println!("== Grouping, DISTINCT and counting treat NULLs together ==\n");
    show(
        &ctx,
        "SELECT segment, count(*) AS n FROM customer GROUP BY segment ORDER BY segment",
    )
    .await;
    show(
        &ctx,
        "SELECT count(DISTINCT segment) AS distinct_segments, count(segment) AS with_segment, \
         count(*) AS rows FROM customer",
    )
    .await;

    println!("== NULL join keys never match ==\n");
    exec(&ctx, "CREATE TABLE a (k INT) AS VALUES (1), (NULL)").await;
    exec(&ctx, "CREATE TABLE b (k INT) AS VALUES (1), (NULL)").await;
    show(
        &ctx,
        "SELECT count(*) AS matches FROM a JOIN b ON a.k = b.k",
    )
    .await;
    show(
        &ctx,
        "SELECT count(*) AS matches FROM a JOIN b ON a.k IS NOT DISTINCT FROM b.k",
    )
    .await;

    println!("== Where NULLs sort ==\n");
    show(
        &ctx,
        "SELECT name, segment FROM customer ORDER BY segment, id",
    )
    .await;
    show(
        &ctx,
        "SELECT name, segment FROM customer ORDER BY segment DESC, id",
    )
    .await;
}
