//! Article: "Relations, tuples and attributes".
//! The same data as a mathematical relation and as an SQL table, and where they differ.
//!
//!     cargo run --release -p m01-relational --example relations

use datafusion::prelude::*;
use labkit::data::data_file;
use m01_relational::{Relation, Value, sample};

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    println!("== A relation is a set of tuples ==\n");
    let customer = sample::customer();
    println!("{customer}\n");

    // Writing the same tuple twice adds nothing: the body is a set.
    let twice = Relation::new(
        ["id", "name"],
        vec![
            [Value::from(1), "Asha".into()],
            [Value::from(1), "Asha".into()],
        ],
    );
    println!("Two identical rows given, tuples stored: {}", twice.len());

    // Listing attributes in a different order describes the same relation.
    let swapped = Relation::new(["name", "id"], vec![["Asha".into(), Value::from(1)]]);
    println!(
        "Same relation with attributes in another order? {}\n",
        twice == swapped
    );

    for key in [
        &["id"][..],
        &["name"],
        &["region"],
        &["id", "region"],
        &["segment", "region"],
    ] {
        println!(
            "{:<22} superkey: {:<5}  candidate key: {}",
            format!("{key:?}"),
            customer.is_superkey(key),
            customer.is_candidate_key(key)
        );
    }

    println!("\n== An SQL table is a bag of rows ==\n");
    let ctx = SessionContext::new();

    // A primary key is declared, then violated. DataFusion records constraints but, like
    // Snowflake and BigQuery, does not enforce uniqueness.
    ctx.sql("CREATE TABLE people (id INT PRIMARY KEY, name VARCHAR)")
        .await?
        .collect()
        .await?;
    ctx.sql("INSERT INTO people VALUES (1, 'Asha'), (1, 'Asha')")
        .await?
        .collect()
        .await?;
    ctx.sql("SELECT * FROM people").await?.show().await?;

    ctx.register_parquet(
        "customer",
        &data_file("customer.parquet"),
        ParquetReadOptions::default(),
    )
    .await?;
    ctx.sql("SELECT count(*) AS rows_kept FROM (SELECT region FROM customer)")
        .await?
        .show()
        .await?;
    ctx.sql("SELECT count(*) AS rows_kept FROM (SELECT DISTINCT region FROM customer)")
        .await?
        .show()
        .await?;

    println!("\n== Without ORDER BY, row order is whatever the plan produces ==\n");
    let q = "SELECT region, count(*) AS n FROM customer GROUP BY region";
    for run in 1..=5 {
        let batches = ctx.sql(q).await?.collect().await?;
        let regions: Vec<String> = batches
            .iter()
            .flat_map(|b| {
                let col = b.column(0);
                (0..b.num_rows())
                    .map(|i| {
                        datafusion::arrow::util::display::array_value_to_string(col, i).unwrap()
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        println!("run {run}: {}", regions.join(", "));
    }
    Ok(())
}
