//! Article: "Grouping and aggregation semantics".
//! Partition then reduce, NULLs in aggregates, WHERE vs HAVING, empty input, functional
//! dependencies, and why aggregation needs bags.
//!
//!     cargo run --release -p m01-relational --example grouping

use datafusion::prelude::*;
use m01_relational::bag::Bag;
use m01_relational::sql::{exec, show};
use m01_relational::{Relation, Value};

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE orders (order_id INT, region VARCHAR, amount INT) AS VALUES \
         (101, 'ASIA', 250), (102, 'ASIA', 40), (103, 'ASIA', 90), \
         (104, 'AFRICA', 610), (105, 'EUROPE', 75), (106, 'EUROPE', NULL), \
         (107, 'EUROPE', 40)",
    )
    .await;

    println!("== One row per group ==\n");
    show(
        &ctx,
        "SELECT region, count(*) AS n, count(amount) AS n_amount, sum(amount) AS total, \
         avg(amount) AS mean, min(amount) AS lo, max(amount) AS hi \
         FROM orders GROUP BY region ORDER BY region",
    )
    .await;

    println!("== WHERE filters rows before grouping; HAVING filters groups after ==\n");
    show(
        &ctx,
        "SELECT region, sum(amount) AS total FROM orders WHERE amount > 50 \
         GROUP BY region ORDER BY region",
    )
    .await;
    show(
        &ctx,
        "SELECT region, sum(amount) AS total FROM orders \
         GROUP BY region HAVING sum(amount) > 200 ORDER BY region",
    )
    .await;

    println!("== Aggregating nothing ==\n");
    show(
        &ctx,
        "SELECT count(*) AS n, sum(amount) AS total, max(amount) AS hi FROM orders WHERE region = 'MARS'",
    )
    .await;
    show(
        &ctx,
        "SELECT region, count(*) AS n FROM orders WHERE region = 'MARS' GROUP BY region",
    )
    .await;

    println!("== Columns that are neither grouped nor aggregated ==\n");
    exec(
        &ctx,
        "CREATE TABLE customer (id INT PRIMARY KEY, name VARCHAR, region VARCHAR) AS VALUES \
         (1, 'Asha', 'ASIA'), (3, 'Chen', 'ASIA'), (4, 'Dara', 'AFRICA')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE customer_nokey (id INT, name VARCHAR, region VARCHAR) AS SELECT * FROM customer",
    )
    .await;
    show(&ctx, "SELECT region, name FROM customer GROUP BY region").await;
    show(
        &ctx,
        "SELECT id, name, region FROM customer GROUP BY id ORDER BY id",
    )
    .await;
    show(
        &ctx,
        "SELECT id, name, region FROM customer_nokey GROUP BY id ORDER BY id",
    )
    .await;

    println!("== The plan ==\n");
    let df = ctx
        .sql("SELECT region, sum(amount) AS total FROM orders WHERE amount > 50 GROUP BY region HAVING sum(amount) > 200")
        .await
        .unwrap();
    println!("{}\n", df.into_optimized_plan().unwrap().display_indent());

    println!("== Why aggregation needs bags ==\n");
    // Two payments of 40 by the same customer. As a set, projecting to `amount` merges them.
    let payments = Relation::new(
        ["payment_id", "amount"],
        vec![
            [Value::from(1), Value::from(40)],
            [Value::from(2), Value::from(40)],
            [Value::from(3), Value::from(90)],
        ],
    );
    let as_set = payments.project(&["amount"]);
    println!("π amount as a set: {} tuples", as_set.len());
    let set_sum: i64 = [40, 90].iter().sum();
    let amounts = Bag::new(
        ["amount"],
        vec![[Value::from(40)], [Value::from(40)], [Value::from(90)]],
    );
    let bag_sum: i64 = amounts
        .rows()
        .iter()
        .map(|r| match &r[0] {
            Value::Int(v) => *v,
            _ => 0,
        })
        .sum();
    println!("sum over the set: {set_sum}   sum over the bag: {bag_sum}");
}
