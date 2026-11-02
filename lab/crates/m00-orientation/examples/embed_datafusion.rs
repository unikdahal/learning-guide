//! Article: "A tour of real engines". DataFusion as a library inside a program.
//!
//!     cargo run --release -p m00-orientation --example embed_datafusion

use datafusion::functions_aggregate::expr_fn::count;
use datafusion::prelude::*;
use labkit::data::data_file;

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let ctx = SessionContext::new();

    // SQL over a CSV file, with no server and no load step.
    ctx.register_csv("trips", &data_file("trips.csv"), CsvReadOptions::new())
        .await?;
    let by_hour = ctx
        .sql("SELECT hour, count(*) AS trips FROM trips GROUP BY hour ORDER BY hour")
        .await?;
    by_hour.show().await?;

    // The same engine through the DataFrame API.
    let busiest = ctx
        .table("trips")
        .await?
        .aggregate(vec![col("hour")], vec![count(lit(1)).alias("trips")])?
        .sort(vec![col("trips").sort(false, false)])?
        .limit(0, Some(3))?;
    busiest.show().await?;
    Ok(())
}
