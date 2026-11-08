//! Article: "What a query engine actually does".
//! Prints the logical, optimized and physical plans for the running example, then runs it.
//!
//!     cargo run --release -p m00-orientation --example query_stages

use datafusion::physical_plan::displayable;
use datafusion::prelude::*;
use labkit::data::data_file;

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let ctx = SessionContext::new();
    ctx.register_parquet(
        "orders",
        &data_file("orders.parquet"),
        ParquetReadOptions::default(),
    )
    .await?;
    ctx.register_parquet(
        "customer",
        &data_file("customer.parquet"),
        ParquetReadOptions::default(),
    )
    .await?;

    let sql = "SELECT c.region, sum(o.amount) AS revenue \
               FROM orders o JOIN customer c ON o.cust_id = c.id \
               WHERE c.segment = 'retail' GROUP BY c.region";

    // Parse and bind: SQL text to a logical plan.
    let df = ctx.sql(sql).await?;
    println!("Logical plan:\n{}\n", df.logical_plan().display_indent());

    // Logical optimization.
    let optimized = df.clone().into_optimized_plan()?;
    println!("Optimized plan:\n{}\n", optimized.display_indent());

    // Physical planning.
    let physical = df.clone().create_physical_plan().await?;
    println!(
        "Physical plan:\n{}",
        displayable(physical.as_ref()).indent(true)
    );

    // Execution.
    labkit::data::assert_revenue(&df.clone().collect().await?);
    df.show().await?;
    Ok(())
}
