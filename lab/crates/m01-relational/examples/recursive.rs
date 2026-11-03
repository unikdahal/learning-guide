//! Article: "CTEs and recursive queries".
//! A recursive CTE as fixpoint iteration: run by hand in Rust, then in DataFusion, with
//! termination, cycles, and UNION vs UNION ALL.
//!
//!     cargo run --release -p m01-relational --example recursive

use std::collections::BTreeSet;

use datafusion::prelude::*;
use m01_relational::sql::{exec, show};

const EMPLOYEES: [(i32, &str, Option<i32>); 8] = [
    (1, "Mira", None),
    (2, "Ravi", Some(1)),
    (3, "Sana", Some(1)),
    (4, "Tomas", Some(2)),
    (5, "Uma", Some(2)),
    (6, "Vik", Some(3)),
    (7, "Wen", Some(4)),
    (8, "Xia", Some(7)),
];

/// What `WITH RECURSIVE ... UNION ALL` computes: start with the anchor rows, then repeatedly
/// apply the recursive step to the rows produced in the previous iteration only (the working
/// table), appending its output, until a step produces nothing.
fn fixpoint_by_hand() {
    let name = |id: i32| EMPLOYEES.iter().find(|e| e.0 == id).unwrap().1;
    let mut result: Vec<(i32, usize)> = Vec::new();
    let mut working: Vec<(i32, usize)> = vec![(1, 0)]; // anchor: the CEO, depth 0
    let mut iteration = 0;
    while !working.is_empty() {
        let names: Vec<&str> = working.iter().map(|&(id, _)| name(id)).collect();
        println!("iteration {iteration}: working table = {names:?}");
        result.extend(&working);
        // recursive step: employees whose manager is in the working table
        let mut next = Vec::new();
        let mut join_probes = 0;
        for &(boss, depth) in &working {
            for &(id, _, manager) in &EMPLOYEES {
                join_probes += 1;
                if manager == Some(boss) {
                    next.push((id, depth + 1));
                }
            }
        }
        println!(
            "             step compared {join_probes} pairs, produced {}",
            next.len()
        );
        working = next;
        iteration += 1;
    }
    println!(
        "iteration {iteration}: working table empty, stop. {} rows\n",
        result.len()
    );
}

/// Transitive closure of a chain a0 -> a1 -> ... -> a(n-1), computed naively (each round joins
/// the whole result so far) and semi-naively (each round joins only the new pairs). Both reach
/// the same fixpoint; the naive one re-derives old facts every round.
fn naive_vs_semi_naive(n: i32) {
    let edges: Vec<(i32, i32)> = (0..n - 1).map(|i| (i, i + 1)).collect();

    let mut all: BTreeSet<(i32, i32)> = edges.iter().copied().collect();
    let (mut rounds, mut derived) = (0, 0);
    loop {
        rounds += 1;
        let mut new = BTreeSet::new();
        for &(a, b) in &all {
            for &(c, d) in &edges {
                if b == c {
                    derived += 1;
                    new.insert((a, d));
                }
            }
        }
        let before = all.len();
        all.extend(new);
        if all.len() == before {
            break;
        }
    }
    println!(
        "naive:      {rounds} rounds, {derived} pairs derived, {} in closure",
        all.len()
    );

    let mut all: BTreeSet<(i32, i32)> = edges.iter().copied().collect();
    let mut delta = all.clone();
    let (mut rounds, mut derived) = (0, 0);
    while !delta.is_empty() {
        rounds += 1;
        let mut new = BTreeSet::new();
        for &(a, b) in &delta {
            for &(c, d) in &edges {
                if b == c {
                    derived += 1;
                    if !all.contains(&(a, d)) {
                        new.insert((a, d));
                    }
                }
            }
        }
        all.extend(new.iter().copied());
        delta = new;
    }
    println!(
        "semi-naive: {rounds} rounds, {derived} pairs derived, {} in closure\n",
        all.len()
    );
}

