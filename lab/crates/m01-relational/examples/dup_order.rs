//! Article: "Duplicate-sensitive and order-sensitive operators".
//! Set laws that fail for bags, aggregates that do or don't care about duplicates, and operators
//! whose result depends on row order, checked with the bag engine and in DataFusion.
//!
//!     cargo run --release -p m01-relational --example dup_order

use datafusion::prelude::*;
use m01_relational::bag::Bag;
use m01_relational::sql::{exec, show};

fn bag(xs: &[&str]) -> Bag {
    Bag::new(["x"], xs.iter().map(|x| [(*x).into()]).collect())
}

fn law(name: &str, lhs: Bag, rhs: Bag) {
    let verdict = if lhs == rhs { "holds" } else { "FAILS" };
    println!("{name}\n    left  {lhs}\n    right {rhs}\n    {verdict}\n");
}

#[tokio::main]
async fn main() {
    let r = bag(&["a"]);
    let s = bag(&["a"]);
    let t = bag(&["a"]);
    println!("R = {r}   S = {s}   T = {t}\n");

    println!("== Set laws, checked with set operators (UNION, INTERSECT, EXCEPT) ==\n");
    law("R ∪ R = R", r.union(&r), r.clone());
    law(
        "R ∩ (S ∪ T) = (R ∩ S) ∪ (R ∩ T)",
        r.intersect(&s.union(&t)),
        r.intersect(&s).union(&r.intersect(&t)),
    );
    law(
        "(R ∪ S) − T = (R − T) ∪ (S − T)",
        r.union(&s).except(&t),
        r.except(&t).union(&s.except(&t)),
    );

    println!("== The same laws with bag operators (UNION ALL, INTERSECT ALL, EXCEPT ALL) ==\n");
    law("R ∪ R = R", r.union_all(&r), r.clone());
    law(
        "R ∩ (S ∪ T) = (R ∩ S) ∪ (R ∩ T)",
        r.intersect_all(&s.union_all(&t)),
        r.intersect_all(&s).union_all(&r.intersect_all(&t)),
    );
    law(
        "(R ∪ S) − T = (R − T) ∪ (S − T)",
        r.union_all(&s).except_all(&t),
        r.except_all(&t).union_all(&s.except_all(&t)),
    );
    let big = bag(&["a", "a", "b"]);
    law(
        "δ(R ∪ S) = δ(R) ∪ δ(S), with R = {a×2 b×1}, S = {a×1}",
        big.union_all(&s).distinct(),
        big.distinct().union_all(&s.distinct()),
    );
    law(
        "δ(R ∪ S) = δ(δ(R) ∪ δ(S))",
        big.union_all(&s).distinct(),
        big.distinct().union_all(&s.distinct()).distinct(),
    );

    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE t (g VARCHAR, x INT) AS VALUES \
         ('p', 10), ('p', 10), ('p', 30), ('q', 5), ('q', 5), ('q', 5)",
    )
    .await;

    println!("== Which aggregates care about duplicates? ==\n");
    show(
        &ctx,
        "SELECT g, min(x), max(x), count(DISTINCT x), bool_or(x > 20), \
                sum(x), count(x), avg(x) FROM t GROUP BY g ORDER BY g",
    )
    .await;
    show(
        &ctx,
        "SELECT g, min(x), max(x), count(DISTINCT x), bool_or(x > 20), \
                sum(x), count(x), avg(x) FROM (SELECT DISTINCT g, x FROM t) GROUP BY g ORDER BY g",
    )
    .await;
    for sql in [
        "SELECT max(x) FROM (SELECT DISTINCT x FROM t)",
        "SELECT sum(x) FROM (SELECT DISTINCT x FROM t)",
    ] {
        let plan = ctx.sql(sql).await.unwrap().into_optimized_plan().unwrap();
        println!("{sql}\n{}\n", plan.display_indent());
    }

    println!("== A join multiplies duplicates; a semi join doesn't ==\n");
    exec(
        &ctx,
        "CREATE TABLE customer (id INT, name VARCHAR) AS VALUES (1, 'Asha'), (2, 'Bruno'), (3, 'Chen')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE orders (order_id INT, cust_id INT) AS VALUES (101, 1), (102, 3), (103, 1)",
    )
    .await;
    show(
        &ctx,
        "SELECT c.name FROM customer c JOIN orders o ON o.cust_id = c.id ORDER BY c.name",
    )
    .await;
    show(
        &ctx,
        "SELECT c.name FROM customer c WHERE EXISTS (SELECT 1 FROM orders o WHERE o.cust_id = c.id) ORDER BY c.name",
    )
    .await;
    show(
        &ctx,
        "SELECT DISTINCT c.name FROM customer c JOIN orders o ON o.cust_id = c.id ORDER BY c.name",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE people (name VARCHAR) AS VALUES ('Asha'), ('Asha'), ('Chen')",
    )
    .await;
    println!("Two different people named Asha, no key:\n");
    show(
        &ctx,
        "SELECT p.name FROM people p WHERE EXISTS (SELECT 1 FROM customer c WHERE c.name = p.name) ORDER BY p.name",
    )
    .await;
    show(
        &ctx,
        "SELECT DISTINCT p.name FROM people p JOIN customer c ON c.name = p.name ORDER BY p.name",
    )
    .await;

    println!("== Order-sensitive operators ==\n");
    exec(
        &ctx,
        "CREATE TABLE ev (k INT, v VARCHAR) AS VALUES (3, 'c'), (1, 'a'), (2, 'b'), (5, 'e'), (4, 'd')",
    )
    .await;
    show(
        &ctx,
        "SELECT string_agg(v, ',') AS no_order, string_agg(v, ',' ORDER BY k) AS ordered, \
                array_agg(k) AS arr, array_agg(k ORDER BY k DESC) AS arr_desc FROM ev",
    )
    .await;
    show(
        &ctx,
        "SELECT string_agg(v, ',') AS alone, array_agg(k) AS arr_alone FROM ev",
    )
    .await;
    for sql in [
        "SELECT count(*) FROM (SELECT * FROM ev ORDER BY k)",
        "SELECT k FROM (SELECT k FROM ev ORDER BY k) WHERE k > 2",
        "SELECT k FROM (SELECT k FROM ev ORDER BY k LIMIT 3) WHERE k > 1",
    ] {
        let df = ctx.sql(sql).await.unwrap();
        let plan = df.into_optimized_plan().unwrap();
        println!("{sql}\n{}\n", plan.display_indent());
    }

    println!("== Float addition depends on order ==\n");
    let xs: Vec<f64> = (1..=1_000_000).map(|i| 1.0 / i as f64).collect();
    let forward: f64 = xs.iter().sum();
    let backward: f64 = xs.iter().rev().sum();
    let pairwise = {
        fn pw(v: &[f64]) -> f64 {
            if v.len() <= 8 {
                v.iter().sum()
            } else {
                let (a, b) = v.split_at(v.len() / 2);
                pw(a) + pw(b)
            }
        }
        pw(&xs)
    };
    println!("sum of 1/i for i = 1..1,000,000");
    println!("  forward   {forward:.17}");
    println!("  backward  {backward:.17}");
    println!("  pairwise  {pairwise:.17}");
    println!("  forward == backward: {}", forward == backward);
}
