//! Article: "Relational algebra: selection, projection, rename".
//! σ, π and ρ on the tiny engine, then the same operators inside DataFusion's plans.
//!
//!     cargo run --release -p m01-relational --example algebra_ops

use datafusion::prelude::*;
use labkit::data::data_file;
use m01_relational::sample;

async fn plans(ctx: &SessionContext, sql: &str) -> datafusion::error::Result<()> {
    let df = ctx.sql(sql).await?;
    println!("{sql}\n");
    println!("Logical plan:\n{}\n", df.logical_plan().display_indent());
    println!(
        "Optimized:\n{}\n",
        df.clone().into_optimized_plan()?.display_indent()
    );
    Ok(())
}

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let customer = sample::customer();

    println!("σ segment = 'retail' (customer)\n");
    let retail = customer.select(|t| t.text("segment") == "retail");
    println!("{retail}\n");

    println!("π region (σ segment = 'retail' (customer))\n");
    println!("{}\n", retail.project(&["region"]));

    println!("ρ id→cust_id (customer)\n");
    println!(
        "{}\n",
        customer
            .rename("id", "cust_id")
            .project(&["cust_id", "name"])
    );

    println!("== The same operators in DataFusion ==\n");
    let ctx = SessionContext::new();
    ctx.register_parquet(
        "customer",
        &data_file("customer.parquet"),
        ParquetReadOptions::default(),
    )
    .await?;

    plans(&ctx, "SELECT region FROM customer WHERE segment = 'retail'").await?;
    plans(
        &ctx,
        "SELECT DISTINCT region FROM customer WHERE segment = 'retail'",
    )
    .await?;
    plans(&ctx, "SELECT c.id AS cust_id, c.name FROM customer AS c").await?;

    for sql in [
        "SELECT count(*) AS n FROM (SELECT region FROM customer WHERE segment = 'retail')",
        "SELECT count(*) AS n FROM (SELECT DISTINCT region FROM customer WHERE segment = 'retail')",
    ] {
        println!("{sql}");
        ctx.sql(sql).await?.show().await?;
    }
    Ok(())
}
