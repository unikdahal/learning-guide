//! Article: "Measuring things honestly". Times the running example query properly:
//! first run vs warm runs, wall time vs CPU time, one thread vs all cores.
//!
//!     cargo run --release -p m00-orientation --example measure_query

use std::time::Instant;

use datafusion::prelude::*;
use labkit::data::data_file;
use labkit::timing::{Sample, Summary, process_cpu_time};

const SQL: &str = "SELECT c.region, sum(o.amount) AS revenue \
                   FROM orders o JOIN customer c ON o.cust_id = c.id \
                   WHERE c.segment = 'retail' GROUP BY c.region";

async fn context(threads: usize) -> datafusion::error::Result<SessionContext> {
    let config = SessionConfig::new().with_target_partitions(threads);
    let ctx = SessionContext::new_with_config(config);
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
    Ok(ctx)
}

/// Plans and runs the query once, measuring wall time and the CPU time of every thread.
async fn run_once(ctx: &SessionContext) -> datafusion::error::Result<Sample> {
    let cpu0 = process_cpu_time();
    let t0 = Instant::now();
    let batches = ctx.sql(SQL).await?.collect().await?;
    let wall = t0.elapsed();
    let cpu = process_cpu_time() - cpu0;
    assert_eq!(batches.iter().map(|b| b.num_rows()).sum::<usize>(), 5);
    Ok(Sample { wall, cpu })
}

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    for threads in [1, cores] {
        let ctx = context(threads).await?;

        // The first run pays for things later runs don't: reading file metadata, warming the
        // OS page cache and CPU caches, allocating memory for the first time.
        let first = run_once(&ctx).await?;

        // Warm-up runs are discarded, then every measured run is kept.
        for _ in 0..3 {
            run_once(&ctx).await?;
        }
        let mut wall = Vec::new();
        let mut cpu = Vec::new();
        for _ in 0..20 {
            let s = run_once(&ctx).await?;
            wall.push(s.wall);
            cpu.push(s.cpu);
        }

        let median_cpu = Summary::of(&cpu).median;
        let summary = Summary::of(&wall);
        println!("\n{threads} thread(s)");
        println!(
            "  first run   wall {:>9.3?}  cpu {:>9.3?}",
            first.wall, first.cpu
        );
        println!("  warm runs   {summary}");
        println!(
            "  median cpu {median_cpu:>9.3?}  = {:.1}x the median wall time",
            median_cpu.as_secs_f64() / summary.median.as_secs_f64().max(f64::EPSILON)
        );
    }
    Ok(())
}
