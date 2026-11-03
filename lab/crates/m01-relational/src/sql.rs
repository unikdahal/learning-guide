//! A small helper for the examples: run a statement and print its result or its error.

use datafusion::prelude::SessionContext;

/// Prints `sql`, then the result table, or the first line of the error if the engine rejects it.
pub async fn show(ctx: &SessionContext, sql: &str) {
    println!("> {sql}");
    match ctx.sql(sql).await {
        Ok(df) => match df.show().await {
            Ok(()) => {}
            Err(e) => println!("error: {}", first_line(&e.to_string())),
        },
        Err(e) => println!("error: {}", first_line(&e.to_string())),
    }
    println!();
}

/// Runs a statement whose output we don't need, such as CREATE TABLE or INSERT.
pub async fn exec(ctx: &SessionContext, sql: &str) {
    ctx.sql(sql)
        .await
        .expect("statement failed")
        .collect()
        .await
        .expect("statement failed");
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(160).collect()
}
