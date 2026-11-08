//! Article: "Set operations and bag semantics".
//! The six set operations on two small bags, computed by the tiny engine and by DataFusion,
//! then the positional matching trap.
//!
//!     cargo run --release -p m01-relational --example set_ops

use datafusion::arrow::util::display::array_value_to_string;
use datafusion::prelude::*;
use m01_relational::Value;
use m01_relational::bag::Bag;
use m01_relational::sql::{exec, show};

/// Runs a one-column query and reads the result back as a bag.
async fn bag_of(ctx: &SessionContext, sql: &str) -> Bag {
    let batches = ctx.sql(sql).await.unwrap().collect().await.unwrap();
    let rows: Vec<Vec<Value>> = batches
        .iter()
        .flat_map(|b| {
            (0..b.num_rows())
                .map(|i| vec![Value::Text(array_value_to_string(b.column(0), i).unwrap())])
                .collect::<Vec<_>>()
        })
        .collect();
    Bag::from_rows(["x"], rows)
}

#[tokio::main]
async fn main() {
    let l = Bag::new(
        ["x"],
        vec![
            ["a".into()],
            ["a".into()],
            ["a".into()],
            ["b".into()],
            ["c".into()],
        ],
    );
    let r = Bag::new(
        ["x"],
        vec![["a".into()], ["b".into()], ["b".into()], ["d".into()]],
    );
    println!("L = {l}");
    println!("R = {r}\n");
    for (name, b) in [
        ("UNION ALL", l.union_all(&r)),
        ("UNION", l.union(&r)),
        ("INTERSECT ALL", l.intersect_all(&r)),
        ("INTERSECT", l.intersect(&r)),
        ("EXCEPT ALL", l.except_all(&r)),
        ("EXCEPT", l.except(&r)),
    ] {
        println!("{name:<14} {b}");
    }

    println!("\n== The same six in DataFusion, checked against the standard ==\n");
    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE l (x VARCHAR) AS VALUES ('a'), ('a'), ('a'), ('b'), ('c')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE r (x VARCHAR) AS VALUES ('a'), ('b'), ('b'), ('d')",
    )
    .await;
    let expected = [
        ("UNION ALL", l.union_all(&r)),
        ("UNION", l.union(&r)),
        ("INTERSECT ALL", l.intersect_all(&r)),
        ("INTERSECT", l.intersect(&r)),
        ("EXCEPT ALL", l.except_all(&r)),
        ("EXCEPT", l.except(&r)),
    ];
    for (op, want) in &expected {
        let got_bag = bag_of(&ctx, &format!("SELECT x FROM l {op} SELECT x FROM r")).await;
        let verdict = if &got_bag == want {
            "matches"
        } else {
            "DIFFERS from the standard"
        };
        if ["INTERSECT ALL", "EXCEPT ALL"].contains(op) {
            // DataFusion 55.2.0: keep this expectation explicit until an engine upgrade.
            assert_ne!(
                &got_bag, want,
                "upstream bug fixed: update the pinned lesson/expectation"
            );
            let pinned = if *op == "INTERSECT ALL" {
                l.intersect(&r)
                    .rows()
                    .into_iter()
                    .flat_map(|row| std::iter::repeat_n(row.clone(), l.count(&row)))
                    .collect()
            } else {
                l.except(&r).rows()
            };
            assert_eq!(got_bag, Bag::from_rows(["x"], pinned));
        } else {
            assert_eq!(&got_bag, want, "unexpected SQL result");
        }
        println!("{op:<14} {got_bag}   {verdict}");
    }
    for op in ["INTERSECT ALL", "EXCEPT ALL"] {
        let df = ctx
            .sql(&format!("SELECT x FROM l {op} SELECT x FROM r"))
            .await
            .unwrap();
        println!(
            "\n{op} is planned as:\n{}",
            df.into_optimized_plan().unwrap().display_indent()
        );
    }

    println!("\n== The set versions, as DataFusion plans them ==\n");
    for op in ["UNION", "INTERSECT", "EXCEPT"] {
        let df = ctx
            .sql(&format!("SELECT x FROM l {op} SELECT x FROM r"))
            .await
            .unwrap();
        println!(
            "{op}:\n{}\n",
            df.into_optimized_plan().unwrap().display_indent()
        );
    }

    println!("== Numbering the copies makes semi and anti joins count correctly ==\n");
    // Copy k of a value on the left survives INTERSECT ALL only if the right has a copy k too,
    // so the k-th copy survives exactly when k <= min(m, n). EXCEPT ALL keeps the rest.
    let numbered = |t: &str| format!("SELECT x, row_number() OVER (PARTITION BY x) AS k FROM {t}");
    let fixed = [
        (
            "INTERSECT ALL",
            format!(
                "SELECT a.x FROM ({}) a WHERE EXISTS (SELECT 1 FROM ({}) b WHERE (b.x IS NOT DISTINCT FROM a.x) AND b.k = a.k)",
                numbered("l"),
                numbered("r")
            ),
            l.intersect_all(&r),
        ),
        (
            "EXCEPT ALL",
            format!(
                "SELECT a.x FROM ({}) a WHERE NOT EXISTS (SELECT 1 FROM ({}) b WHERE (b.x IS NOT DISTINCT FROM a.x) AND b.k = a.k)",
                numbered("l"),
                numbered("r")
            ),
            l.except_all(&r),
        ),
    ];
    for (op, sql, want) in &fixed {
        let got = bag_of(&ctx, sql).await;
        let verdict = if &got == want {
            "matches"
        } else {
            "DIFFERS from the standard"
        };
        assert_eq!(
            &got, want,
            "numbered-copy workaround must match SQL bag semantics"
        );
        println!("{op:<14} rewritten: {got}   {verdict}");
    }

    println!("\n== Set operations match columns by position ==\n");
    exec(
        &ctx,
        "CREATE TABLE staff (first_name VARCHAR, last_name VARCHAR) AS VALUES ('Ada', 'Lovelace')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE contractors (last_name VARCHAR, first_name VARCHAR) AS VALUES ('Hopper', 'Grace')",
    )
    .await;
    show(
        &ctx,
        "SELECT * FROM staff UNION ALL SELECT * FROM contractors",
    )
    .await;
    show(
        &ctx,
        "SELECT * FROM staff UNION ALL BY NAME SELECT * FROM contractors",
    )
    .await;
}
