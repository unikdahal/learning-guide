//! Article: "Logical processing order of a query".
//! Where aliases are visible, why window functions can't go in WHERE, and how the plan tree
//! records the logical order.
//!
//!     cargo run --release -p m01-relational --example clause_order

use datafusion::prelude::*;
use m01_relational::sql::{exec, show};

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE customer (id INT, name VARCHAR, segment VARCHAR, region VARCHAR) AS VALUES \
         (1, 'Asha', 'retail', 'ASIA'), (2, 'Bruno', 'wholesale', 'EUROPE'), \
         (3, 'Chen', 'retail', 'ASIA'), (4, 'Dara', 'retail', 'AFRICA'), \
         (5, 'Emeka', 'government', 'AFRICA'), (6, 'Farah', 'retail', 'EUROPE')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE orders (order_id INT, cust_id INT, amount INT) AS VALUES \
         (101, 1, 250), (102, 3, 40), (103, 1, 90), (104, 4, 610), (105, 6, 75), (106, 9, 120)",
    )
    .await;

    println!("== Where a SELECT alias is visible ==\n");
    show(
        &ctx,
        "SELECT order_id, amount * 2 AS doubled FROM orders WHERE doubled > 200",
    )
    .await;
    show(
        &ctx,
        "SELECT order_id, amount * 2 AS doubled FROM orders ORDER BY doubled DESC LIMIT 2",
    )
    .await;
    show(
        &ctx,
        "SELECT cust_id % 2 AS parity, count(*) AS n FROM orders GROUP BY parity ORDER BY parity",
    )
    .await;

    println!("== Window functions run after WHERE ==\n");
    show(
        &ctx,
        "SELECT order_id, amount FROM orders WHERE rank() OVER (ORDER BY amount DESC) <= 2",
    )
    .await;
    show(
        &ctx,
        "SELECT order_id, amount FROM orders QUALIFY rank() OVER (ORDER BY amount DESC) <= 2",
    )
    .await;
    show(
        &ctx,
        "SELECT order_id, amount FROM \
         (SELECT order_id, amount, rank() OVER (ORDER BY amount DESC) AS r FROM orders) \
         WHERE r <= 2 ORDER BY amount DESC",
    )
    .await;

    println!("== The plan records the logical order ==\n");
    let sql = "SELECT c.region, sum(o.amount) AS total \
               FROM orders o JOIN customer c ON o.cust_id = c.id \
               WHERE o.amount > 50 \
               GROUP BY c.region HAVING sum(o.amount) > 100 \
               ORDER BY total DESC LIMIT 1";
    let df = ctx.sql(sql).await.unwrap();
    println!("{sql}\n");
    println!(
        "Logical plan, read bottom-up:\n{}\n",
        df.logical_plan().display_indent()
    );
    println!(
        "Optimized:\n{}\n",
        df.clone().into_optimized_plan().unwrap().display_indent()
    );
    df.show().await.unwrap();
}
