//! Article: "Equivalence rules of relational algebra".
//! Tests each rewrite rule on hundreds of random relations with the tiny engine, including the
//! versions that break a side condition, then shows DataFusion's optimizer applying the rules.
//!
//!     cargo run --release -p m01-relational --example equivalences

use datafusion::prelude::*;
use labkit::rng::Rng;
use m01_relational::Relation;
use m01_relational::sql::{exec, show};

const TRIALS: usize = 500;

/// A random relation r(a, b) or s(b, c) with values from a small domain, so that joins match
/// and duplicates collide often: the cases where wrong rules get caught.
fn random(rng: &mut Rng, attrs: [&str; 2]) -> Relation {
    let n = rng.below(7) as usize;
    let rows = (0..n)
        .map(|_| [(rng.below(4) as i64).into(), (rng.below(4) as i64).into()])
        .collect();
    Relation::new(attrs, rows)
}

/// Checks `law` on TRIALS random triples (R(a,b), S(b,c), T(a,b)) and reports how often it held.
/// Prints the first counterexample, if any.
fn check(name: &str, law: impl Fn(&Relation, &Relation, &Relation) -> (Relation, Relation)) {
    let mut rng = Rng::new(2026);
    let mut held = 0;
    let mut witness = None;
    for _ in 0..TRIALS {
        let r = random(&mut rng, ["a", "b"]);
        let s = random(&mut rng, ["b", "c"]);
        let t = random(&mut rng, ["a", "b"]);
        let (lhs, rhs) = law(&r, &s, &t);
        if lhs == rhs {
            held += 1;
        } else if witness.is_none() {
            witness = Some((r, s, t, lhs, rhs));
        }
    }
    println!("{name:<58} held in {held:>3} of {TRIALS}");
    if let Some((r, s, t, lhs, rhs)) = witness {
        println!("    counterexample:\n    R = {r}\n    S = {s}\n    T = {t}");
        println!("    left side  = {lhs}\n    right side = {rhs}");
    }
}

