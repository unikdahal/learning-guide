//! Article: "Joins in relational algebra".
//! Product, theta join and natural join on the tiny engine; then how DataFusion turns a filtered
//! product into a join, the NATURAL JOIN trap, and what a join without equality costs.
//!
//!     cargo run --release -p m01-relational --example joins

use datafusion::physical_plan::displayable;
use datafusion::prelude::*;
use labkit::timing::{Summary, repeat_async};
use m01_relational::sample;

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let c = sample::customer().alias("c");
    let o = sample::orders().alias("o");

    let product = c.product(&o);
    println!(
        "c × o has {} × {} = {} tuples\n",
        c.len(),
        o.len(),
        product.len()
    );

    let joined = c.theta_join(&o, |t| t.get("c.id") == t.get("o.cust_id"));
    println!(
        "c ⋈ c.id = o.cust_id o\n{}\n",
        joined.project(&["c.name", "o.order_id", "o.amount"])
    );

    let big = c.theta_join(&o, |t| {
        t.int("o.amount") > 100 && t.text("c.segment") == "retail"
    });
    println!(
        "c ⋈ (o.amount > 100 ∧ c.segment = 'retail') o: {} tuples\n",
        big.len()
    );

    let natural = sample::customer()
        .rename("id", "cust_id")
        .natural_join(&sample::orders());
    println!("customer(id→cust_id) ⋈ orders\n{natural}\n");

    println!("== A filtered product, as the optimizer sees it ==\n");
    let ctx = SessionContext::new();
    ctx.sql(
        "CREATE TABLE customer (id INT, name VARCHAR, updated_at DATE) AS VALUES \
         (1, 'Asha', DATE '2026-01-05'), (3, 'Chen', DATE '2026-02-11'), (4, 'Dara', DATE '2026-03-02')",
    )
    .await?
    .collect()
    .await?;
    ctx.sql(
        "CREATE TABLE orders (order_id INT, cust_id INT, amount INT, updated_at DATE) AS VALUES \
         (101, 1, 250, DATE '2026-04-01'), (102, 3, 40, DATE '2026-04-02'), (104, 4, 610, DATE '2026-04-03')",
    )
    .await?
    .collect()
    .await?;

    let df = ctx
        .sql("SELECT c.name, o.amount FROM customer c, orders o WHERE c.id = o.cust_id")
        .await?;
    println!("Logical plan:\n{}\n", df.logical_plan().display_indent());
    println!(
        "Optimized:\n{}\n",
        df.clone().into_optimized_plan()?.display_indent()
    );

    println!("== NATURAL JOIN matches every shared column name ==\n");
    ctx.sql("SELECT c.name, o.amount FROM customer c JOIN orders o ON c.id = o.cust_id")
        .await?
        .show()
        .await?;
    ctx.sql("CREATE VIEW customer2 AS SELECT id AS cust_id, name, updated_at FROM customer")
        .await?
        .collect()
        .await?;
    let natural = ctx
        .sql("SELECT * FROM customer2 NATURAL JOIN orders")
        .await?;
    println!(
        "{}\n",
        natural.clone().into_optimized_plan()?.display_indent()
    );
    natural.show().await?;

    println!("\n== Equality lets the engine skip non-matching pairs ==\n");
    let n = 5_000;
    for t in ["a", "b"] {
        ctx.sql(&format!(
            "CREATE TABLE {t} AS SELECT value AS k FROM generate_series(1, {n})"
        ))
        .await?
        .collect()
        .await?;
    }
    // Both queries return exactly 5,000 rows, one per value of a.k.
    let equi = "SELECT count(*) FROM a JOIN b ON a.k = b.k";
    let theta = format!("SELECT count(*) FROM a JOIN b ON a.k + b.k = {}", n + 1);
    // The same condition, rearranged so each side of the equality uses one table only.
    let rewritten = format!("SELECT count(*) FROM a JOIN b ON b.k = {} - a.k", n + 1);
    for (name, sql) in [
        ("equality", equi.to_string()),
        ("expression", theta),
        ("rewritten", rewritten),
    ] {
        let df = ctx.sql(&sql).await?;
        let plan = df.clone().create_physical_plan().await?;
        let text = displayable(plan.as_ref()).indent(true).to_string();
        let join = text
            .lines()
            .find(|l| l.contains("Join"))
            .unwrap_or("")
            .trim();
        let samples = repeat_async(3, 20, || async {
            ctx.sql(&sql).await.unwrap().collect().await.unwrap()
        })
        .await;
        println!("{name:<10} {join}");
        println!("           {}\n", Summary::of(&samples));
    }
    Ok(())
}
