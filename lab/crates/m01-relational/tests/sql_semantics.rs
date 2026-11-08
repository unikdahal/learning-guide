#![cfg(feature = "sql")]
use datafusion::{arrow::array::Array, arrow::util::display::array_value_to_string, prelude::*};
use m01_relational::logic::Truth;

async fn rows(ctx: &SessionContext, sql: &str) -> Vec<Vec<Option<String>>> {
    let mut out = vec![];
    for b in ctx.sql(sql).await.unwrap().collect().await.unwrap() {
        for i in 0..b.num_rows() {
            out.push(
                b.columns()
                    .iter()
                    .map(|a| {
                        if a.is_null(i) {
                            None
                        } else {
                            Some(array_value_to_string(a, i).unwrap())
                        }
                    })
                    .collect(),
            );
        }
    }
    out.sort();
    out
}
fn expected(rows: &[&[Option<&str>]]) -> Vec<Vec<Option<String>>> {
    let mut out: Vec<_> = rows
        .iter()
        .map(|r| r.iter().map(|v| v.map(str::to_owned)).collect::<Vec<_>>())
        .collect();
    out.sort();
    out
}
#[tokio::test]
async fn duplicate_free_union_requires_both_internal_and_cross_input_uniqueness() {
    let c = SessionContext::new();
    assert_eq!(
        rows(&c, "SELECT x FROM (VALUES (1),(1)) l(x) UNION ALL SELECT 2")
            .await
            .len(),
        3
    );
    assert_eq!(
        rows(&c, "SELECT x FROM (VALUES (1),(1)) l(x) UNION SELECT 2")
            .await
            .len(),
        2
    );
}
#[tokio::test]
async fn numbered_copies_match_nullable_composite_rows() {
    let c = SessionContext::new();
    c.sql("CREATE TABLE l(x INT,y VARCHAR) AS VALUES (NULL,'a'),(NULL,'a'),(1,NULL),(1,NULL),(NULL,NULL)").await.unwrap().collect().await.unwrap();
    c.sql("CREATE TABLE r(x INT,y VARCHAR) AS VALUES (NULL,'a'),(1,NULL),(NULL,NULL)")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    for (op, want) in [
        (
            "EXISTS",
            expected(&[&[None, Some("a")], &[Some("1"), None], &[None, None]]),
        ),
        (
            "NOT EXISTS",
            expected(&[&[None, Some("a")], &[Some("1"), None]]),
        ),
    ] {
        let sql = format!(
            "SELECT a.x,a.y FROM (SELECT x,y,row_number() OVER(PARTITION BY x,y) AS k FROM l) a WHERE {op} (SELECT 1 FROM (SELECT x,y,row_number() OVER(PARTITION BY x,y) AS k FROM r) b WHERE (b.x IS NOT DISTINCT FROM a.x) AND (b.y IS NOT DISTINCT FROM a.y) AND b.k=a.k)"
        );
        assert_eq!(rows(&c, &sql).await, want);
    }
}
#[tokio::test]
async fn empty_ungrouped_aggregate_can_be_removed_by_having() {
    let c = SessionContext::new();
    assert_eq!(
        rows(
            &c,
            "SELECT count(*),sum(x) FROM (VALUES (1)) t(x) WHERE FALSE"
        )
        .await,
        expected(&[&[Some("0"), None]])
    );
    assert!(
        rows(
            &c,
            "SELECT sum(x) FROM (VALUES (1)) t(x) WHERE FALSE HAVING sum(x)>200"
        )
        .await
        .is_empty()
    );
    assert!(
        rows(
            &c,
            "SELECT x,count(*) FROM (VALUES (1)) t(x) WHERE FALSE GROUP BY x"
        )
        .await
        .is_empty()
    );
    assert_eq!(
        rows(
            &c,
            "SELECT count(*),count(x),sum(x) FROM (VALUES (CAST(NULL AS INT)),(NULL)) t(x)"
        )
        .await,
        expected(&[&[Some("2"), Some("0"), None]])
    );
}
#[tokio::test]
async fn truth_tables_agree_with_sql_including_empty_not_in() {
    let c = SessionContext::new();
    let lit = |v| match v {
        Truth::True => "TRUE",
        Truth::False => "FALSE",
        Truth::Unknown => "CAST(NULL AS BOOLEAN)",
    };
    let result = |v| match v {
        Truth::True => Some("true"),
        Truth::False => Some("false"),
        Truth::Unknown => None,
    };
    for a in Truth::ALL {
        for b in Truth::ALL {
            let sql = format!(
                "SELECT {} AND {},{} OR {},NOT {}",
                lit(a),
                lit(b),
                lit(a),
                lit(b),
                lit(a)
            );
            assert_eq!(
                rows(&c, &sql).await,
                expected(&[&[result(a.and(b)), result(a.or(b)), result(a.not())]])
            );
        }
    }
    assert_eq!(
        rows(
            &c,
            "SELECT x FROM (VALUES (CAST(NULL AS INT))) t(x) WHERE x NOT IN (SELECT 1 WHERE FALSE)"
        )
        .await,
        expected(&[&[None]])
    );
}
#[tokio::test]
async fn pinned_datafusion_55_2_all_bugs_are_explicit_expectations() {
    let c = SessionContext::new();
    let left = "SELECT x FROM (VALUES ('a'),('a'),('a'),('b'),('c')) l(x)";
    let right = "SELECT x FROM (VALUES ('a'),('b'),('b'),('d')) r(x)";
    assert_eq!(
        rows(&c, &format!("{left} INTERSECT ALL {right}")).await,
        expected(&[&[Some("a")], &[Some("a")], &[Some("a")], &[Some("b")]]),
        "If the upstream bug is fixed, update this version-pinned lesson and expectation."
    );
    assert_eq!(
        rows(&c, &format!("{left} EXCEPT ALL {right}")).await,
        expected(&[&[Some("c")]])
    );
}

