//! Article: "From SQL to relational algebra by hand".
//! Five queries translated into algebra, evaluated on the tiny engine, checked against
//! DataFusion, and compared with the logical plan DataFusion builds for the same SQL.
//!
//!     cargo run --release -p m01-relational --example sql_to_algebra

use std::collections::BTreeSet;

use datafusion::arrow::util::display::array_value_to_string;
use datafusion::prelude::*;
use m01_relational::sql::exec;
use m01_relational::{Relation, sample};

type Rows = BTreeSet<Vec<String>>;

fn rows_of(r: &Relation, cols: &[&str]) -> Rows {
    let idx: Vec<usize> = cols
        .iter()
        .map(|c| r.attrs().iter().position(|a| a == c).unwrap())
        .collect();
    r.tuples()
        .map(|t| idx.iter().map(|&i| t[i].to_string()).collect())
        .collect()
}

async fn sql_rows(ctx: &SessionContext, sql: &str) -> (Rows, usize) {
    let batches = ctx.sql(sql).await.unwrap().collect().await.unwrap();
    let mut out = BTreeSet::new();
    let mut n = 0;
    for b in &batches {
        for i in 0..b.num_rows() {
            n += 1;
            out.insert(
                (0..b.num_columns())
                    .map(|c| array_value_to_string(b.column(c), i).unwrap())
                    .collect(),
            );
        }
    }
    (out, n)
}

async fn check(
    ctx: &SessionContext,
    title: &str,
    algebra: &str,
    sql: &str,
    r: &Relation,
    cols: &[&str],
) {
    println!("== {title} ==\n");
    println!("SQL:     {sql}");
    println!("Algebra: {algebra}\n");
    let ours = rows_of(r, cols);
    let (theirs, sql_row_count) = sql_rows(ctx, sql).await;
    let shown: Vec<String> = ours.iter().map(|t| t.join(" | ")).collect();
    println!("Tiny engine ({} tuples): {}", ours.len(), shown.join(", "));
    println!(
        "DataFusion returned {sql_row_count} rows, {} distinct: {}",
        theirs.len(),
        if ours == theirs {
            "same set"
        } else {
            "DIFFERENT"
        }
    );
    let plan = ctx.sql(sql).await.unwrap().into_unoptimized_plan();
    println!(
        "\nDataFusion's unoptimized logical plan:\n{}\n",
        plan.display_indent()
    );
}

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE customer (id BIGINT, name VARCHAR, segment VARCHAR, region VARCHAR) AS VALUES \
         (1, 'Asha', 'retail', 'ASIA'), (2, 'Bruno', 'wholesale', 'EUROPE'), \
         (3, 'Chen', 'retail', 'ASIA'), (4, 'Dara', 'retail', 'AFRICA'), \
         (5, 'Emeka', 'government', 'AFRICA'), (6, 'Farah', 'retail', 'EUROPE')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE orders (order_id BIGINT, cust_id BIGINT, amount BIGINT) AS VALUES \
         (101, 1, 250), (102, 3, 40), (103, 1, 90), (104, 4, 610), (105, 6, 75), (106, 9, 120)",
    )
    .await;
    let customer = sample::customer();
    let orders = sample::orders();

    // 1. One table: FROM is the relation, WHERE is σ, SELECT is π.
    let q1 = customer
        .select(|t| t.text("region") == "ASIA")
        .project(&["name"]);
    check(
        &ctx,
        "1. Selection and projection",
        "π name (σ region = 'ASIA' (customer))",
        "SELECT name FROM customer WHERE region = 'ASIA'",
        &q1,
        &["name"],
    )
    .await;

    // 2. A join and a filter on the joined rows.
    let c = customer.alias("c");
    let o = orders.alias("o");
    let q2 = c
        .theta_join(&o, |t| t.get("c.id") == t.get("o.cust_id"))
        .select(|t| t.int("o.amount") > 80)
        .project(&["c.name", "o.amount"]);
    check(
        &ctx,
        "2. Join, then filter",
        "π c.name, o.amount (σ o.amount > 80 (ρ c (customer) ⋈ c.id = o.cust_id ρ o (orders)))",
        "SELECT c.name, o.amount FROM customer c JOIN orders o ON c.id = o.cust_id WHERE o.amount > 80",
        &q2,
        &["c.name", "o.amount"],
    )
    .await;

    // 3. Duplicates: the SQL returns Asha twice, the algebra once.
    let q3 = c
        .theta_join(&o, |t| t.get("c.id") == t.get("o.cust_id"))
        .project(&["c.name"]);
    check(
        &ctx,
        "3. Where SQL and set algebra disagree on duplicates",
        "π c.name (ρ c (customer) ⋈ c.id = o.cust_id ρ o (orders))",
        "SELECT c.name FROM customer c JOIN orders o ON c.id = o.cust_id",
        &q3,
        &["c.name"],
    )
    .await;

    // 4. NOT EXISTS as difference: all customers minus those with an order.
    let with_orders = c
        .theta_join(&o, |t| t.get("c.id") == t.get("o.cust_id"))
        .project(&["c.id", "c.name"]);
    let q4 = c.project(&["c.id", "c.name"]).difference(&with_orders);
    check(
        &ctx,
        "4. NOT EXISTS as set difference",
        "π c.id, c.name (ρ c (customer)) − π c.id, c.name (ρ c (customer) ⋈ c.id = o.cust_id ρ o (orders))",
        "SELECT c.id, c.name FROM customer c WHERE NOT EXISTS (SELECT 1 FROM orders o WHERE o.cust_id = c.id)",
        &q4,
        &["c.id", "c.name"],
    )
    .await;

    // 5. A self-join needs rename: pairs of customers in the same region.
    let a = customer.alias("a");
    let b = customer.alias("b");
    let q5 = a
        .theta_join(&b, |t| {
            t.get("a.region") == t.get("b.region") && t.int("a.id") < t.int("b.id")
        })
        .project(&["a.name", "b.name", "a.region"]);
    check(
        &ctx,
        "5. Self-join with rename",
        "π a.name, b.name, a.region (ρ a (customer) ⋈ a.region = b.region ∧ a.id < b.id ρ b (customer))",
        "SELECT a.name, b.name, a.region FROM customer a JOIN customer b ON a.region = b.region AND a.id < b.id",
        &q5,
        &["a.name", "b.name", "a.region"],
    )
    .await;

    // 6. Aggregation is beyond the five basic operators: γ, shown as DataFusion plans it.
    let sql = "SELECT c.region, sum(o.amount) AS total FROM customer c JOIN orders o ON c.id = o.cust_id \
               WHERE c.segment = 'retail' GROUP BY c.region HAVING sum(o.amount) > 100 ORDER BY total DESC";
    println!("== 6. Grouping: the extended operator γ ==\n");
    println!("SQL:     {sql}");
    println!(
        "Algebra: τ total DESC (σ total > 100 (γ c.region; sum(o.amount) → total (σ c.segment = 'retail' (ρ c (customer) ⋈ c.id = o.cust_id ρ o (orders)))))\n"
    );
    let df = ctx.sql(sql).await.unwrap();
    println!(
        "DataFusion's unoptimized logical plan:\n{}\n",
        df.clone().into_unoptimized_plan().display_indent()
    );
    df.show().await.unwrap();
}
