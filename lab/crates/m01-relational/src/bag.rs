//! Bags (multisets): what SQL tables actually are.
//!
//! A [`Bag`] stores each distinct tuple once, with a count of how many copies it has. The set
//! operations follow the SQL standard's rules, which PostgreSQL's documentation states as: with
//! `ALL`, a row with m copies on the left and n on the right appears m + n times in `UNION ALL`,
//! min(m, n) times in `INTERSECT ALL`, and max(m − n, 0) times in `EXCEPT ALL`. Without `ALL`,
//! each operation removes duplicates from its result.

use std::collections::BTreeMap;
use std::fmt;

use crate::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bag {
    attrs: Vec<String>,
    counts: BTreeMap<Vec<Value>, usize>,
}

impl Bag {
    pub fn new<const N: usize>(attrs: [&str; N], rows: Vec<[Value; N]>) -> Bag {
        let mut counts = BTreeMap::new();
        for r in rows {
            *counts.entry(r.to_vec()).or_insert(0) += 1;
        }
        Bag {
            attrs: attrs.iter().map(|a| a.to_string()).collect(),
            counts,
        }
    }

    /// Builds a bag from rows of any length, such as rows read back from a query.
    pub fn from_rows<const N: usize>(attrs: [&str; N], rows: Vec<Vec<Value>>) -> Bag {
        let mut counts = BTreeMap::new();
        for r in rows {
            assert_eq!(r.len(), N, "row has the wrong number of values");
            *counts.entry(r).or_insert(0) += 1;
        }
        Bag {
            attrs: attrs.iter().map(|a| a.to_string()).collect(),
            counts,
        }
    }

    /// Total number of rows, counting every copy.
    pub fn len(&self) -> usize {
        self.counts.values().sum()
    }

    pub fn is_empty(&self) -> bool {
        self.counts.is_empty()
    }

    /// How many copies of `row` the bag holds.
    pub fn count(&self, row: &[Value]) -> usize {
        self.counts.get(row).copied().unwrap_or(0)
    }

    fn combine(&self, other: &Bag, f: impl Fn(usize, usize) -> usize) -> Bag {
        assert_eq!(self.attrs.len(), other.attrs.len(), "not union compatible");
        let mut counts = BTreeMap::new();
        for row in self.counts.keys().chain(other.counts.keys()) {
            let n = f(self.count(row), other.count(row));
            if n > 0 {
                counts.insert(row.clone(), n);
            }
        }
        Bag {
            attrs: self.attrs.clone(),
            counts,
        }
    }

    /// δ: one copy of each distinct row. `SELECT DISTINCT`.
    pub fn distinct(&self) -> Bag {
        self.combine(self, |m, _| m.min(1))
    }

    /// `UNION ALL`: m + n copies.
    pub fn union_all(&self, other: &Bag) -> Bag {
        self.combine(other, |m, n| m + n)
    }

    /// `UNION`: one copy of every row present on either side.
    pub fn union(&self, other: &Bag) -> Bag {
        self.combine(other, |m, n| (m + n).min(1))
    }

    /// `INTERSECT ALL`: min(m, n) copies.
    pub fn intersect_all(&self, other: &Bag) -> Bag {
        self.combine(other, |m, n| m.min(n))
    }

    /// `INTERSECT`: one copy of every row present on both sides.
    pub fn intersect(&self, other: &Bag) -> Bag {
        self.combine(other, |m, n| m.min(n).min(1))
    }

    /// `EXCEPT ALL`: max(m − n, 0) copies.
    pub fn except_all(&self, other: &Bag) -> Bag {
        self.combine(other, |m, n| m.saturating_sub(n))
    }

    /// `EXCEPT`: one copy of every row present on the left and absent on the right.
    pub fn except(&self, other: &Bag) -> Bag {
        self.combine(other, |m, n| usize::from(m > 0 && n == 0))
    }

    /// Rows as a sorted list with repeats, the way a query would print them with ORDER BY.
    pub fn rows(&self) -> Vec<Vec<Value>> {
        self.counts
            .iter()
            .flat_map(|(r, &n)| std::iter::repeat_n(r.clone(), n))
            .collect()
    }
}

impl fmt::Display for Bag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self
            .counts
            .iter()
            .map(|(r, n)| {
                let vals: Vec<String> = r.iter().map(|v| v.to_string()).collect();
                format!("{}×{n}", vals.join(","))
            })
            .collect();
        write!(f, "{{{}}} ({} rows)", parts.join(" "), self.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn left() -> Bag {
        Bag::new(
            ["x"],
            vec![
                ["a".into()],
                ["a".into()],
                ["a".into()],
                ["b".into()],
                ["c".into()],
            ],
        )
    }
    fn right() -> Bag {
        Bag::new(
            ["x"],
            vec![["a".into()], ["b".into()], ["b".into()], ["d".into()]],
        )
    }
    fn n(b: &Bag, v: &str) -> usize {
        b.count(&[v.into()])
    }

    #[test]
    fn multiplicities_follow_the_standard() {
        let (l, r) = (left(), right());
        let ua = l.union_all(&r);
        assert_eq!(
            (n(&ua, "a"), n(&ua, "b"), n(&ua, "c"), n(&ua, "d")),
            (4, 3, 1, 1)
        );
        let ia = l.intersect_all(&r);
        assert_eq!((n(&ia, "a"), n(&ia, "b"), n(&ia, "c")), (1, 1, 0));
        let ea = l.except_all(&r);
        assert_eq!((n(&ea, "a"), n(&ea, "b"), n(&ea, "c")), (2, 0, 1));
    }

    #[test]
    fn set_versions_are_distinct_of_bag_versions() {
        let (l, r) = (left(), right());
        assert_eq!(l.union(&r), l.union_all(&r).distinct());
        assert_eq!(l.intersect(&r), l.intersect_all(&r).distinct());
    }

    #[test]
    fn except_is_not_distinct_of_except_all() {
        // EXCEPT removes 'a' entirely because it appears on the right at all; EXCEPT ALL keeps the
        // two copies of 'a' that the right side did not cancel.
        let (l, r) = (left(), right());
        assert_eq!(n(&l.except(&r), "a"), 0);
        assert_eq!(n(&l.except_all(&r).distinct(), "a"), 1);
    }

    #[test]
    fn union_all_is_not_idempotent_but_union_is() {
        let l = left();
        assert_eq!(l.union(&l), l.distinct());
        assert_eq!(l.union_all(&l).len(), 2 * l.len());
    }
}