#[tokio::test]
async fn natural_and_outer_joins_preserve_keys_and_null_extension() {
    let c = SessionContext::new();
    let q = "SELECT * FROM (VALUES (1,'Asha'),(2,'Bruno')) customer(cust_id,name) NATURAL JOIN (VALUES (101,1),(102,1)) orders(order_id,cust_id)";
    let b = c.sql(q).await.unwrap().collect().await.unwrap();
    assert_eq!(
        b[0].schema()
            .fields()
            .iter()
            .map(|f| f.name().as_str())
            .collect::<Vec<_>>(),
        vec!["cust_id", "name", "order_id"]
    );
    assert_eq!(
        rows(&c, q).await,
        expected(&[
            &[Some("1"), Some("Asha"), Some("101")],
            &[Some("1"), Some("Asha"), Some("102")]
        ])
    );
    assert_eq!(rows(&c,"SELECT c.id,count(*),count(o.id) FROM (VALUES (1),(2)) c(id) LEFT JOIN (VALUES (10,1),(11,1)) o(id,cust_id) ON c.id=o.cust_id GROUP BY c.id").await,expected(&[&[Some("1"),Some("2"),Some("2")],&[Some("2"),Some("1"),Some("0")]]));
}

#[tokio::test]
async fn ranking_keeps_ties_and_total_order_makes_keyset_pages_stable() {
    let c = SessionContext::new();
    let input = "(VALUES (1,100),(2,90),(3,90),(4,80)) t(id,amount)";
    assert_eq!(
        rows(
            &c,
            &format!("SELECT id FROM {input} QUALIFY rank() OVER(ORDER BY amount DESC)<=2")
        )
        .await,
        expected(&[&[Some("1")], &[Some("2")], &[Some("3")]])
    );
    assert_eq!(rows(&c,&format!("SELECT id FROM {input} WHERE amount<90 OR (amount=90 AND id>2) ORDER BY amount DESC,id LIMIT 2")).await,expected(&[&[Some("3")],&[Some("4")]]));
    // A window expression in WHERE is a deliberate rejection, not a successful demonstration.
    let rejected = c
        .sql(&format!(
            "SELECT id FROM {input} WHERE rank() OVER(ORDER BY amount DESC)<=2"
        ))
        .await;
    match rejected {
        Err(_) => {}
        Ok(df) => assert!(
            df.collect().await.is_err(),
            "window expressions cannot run in WHERE"
        ),
    }
}