#[tokio::main]
async fn main() {
    println!("== Fixpoint iteration by hand ==\n");
    fixpoint_by_hand();

    println!("== Naive vs semi-naive evaluation, a chain of 200 nodes ==\n");
    naive_vs_semi_naive(200);

    let ctx = SessionContext::new();
    let rows: Vec<String> = EMPLOYEES
        .iter()
        .map(|(id, name, m)| {
            format!(
                "({id}, '{name}', {})",
                m.map_or("NULL".to_string(), |m| m.to_string())
            )
        })
        .collect();
    exec(
        &ctx,
        &format!(
            "CREATE TABLE employee (id INT, name VARCHAR, manager_id INT) AS VALUES {}",
            rows.join(", ")
        ),
    )
    .await;

    println!("== A plain CTE is a named subquery ==\n");
    show(
        &ctx,
        "WITH managers AS (SELECT DISTINCT manager_id AS id FROM employee WHERE manager_id IS NOT NULL) \
         SELECT e.name FROM employee e JOIN managers m ON e.id = m.id ORDER BY e.id",
    )
    .await;
    let df = ctx
        .sql(
            "WITH big AS (SELECT id FROM employee WHERE id > 4) \
             SELECT a.id, b.id FROM big a JOIN big b ON a.id + 1 = b.id",
        )
        .await
        .unwrap();
    println!(
        "A CTE referenced twice, optimized plan:\n\n{}\n",
        df.into_optimized_plan().unwrap().display_indent()
    );

    println!("== Counting with recursion ==\n");
    show(
        &ctx,
        "WITH RECURSIVE n(i) AS (SELECT 1 UNION ALL SELECT i + 1 FROM n WHERE i < 5) SELECT i FROM n",
    )
    .await;

    println!("== Walking a hierarchy ==\n");
    show(
        &ctx,
        "WITH RECURSIVE chain(id, name, depth, path) AS ( \
           SELECT id, name, 0 AS depth, name AS path FROM employee WHERE manager_id IS NULL \
           UNION ALL \
           SELECT e.id, e.name, c.depth + 1, concat(c.path, ' > ', e.name) \
           FROM employee e JOIN chain c ON e.manager_id = c.id \
         ) SELECT depth, name, path FROM chain ORDER BY depth, id",
    )
    .await;
    let df = ctx
        .sql(
            "WITH RECURSIVE chain(id, depth) AS ( \
               SELECT id, 0 FROM employee WHERE manager_id IS NULL \
               UNION ALL \
               SELECT e.id, c.depth + 1 FROM employee e JOIN chain c ON e.manager_id = c.id \
             ) SELECT * FROM chain",
        )
        .await
        .unwrap();
    println!(
        "Optimized logical plan:\n\n{}\n",
        df.into_optimized_plan().unwrap().display_indent()
    );

    println!("== Cycles: UNION ALL vs UNION ==\n");
    exec(
        &ctx,
        "CREATE TABLE flight (src VARCHAR, dst VARCHAR) AS VALUES \
         ('KTM', 'DEL'), ('DEL', 'BOM'), ('BOM', 'KTM'), ('DEL', 'DXB')",
    )
    .await;
    show(
        &ctx,
        "WITH RECURSIVE reach(city, hops) AS ( \
           SELECT 'KTM', 0 \
           UNION ALL \
           SELECT f.dst, r.hops + 1 FROM flight f JOIN reach r ON f.src = r.city WHERE r.hops < 6 \
         ) SELECT city, count(*) AS times_reached, min(hops) AS fewest_hops FROM reach GROUP BY city ORDER BY fewest_hops",
    )
    .await;
    show(
        &ctx,
        "WITH RECURSIVE reach(city) AS ( \
           SELECT 'KTM' \
           UNION \
           SELECT f.dst FROM flight f JOIN reach r ON f.src = r.city \
         ) SELECT city FROM reach ORDER BY city",
    )
    .await;
    show(
        &ctx,
        "WITH RECURSIVE reach(city, hops) AS ( \
           SELECT 'KTM', 0 \
           UNION \
           SELECT f.dst, r.hops + 1 FROM flight f JOIN reach r ON f.src = r.city WHERE r.hops < 6 \
         ) SELECT count(*) AS rows_with_hops FROM reach",
    )
    .await;
    println!("UNION ALL on the cycle with no depth limit, stopped by LIMIT:\n");
    show(
        &ctx,
        "WITH RECURSIVE reach(city) AS ( \
           SELECT 'KTM' \
           UNION ALL \
           SELECT f.dst FROM flight f JOIN reach r ON f.src = r.city \
         ) SELECT city FROM reach LIMIT 8",
    )
    .await;
}
