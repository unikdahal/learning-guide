//! Article: "ORDER BY, LIMIT and determinism".
//! LIMIT without ORDER BY, ties, the top-k plan, and why OFFSET pagination drifts.
//!
//!     cargo run --release -p m01-relational --example order_limit

use datafusion::arrow::util::display::array_value_to_string;
use datafusion::physical_plan::displayable;
use datafusion::prelude::*;
use labkit::data::data_file;
use m01_relational::sql::{exec, show};

/// Runs a query and returns its first column as strings.
async fn first_col(ctx: &SessionContext, sql: &str) -> Vec<String> {
    let batches = ctx.sql(sql).await.unwrap().collect().await.unwrap();
    batches
        .iter()
        .flat_map(|b| (0..b.num_rows()).map(|i| array_value_to_string(b.column(0), i).unwrap()))
        .collect()
}

async fn lab_ctx(threads: usize) -> SessionContext {
    let ctx = SessionContext::new_with_config(SessionConfig::new().with_target_partitions(threads));
    ctx.register_parquet(
        "customer",
        &data_file("customer.parquet"),
        ParquetReadOptions::default(),
    )
    .await
    .unwrap();
    ctx
}

#[tokio::main]
async fn main() {
    println!("== LIMIT without ORDER BY, 150,000 customers ==\n");
    for threads in [1, 2] {
        let ctx = lab_ctx(threads).await;
        for run in 1..=4 {
            let ids = first_col(
                &ctx,
                "SELECT id FROM customer WHERE segment = 'retail' LIMIT 3",
            )
            .await;
            println!("{threads} thread(s), run {run}: ids {}", ids.join(", "));
        }
    }

    println!("\n== LIMIT on top of a parallel GROUP BY ==\n");
    for threads in [1, 2] {
        let ctx = lab_ctx(threads).await;
        for run in 1..=4 {
            let regions = first_col(
                &ctx,
                "SELECT region FROM (SELECT region, count(*) AS n FROM customer GROUP BY region) LIMIT 2",
            )
            .await;
            println!("{threads} thread(s), run {run}: {}", regions.join(", "));
        }
    }

    println!("\n== ORDER BY with ties: 25 (segment, region) groups, 5 per segment ==\n");
    let ctx = lab_ctx(2).await;
    let tied = "SELECT segment || '/' || region FROM \
                (SELECT segment, region, count(*) AS n FROM customer GROUP BY segment, region) \
                ORDER BY segment LIMIT 3";
    let broken = "SELECT segment || '/' || region FROM \
                  (SELECT segment, region, count(*) AS n FROM customer GROUP BY segment, region) \
                  ORDER BY segment, region LIMIT 3";
    for run in 1..=6 {
        println!(
            "ORDER BY segment, run {run}: {}",
            first_col(&ctx, tied).await.join(", ")
        );
    }
    for run in 1..=3 {
        println!(
            "ORDER BY segment, region, run {run}: {}",
            first_col(&ctx, broken).await.join(", ")
        );
    }

    println!("\n== ORDER BY ... LIMIT is planned as top-k, not a full sort ==\n");
    let df = ctx
        .sql("SELECT id FROM customer ORDER BY segment, id LIMIT 3")
        .await
        .unwrap();
    let plan = df.create_physical_plan().await.unwrap();
    println!("{}", displayable(plan.as_ref()).indent(true));

    println!("== OFFSET pagination while rows arrive ==\n");
    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE feed (id INT, posted INT) AS VALUES (1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)",
    )
    .await;
    show(
        &ctx,
        "SELECT id, posted FROM feed ORDER BY posted DESC LIMIT 3 OFFSET 0",
    )
    .await;
    exec(&ctx, "INSERT INTO feed VALUES (7, 70)").await;
    println!("(a new post, id 7, arrives before the next page is fetched)\n");
    show(
        &ctx,
        "SELECT id, posted FROM feed ORDER BY posted DESC LIMIT 3 OFFSET 3",
    )
    .await;
    println!("Keyset pagination remembers the last row seen instead of counting:\n");
    show(
        &ctx,
        "SELECT id, posted FROM feed WHERE posted < 40 ORDER BY posted DESC LIMIT 3",
    )
    .await;
}