#[tokio::main]
async fn main() {
    println!("== Rules that always hold ==\n");
    check("σ cascade: σ p∧q (R) = σ p (σ q (R))", |r, _, _| {
        (
            r.select(|t| t.int("a") > 0 && t.int("b") < 3),
            r.select(|t| t.int("b") < 3).select(|t| t.int("a") > 0),
        )
    });
    check(
        "σ commutes: σ p (σ q (R)) = σ q (σ p (R))",
        |r, _, _| {
            (
                r.select(|t| t.int("b") < 3).select(|t| t.int("a") > 0),
                r.select(|t| t.int("a") > 0).select(|t| t.int("b") < 3),
            )
        },
    );
    check("join commutes: R ⋈ S = S ⋈ R", |r, s, _| {
        (r.natural_join(s), s.natural_join(r))
    });
    check(
        "join associates: (R ⋈ S) ⋈ T = R ⋈ (S ⋈ T)",
        |r, s, t| {
            let t = t.rename("a", "c").rename("b", "d");
            (
                r.natural_join(s).natural_join(&t),
                r.natural_join(&s.natural_join(&t)),
            )
        },
    );
    check(
        "σ pushdown: σ a>1 (R ⋈ S) = σ a>1 (R) ⋈ S",
        |r, s, _| {
            (
                r.natural_join(s).select(|t| t.int("a") > 1),
                r.select(|t| t.int("a") > 1).natural_join(s),
            )
        },
    );
    check(
        "σ pushdown, other side: σ c>1 (R ⋈ S) = R ⋈ σ c>1 (S)",
        |r, s, _| {
            (
                r.natural_join(s).select(|t| t.int("c") > 1),
                r.natural_join(&s.select(|t| t.int("c") > 1)),
            )
        },
    );
    check(
        "σ over ∪: σ p (R ∪ T) = σ p (R) ∪ σ p (T)",
        |r, _, t| {
            (
                r.union(t).select(|x| x.int("a") > 1),
                r.select(|x| x.int("a") > 1)
                    .union(&t.select(|x| x.int("a") > 1)),
            )
        },
    );
    check("σ over −: σ p (R − T) = σ p (R) − T", |r, _, t| {
        (
            r.difference(t).select(|x| x.int("a") > 1),
            r.select(|x| x.int("a") > 1).difference(t),
        )
    });
    check("π cascade: π a (π a,b (R)) = π a (R)", |r, _, _| {
        (r.project(&["a", "b"]).project(&["a"]), r.project(&["a"]))
    });
    check(
        "π pushdown, keeping the join key: π a,c (R ⋈ S) = π a,c (π a,b (R) ⋈ S)",
        |r, s, _| {
            (
                r.natural_join(s).project(&["a", "c"]),
                r.project(&["a", "b"]).natural_join(s).project(&["a", "c"]),
            )
        },
    );

    println!("\n== The same rules with a side condition broken ==\n");
    check(
        "π pushdown that drops the join key: π a,c (R ⋈ S) vs π a (R) × π c (S)",
        |r, s, _| {
            (
                r.natural_join(s).project(&["a", "c"]),
                r.project(&["a"]).product(&s.project(&["c"])),
            )
        },
    );
    check(
        "π over −: π a (R − T) vs π a (R) − π a (T)",
        |r, _, t| {
            (
                r.difference(t).project(&["a"]),
                r.project(&["a"]).difference(&t.project(&["a"])),
            )
        },
    );
    check(
        "π over ∪ (holds): π a (R ∪ T) = π a (R) ∪ π a (T)",
        |r, _, t| {
            (
                r.union(t).project(&["a"]),
                r.project(&["a"]).union(&t.project(&["a"])),
            )
        },
    );

    println!("\n== DataFusion: outer joins limit where a filter may go ==\n");
    let ctx = SessionContext::new();
    exec(
        &ctx,
        "CREATE TABLE customer (id INT, name VARCHAR, region VARCHAR) AS VALUES \
         (1, 'Asha', 'ASIA'), (2, 'Bruno', 'EUROPE'), (3, 'Chen', 'ASIA'), \
         (4, 'Dara', 'AFRICA'), (5, 'Emeka', 'AFRICA'), (6, 'Farah', 'EUROPE')",
    )
    .await;
    exec(
        &ctx,
        "CREATE TABLE orders (order_id INT, cust_id INT, amount INT) AS VALUES \
         (101, 1, 250), (102, 3, 40), (103, 1, 90), (104, 4, 610), (105, 6, 75), (106, 9, 120)",
    )
    .await;
    show(
        &ctx,
        "SELECT c.name, o.amount FROM customer c LEFT JOIN orders o ON o.cust_id = c.id \
         WHERE o.amount > 80 ORDER BY c.name, o.amount",
    )
    .await;
    show(
        &ctx,
        "SELECT c.name, o.amount FROM customer c \
         LEFT JOIN (SELECT * FROM orders WHERE amount > 80) o ON o.cust_id = c.id ORDER BY c.name, o.amount",
    )
    .await;
    show(
        &ctx,
        "SELECT c.name, o.amount FROM customer c LEFT JOIN orders o ON o.cust_id = c.id AND o.amount > 80 \
         ORDER BY c.name, o.amount",
    )
    .await;
    for sql in [
        "SELECT c.name, o.amount FROM customer c LEFT JOIN orders o ON o.cust_id = c.id WHERE o.amount > 80",
        "SELECT c.name, o.amount FROM customer c LEFT JOIN orders o ON o.cust_id = c.id WHERE c.region = 'ASIA'",
        "SELECT c.name, o.amount FROM customer c LEFT JOIN orders o ON o.cust_id = c.id WHERE o.amount IS NULL",
    ] {
        let plan = ctx.sql(sql).await.unwrap().into_optimized_plan().unwrap();
        println!("{sql}\n\n{}\n", plan.display_indent());
    }

    println!("== DataFusion's optimizer, rule by rule ==\n");
    let sql = "SELECT c.name, o.amount FROM customer c, orders o \
               WHERE c.id = o.cust_id AND c.region = 'ASIA' AND o.amount > 80";
    let batches = ctx
        .sql(&format!("EXPLAIN VERBOSE {sql}"))
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    println!("{sql}\n");
    let mut last = String::new();
    for b in &batches {
        let kind = b
            .column(0)
            .as_any()
            .downcast_ref::<datafusion::arrow::array::StringArray>();
        let text = b
            .column(1)
            .as_any()
            .downcast_ref::<datafusion::arrow::array::StringArray>();
        let (Some(kind), Some(text)) = (kind, text) else {
            println!("(unexpected EXPLAIN schema: {:?})", b.schema());
            return;
        };
        for i in 0..b.num_rows() {
            let k = kind.value(i);
            let t = text.value(i);
            // Logical stages only; skip passes that left the plan unchanged.
            let logical = k.starts_with("initial_logical_plan")
                || k.starts_with("logical_plan after")
                || k == "logical_plan";
            if !logical || t == "SAME TEXT AS ABOVE" || t == last {
                continue;
            }
            println!("--- {k}\n{t}\n");
            last = t.to_string();
        }
    }
}
