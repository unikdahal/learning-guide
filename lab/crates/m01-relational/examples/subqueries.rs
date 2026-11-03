//! Article: "Subqueries".
//! Scalar, IN, EXISTS and correlated subqueries: their meaning, their NULL edge cases, the
//! COUNT trap, and how DataFusion turns them into joins.
//!
//!     cargo run --release -p m01-relational --example subqueries

use datafusion::prelude::*;
use m01_relational::sql::{exec, show};

/// The definition of a correlated subquery, run literally: for each outer row, evaluate the
/// inner query again with the outer row's values bound. Counts how many inner rows are read.
fn correlated_by_definition(customers: &[(i32, &str)], orders: &[(i32, i32, i32)]) {
    let mut inner_rows_read = 0;
    println!("{:<6} {:>6} {:>6}", "name", "count", "sum");
    for &(id, name) in customers {
        let mut count = 0;
        let mut sum: Option<i32> = None;
        for &(_, cust_id, amount) in orders {
            inner_rows_read += 1;
            if cust_id == id {
                count += 1;
                sum = Some(sum.unwrap_or(0) + amount);
            }
        }
        let sum = sum.map_or("NULL".to_string(), |s| s.to_string());
        println!("{name:<6} {count:>6} {sum:>6}");
    }
    println!(
        "\n{} outer rows x {} inner rows = {inner_rows_read} inner rows read\n",
        customers.len(),
        orders.len()
    );
}

#[tokio::main]
async fn main() {
    let customers = [
        (1, "Asha"),
        (2, "Bruno"),
        (3, "Chen"),
        (4, "Dara"),
        (5, "Emeka"),
        (6, "Farah"),
    ];
    let orders = [
        (101, 1, 250),
        (102, 3, 40),
        (103, 1, 90),
        (104, 4, 610),
        (105, 6, 75),
        (106, 9, 120),
    ];

    println!("== A correlated subquery, evaluated by its definition ==\n");
    correlated_by_definition(&customers, &orders);

    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE customer (id INT, name VARCHAR) AS VALUES \
         (1, 'Asha'), (2, 'Bruno'), (3, 'Chen'), (4, 'Dara'), (5, 'Emeka'), (6, 'Farah')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE orders (order_id INT, cust_id INT, amount INT) AS VALUES \
         (101, 1, 250), (102, 3, 40), (103, 1, 90), (104, 4, 610), (105, 6, 75), (106, 9, 120)",
    )
    .await;

    println!("== Scalar subqueries ==\n");
    show(
        &ctx,
        "SELECT order_id, amount FROM orders WHERE amount > (SELECT avg(amount) FROM orders) ORDER BY order_id",
    )
    .await;
    show(
        &ctx,
        "SELECT name, (SELECT amount FROM orders WHERE cust_id = 5) AS emeka_order FROM customer WHERE id = 1",
    )
    .await;
    show(
        &ctx,
        "SELECT name, (SELECT amount FROM orders WHERE cust_id = 1) AS asha_order FROM customer WHERE id = 1",
    )
    .await;

    println!("== Correlated scalar subqueries, and the COUNT trap ==\n");
    show(
        &ctx,
        "SELECT name, \
           (SELECT count(*) FROM orders o WHERE o.cust_id = c.id) AS n, \
           (SELECT sum(amount) FROM orders o WHERE o.cust_id = c.id) AS total \
         FROM customer c ORDER BY c.id",
    )
    .await;
    println!("The same question as a join and GROUP BY:\n");
    show(
        &ctx,
        "SELECT c.name, count(*) AS n, sum(o.amount) AS total \
         FROM customer c LEFT JOIN orders o ON o.cust_id = c.id GROUP BY c.id, c.name ORDER BY c.id",
    )
    .await;
    show(
        &ctx,
        "SELECT c.name, count(o.order_id) AS n, sum(o.amount) AS total \
         FROM customer c LEFT JOIN orders o ON o.cust_id = c.id GROUP BY c.id, c.name ORDER BY c.id",
    )
    .await;
    let df = ctx
        .sql(
            "SELECT name, (SELECT count(*) FROM orders o WHERE o.cust_id = c.id) AS n FROM customer c",
        )
        .await
        .unwrap();
    println!(
        "Optimized logical plan of the correlated count:\n\n{}\n",
        df.into_optimized_plan().unwrap().display_indent()
    );

    println!("== IN and EXISTS ==\n");
    show(
        &ctx,
        "SELECT name FROM customer WHERE id IN (SELECT cust_id FROM orders) ORDER BY id",
    )
    .await;
    show(
        &ctx,
        "SELECT name FROM customer c WHERE EXISTS (SELECT 1 FROM orders o WHERE o.cust_id = c.id) ORDER BY id",
    )
    .await;
    show(
        &ctx,
        "SELECT name FROM customer c WHERE EXISTS (SELECT NULL FROM orders o WHERE o.cust_id = c.id) ORDER BY id",
    )
    .await;
    let df = ctx
        .sql("SELECT name FROM customer c WHERE EXISTS (SELECT 1 FROM orders o WHERE o.cust_id = c.id)")
        .await
        .unwrap();
    println!(
        "Optimized logical plan of the EXISTS:\n\n{}\n",
        df.into_optimized_plan().unwrap().display_indent()
    );

    println!("== IN with NULLs ==\n");
    exec(
        &ctx,
        "CREATE TABLE probe (x INT) AS VALUES (1), (2), (NULL)",
    )
    .await;
    exec(&ctx, "CREATE TABLE list (y INT) AS VALUES (1), (NULL)").await;
    show(
        &ctx,
        "SELECT x, x IN (1, NULL) AS is_in, x NOT IN (1, NULL) AS not_in FROM probe ORDER BY x",
    )
    .await;
    show(&ctx, "SELECT x FROM probe WHERE x IN (SELECT y FROM list)").await;
    show(
        &ctx,
        "SELECT x FROM probe WHERE x NOT IN (SELECT y FROM list)",
    )
    .await;
    show(
        &ctx,
        "SELECT x FROM probe WHERE NOT EXISTS (SELECT 1 FROM list WHERE y = x) ORDER BY x",
    )
    .await;
    println!("A subquery predicate in the SELECT list, rather than in WHERE:\n");
    show(
        &ctx,
        "SELECT x, x IN (SELECT y FROM list) AS is_in FROM probe ORDER BY x",
    )
    .await;

    println!("== Correlation beyond equality ==\n");
    show(
        &ctx,
        "SELECT order_id, cust_id, amount FROM orders o \
         WHERE amount > (SELECT avg(amount) FROM orders o2 WHERE o2.cust_id = o.cust_id) ORDER BY order_id",
    )
    .await;
    show(
        &ctx,
        "SELECT order_id, amount, \
           (SELECT count(*) FROM orders o2 WHERE o2.amount > o.amount) AS larger_orders \
         FROM orders o ORDER BY amount DESC",
    )
    .await;
    show(
        &ctx,
        "SELECT name, (SELECT max(amount) FROM orders o WHERE o.cust_id = c.id AND o.amount < c.id * 100) AS m FROM customer c ORDER BY c.id",
    )
    .await;
    println!("The non-equality subquery rewritten by hand, as a join and a grouping:\n");
    show(
        &ctx,
        "SELECT o.order_id, o.amount, count(o2.order_id) AS larger_orders \
         FROM orders o LEFT JOIN orders o2 ON o2.amount > o.amount \
         GROUP BY o.order_id, o.amount ORDER BY o.amount DESC",
    )
    .await;
}
