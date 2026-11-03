//! Article: "Window function semantics".
//! Partitions, ordering, peers and frames: ROWS vs RANGE vs GROUPS, the default frame, the
//! last_value trap, and ranking functions on ties.
//!
//!     cargo run --release -p m01-relational --example windows

use datafusion::prelude::*;
use m01_relational::sql::{exec, show};

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();
    // Day 3 has two rows (peers under ORDER BY day). Day 4 is missing.
    exec(
        &ctx,
        "CREATE TABLE sales (id INT, day INT, region VARCHAR, amount INT) AS VALUES \
         (1, 1, 'east', 10), (2, 2, 'east', 20), (3, 3, 'east', 30), (4, 3, 'east', 5), \
         (5, 5, 'east', 40), (6, 6, 'east', 50), \
         (7, 1, 'west', 7), (8, 2, 'west', 8), (9, 3, 'west', 9)",
    )
    .await;

    println!("== A window keeps every row; GROUP BY collapses them ==\n");
    show(
        &ctx,
        "SELECT region, sum(amount) AS total FROM sales GROUP BY region ORDER BY region",
    )
    .await;
    show(
        &ctx,
        "SELECT id, region, amount, sum(amount) OVER (PARTITION BY region) AS region_total \
         FROM sales ORDER BY id",
    )
    .await;

    println!("== The default frame includes peers ==\n");
    show(
        &ctx,
        "SELECT id, day, amount, \
           sum(amount) OVER (ORDER BY day) AS default_frame, \
           sum(amount) OVER (ORDER BY day ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS rows_frame, \
           sum(amount) OVER (ORDER BY day, id ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS rows_total_order \
         FROM sales WHERE region = 'east' ORDER BY day, id",
    )
    .await;

    println!("== The last_value trap ==\n");
    show(
        &ctx,
        "SELECT id, day, amount, \
           last_value(amount) OVER (ORDER BY day, id) AS default_frame, \
           last_value(amount) OVER (ORDER BY day, id ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING) AS whole_partition, \
           first_value(amount) OVER (ORDER BY day, id) AS first \
         FROM sales WHERE region = 'east' ORDER BY day, id",
    )
    .await;

    println!("== ROWS, RANGE and GROUPS, with a gap at day 4 ==\n");
    show(
        &ctx,
        "SELECT id, day, amount, \
           sum(amount) OVER (ORDER BY day, id ROWS BETWEEN 1 PRECEDING AND CURRENT ROW) AS rows_1, \
           sum(amount) OVER (ORDER BY day RANGE BETWEEN 1 PRECEDING AND CURRENT ROW) AS range_1, \
           sum(amount) OVER (ORDER BY day GROUPS BETWEEN 1 PRECEDING AND CURRENT ROW) AS groups_1 \
         FROM sales WHERE region = 'east' ORDER BY day, id",
    )
    .await;
    show(
        &ctx,
        "SELECT day, amount, \
           avg(amount) OVER (ORDER BY day ROWS BETWEEN 2 PRECEDING AND CURRENT ROW) AS avg_3_rows, \
           avg(amount) OVER (ORDER BY day RANGE BETWEEN 2 PRECEDING AND CURRENT ROW) AS avg_3_days \
         FROM sales WHERE region = 'east' AND id <> 4 ORDER BY day",
    )
    .await;
    show(
        &ctx,
        "SELECT id, day, amount, \
           sum(amount) OVER (ORDER BY day ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING EXCLUDE CURRENT ROW) AS others \
         FROM sales WHERE region = 'east' ORDER BY day, id",
    )
    .await;

    println!("== Ranking functions on ties ==\n");
    show(
        &ctx,
        "SELECT id, day, \
           row_number() OVER (ORDER BY day) AS row_number, \
           rank() OVER (ORDER BY day) AS rank, \
           dense_rank() OVER (ORDER BY day) AS dense_rank, \
           percent_rank() OVER (ORDER BY day) AS percent_rank, \
           ntile(2) OVER (ORDER BY day, id) AS ntile_2 \
         FROM sales WHERE region = 'east' ORDER BY day, id",
    )
    .await;

    println!("== lag and lead look at rows, not values ==\n");
    show(
        &ctx,
        "SELECT day, amount, \
           lag(amount) OVER (ORDER BY day) AS prev_row, \
           lag(day) OVER (ORDER BY day) AS prev_day, \
           lead(amount, 1, 0) OVER (ORDER BY day) AS next_or_zero \
         FROM sales WHERE region = 'east' AND id <> 4 ORDER BY day",
    )
    .await;

    println!("== Several windows, one query ==\n");
    show(
        &ctx,
        "SELECT id, region, day, amount, \
           sum(amount) OVER w AS running, \
           round(100.0 * amount / sum(amount) OVER (PARTITION BY region), 1) AS pct_of_region \
         FROM sales WINDOW w AS (PARTITION BY region ORDER BY day, id) \
         ORDER BY region, day, id",
    )
    .await;
    let df = ctx
        .sql(
            "SELECT id, sum(amount) OVER (PARTITION BY region ORDER BY day) AS running, \
             rank() OVER (ORDER BY amount DESC) AS r FROM sales",
        )
        .await
        .unwrap();
    println!(
        "Optimized logical plan:\n\n{}\n",
        df.into_optimized_plan().unwrap().display_indent()
    );
}
