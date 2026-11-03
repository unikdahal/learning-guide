//! A tiny relational algebra engine, written to be read.
//!
//! A [`Relation`] is exactly what Codd defined: a heading (a set of attribute names) and a body
//! (a set of tuples). Because the body is a set, duplicates cannot exist and order does not matter.
//! Every operation returns a new relation, so operations compose: the algebra is closed.
//!
//! This is the reference semantics for Module 01. Real engines implement the same operators with
//! very different algorithms, and SQL relaxes the set rules to bags; the articles cover both.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A value in a tuple. Two types are enough to show every idea in this module.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Int(i64),
    Text(String),
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.to_string())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{v}"),
            Value::Text(v) => write!(f, "{v}"),
        }
    }
}

/// A view of one tuple that lets predicates look values up by attribute name.
pub struct Row<'a> {
    attrs: &'a [String],
    values: &'a [Value],
}

impl Row<'_> {
    pub fn get(&self, name: &str) -> &Value {
        let i = self
            .attrs
            .iter()
            .position(|a| a == name)
            .unwrap_or_else(|| panic!("no attribute named {name}"));
        &self.values[i]
    }

    pub fn int(&self, name: &str) -> i64 {
        match self.get(name) {
            Value::Int(v) => *v,
            other => panic!("{name} is {other:?}, not an integer"),
        }
    }

    pub fn text(&self, name: &str) -> &str {
        match self.get(name) {
            Value::Text(v) => v,
            other => panic!("{name} is {other:?}, not text"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Relation {
    /// Attribute names. Stored in a fixed order only so tuples can be stored compactly; no
    /// operation's meaning depends on this order.
    attrs: Vec<String>,
    /// The body: a set of tuples, each with one value per attribute in `attrs` order.
    tuples: BTreeSet<Vec<Value>>,
}

impl Relation {
    /// Builds a relation. Duplicate rows collapse, because the body is a set.
    pub fn new<const N: usize>(attrs: [&str; N], rows: Vec<[Value; N]>) -> Relation {
        Relation::from_parts(
            attrs.iter().map(|a| a.to_string()).collect(),
            rows.into_iter().map(|r| r.to_vec()).collect(),
        )
    }

    fn from_parts(attrs: Vec<String>, tuples: BTreeSet<Vec<Value>>) -> Relation {
        let distinct: BTreeSet<&String> = attrs.iter().collect();
        assert_eq!(
            distinct.len(),
            attrs.len(),
            "attribute names must be unique: {attrs:?}"
        );
        Relation { attrs, tuples }
    }

    /// Number of tuples (the relation's cardinality).
    pub fn len(&self) -> usize {
        self.tuples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tuples.is_empty()
    }

    /// Number of attributes (Codd's "degree").
    pub fn degree(&self) -> usize {
        self.attrs.len()
    }

    pub fn attrs(&self) -> &[String] {
        &self.attrs
    }

    fn index_of(&self, name: &str) -> usize {
        self.attrs
            .iter()
            .position(|a| a == name)
            .unwrap_or_else(|| panic!("no attribute named {name} in {:?}", self.attrs))
    }

    /// The same relation with attributes stored in alphabetical order. Two relations are equal
    /// exactly when their canonical forms are identical.
    fn canonical(&self) -> (Vec<String>, BTreeSet<Vec<Value>>) {
        let mut order: Vec<usize> = (0..self.attrs.len()).collect();
        order.sort_by(|&a, &b| self.attrs[a].cmp(&self.attrs[b]));
        let attrs = order.iter().map(|&i| self.attrs[i].clone()).collect();
        let tuples = self
            .tuples
            .iter()
            .map(|t| order.iter().map(|&i| t[i].clone()).collect())
            .collect();
        (attrs, tuples)
    }

    /// σ, selection (Codd: restriction). Keeps the tuples for which `pred` is true.
    pub fn select(&self, pred: impl Fn(&Row) -> bool) -> Relation {
        let tuples = self
            .tuples
            .iter()
            .filter(|t| {
                pred(&Row {
                    attrs: &self.attrs,
                    values: t,
                })
            })
            .cloned()
            .collect();
        Relation::from_parts(self.attrs.clone(), tuples)
    }

    /// π, projection. Keeps the named attributes. Tuples that become identical merge into one,
    /// so a projection can return fewer tuples than its input.
    pub fn project(&self, names: &[&str]) -> Relation {
        let idx: Vec<usize> = names.iter().map(|n| self.index_of(n)).collect();
        let tuples = self
            .tuples
            .iter()
            .map(|t| idx.iter().map(|&i| t[i].clone()).collect())
            .collect();
        Relation::from_parts(names.iter().map(|n| n.to_string()).collect(), tuples)
    }

    /// ρ, rename. Changes one attribute's name; the tuples are untouched.
    pub fn rename(&self, from: &str, to: &str) -> Relation {
        let i = self.index_of(from);
        let mut attrs = self.attrs.clone();
        attrs[i] = to.to_string();
        Relation::from_parts(attrs, self.tuples.clone())
    }

    /// Renames every attribute by adding a prefix, as `customer AS c` does in SQL.
    pub fn alias(&self, prefix: &str) -> Relation {
        let attrs = self.attrs.iter().map(|a| format!("{prefix}.{a}")).collect();
        Relation::from_parts(attrs, self.tuples.clone())
    }

    /// ×, Cartesian product. Every tuple of `self` paired with every tuple of `other`.
    /// The headings must not overlap; rename first if they do.
    pub fn product(&self, other: &Relation) -> Relation {
        let mut attrs = self.attrs.clone();
        attrs.extend(other.attrs.iter().cloned());
        let tuples = self
            .tuples
            .iter()
            .flat_map(|a| {
                other.tuples.iter().map(move |b| {
                    let mut t = a.clone();
                    t.extend(b.iter().cloned());
                    t
                })
            })
            .collect();
        Relation::from_parts(attrs, tuples)
    }

    /// ⋈θ, theta join. By definition, a product followed by a selection.
    pub fn theta_join(&self, other: &Relation, pred: impl Fn(&Row) -> bool) -> Relation {
        self.product(other).select(pred)
    }

    /// ⋈, natural join. Pairs tuples that agree on every attribute the two headings share, and
    /// keeps each shared attribute once. With no shared attributes it is a product.
    pub fn natural_join(&self, other: &Relation) -> Relation {
        let shared: Vec<&String> = self
            .attrs
            .iter()
            .filter(|a| other.attrs.contains(a))
            .collect();
        let left_idx: Vec<usize> = shared.iter().map(|a| self.index_of(a)).collect();
        let right_idx: Vec<usize> = shared.iter().map(|a| other.index_of(a)).collect();
        let right_rest: Vec<usize> = (0..other.attrs.len())
            .filter(|i| !right_idx.contains(i))
            .collect();

        let mut attrs = self.attrs.clone();
        attrs.extend(right_rest.iter().map(|&i| other.attrs[i].clone()));

        let mut tuples = BTreeSet::new();
        for a in &self.tuples {
            for b in &other.tuples {
                if left_idx.iter().zip(&right_idx).all(|(&i, &j)| a[i] == b[j]) {
                    let mut t = a.clone();
                    t.extend(right_rest.iter().map(|&j| b[j].clone()));
                    tuples.insert(t);
                }
            }
        }
        Relation::from_parts(attrs, tuples)
    }

    /// ∪, union. Both relations must have the same heading.
    pub fn union(&self, other: &Relation) -> Relation {
        let other = other.reordered_like(self);
        Relation::from_parts(
            self.attrs.clone(),
            self.tuples.union(&other.tuples).cloned().collect(),
        )
    }

    /// −, difference. Tuples in `self` that are not in `other`.
    pub fn difference(&self, other: &Relation) -> Relation {
        let other = other.reordered_like(self);
        Relation::from_parts(
            self.attrs.clone(),
            self.tuples.difference(&other.tuples).cloned().collect(),
        )
    }

    fn reordered_like(&self, target: &Relation) -> Relation {
        let mine: BTreeSet<&String> = self.attrs.iter().collect();
        let theirs: BTreeSet<&String> = target.attrs.iter().collect();
        assert_eq!(mine, theirs, "headings differ");
        let names: Vec<&str> = target.attrs.iter().map(String::as_str).collect();
        self.project(&names)
    }

    /// True if no two tuples agree on all of `names`: the attributes identify every tuple.
    pub fn is_superkey(&self, names: &[&str]) -> bool {
        self.project(names).len() == self.len()
    }

    /// True if `names` is a superkey and no attribute can be dropped from it: a candidate key.
    pub fn is_candidate_key(&self, names: &[&str]) -> bool {
        self.is_superkey(names)
            && (0..names.len()).all(|skip| {
                let smaller: Vec<&str> = names
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != skip)
                    .map(|(_, n)| *n)
                    .collect();
                smaller.is_empty() || !self.is_superkey(&smaller)
            })
    }
}

/// Relations are equal when they have the same heading and the same body. The order of
/// attributes and of tuples plays no part.
impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.canonical() == other.canonical()
    }
}

impl Eq for Relation {}

/// Prints the relation as a table. The printed order is just one way of writing a set down.
impl fmt::Display for Relation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cells: Vec<Vec<String>> = self
            .tuples
            .iter()
            .map(|t| t.iter().map(|v| v.to_string()).collect())
            .collect();
        let widths: Vec<usize> = (0..self.attrs.len())
            .map(|i| {
                cells
                    .iter()
                    .map(|r| r[i].len())
                    .chain([self.attrs[i].len()])
                    .max()
                    .unwrap()
            })
            .collect();
        let line = |f: &mut fmt::Formatter<'_>, vals: &[String]| -> fmt::Result {
            let parts: Vec<String> = vals
                .iter()
                .zip(&widths)
                .map(|(v, w)| format!("{v:<w$}"))
                .collect();
            writeln!(f, "  {}", parts.join("  "))
        };
        line(f, &self.attrs)?;
        let rule: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
        line(f, &rule)?;
        for r in &cells {
            line(f, r)?;
        }
        write!(
            f,
            "  ({} tuple{})",
            self.len(),
            if self.len() == 1 { "" } else { "s" }
        )
    }
}

/// Counts how many tuples share each value of an attribute. Handy for explaining projections.
pub fn histogram(r: &Relation, name: &str) -> BTreeMap<Value, usize> {
    let i = r.index_of(name);
    let mut out = BTreeMap::new();
    for t in &r.tuples {
        *out.entry(t[i].clone()).or_insert(0) += 1;
    }
    out
}

/// The running example used across Module 01's articles.
pub mod sample {
    use super::*;

    pub fn customer() -> Relation {
        Relation::new(
            ["id", "name", "segment", "region"],
            vec![
                [1.into(), "Asha".into(), "retail".into(), "ASIA".into()],
                [
                    2.into(),
                    "Bruno".into(),
                    "wholesale".into(),
                    "EUROPE".into(),
                ],
                [3.into(), "Chen".into(), "retail".into(), "ASIA".into()],
                [4.into(), "Dara".into(), "retail".into(), "AFRICA".into()],
                [
                    5.into(),
                    "Emeka".into(),
                    "government".into(),
                    "AFRICA".into(),
                ],
                [6.into(), "Farah".into(), "retail".into(), "EUROPE".into()],
            ],
        )
    }

    pub fn orders() -> Relation {
        Relation::new(
            ["order_id", "cust_id", "amount"],
            vec![
                [101.into(), 1.into(), 250.into()],
                [102.into(), 3.into(), 40.into()],
                [103.into(), 1.into(), 90.into()],
                [104.into(), 4.into(), 610.into()],
                [105.into(), 6.into(), 75.into()],
                [106.into(), 9.into(), 120.into()],
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::sample::{customer, orders};
    use super::*;

    #[test]
    fn duplicates_collapse_and_order_does_not_matter() {
        let a = Relation::new(
            ["x", "y"],
            vec![[1.into(), "a".into()], [2.into(), "b".into()]],
        );
        let b = Relation::new(
            ["y", "x"],
            vec![
                ["b".into(), 2.into()],
                ["a".into(), 1.into()],
                ["a".into(), 1.into()],
            ],
        );
        assert_eq!(b.len(), 2);
        assert_eq!(a, b);
    }

    #[test]
    fn selections_commute_and_cascade() {
        let r = customer();
        let retail = |t: &Row| t.text("segment") == "retail";
        let asia = |t: &Row| t.text("region") == "ASIA";
        assert_eq!(r.select(retail).select(asia), r.select(asia).select(retail));
        assert_eq!(
            r.select(retail).select(asia),
            r.select(|t| retail(t) && asia(t))
        );
    }

    #[test]
    fn projection_can_shrink_cardinality() {
        let r = customer();
        assert_eq!(r.len(), 6);
        assert_eq!(r.project(&["region"]).len(), 3);
        assert_eq!(r.project(&["segment", "region"]).len(), 5);
    }

    #[test]
    fn select_and_project_commute_only_when_the_predicate_survives() {
        let r = customer();
        let in_asia = |t: &Row| t.text("region") == "ASIA";
        // The predicate uses only kept attributes: either order gives the same answer.
        assert_eq!(
            r.select(in_asia).project(&["name", "region"]),
            r.project(&["name", "region"]).select(in_asia)
        );
        // Projecting the predicate's attribute away first makes selection impossible.
        let gone = std::panic::catch_unwind(|| r.project(&["name"]).select(in_asia));
        assert!(gone.is_err());
    }

    #[test]
    fn projections_cascade() {
        let r = customer();
        assert_eq!(
            r.project(&["name", "region"]).project(&["region"]),
            r.project(&["region"])
        );
    }

    #[test]
    fn rename_moves_past_projection() {
        let r = customer();
        assert_eq!(
            r.rename("id", "cust_id").project(&["cust_id", "name"]),
            r.project(&["id", "name"]).rename("id", "cust_id")
        );
    }

    #[test]
    fn rename_changes_only_the_heading() {
        let r = customer().rename("id", "cust_id");
        assert_eq!(r.len(), 6);
        assert!(r.attrs().contains(&"cust_id".to_string()));
    }

    #[test]
    fn product_size_is_the_product_of_sizes() {
        let p = customer().alias("c").product(&orders().alias("o"));
        assert_eq!(p.len(), 6 * 6);
        assert_eq!(p.degree(), 4 + 3);
    }

    #[test]
    fn theta_join_is_a_filtered_product() {
        let c = customer().alias("c");
        let o = orders().alias("o");
        let eq = |t: &Row| t.get("c.id") == t.get("o.cust_id");
        assert_eq!(c.theta_join(&o, eq), c.product(&o).select(eq));
        assert_eq!(c.theta_join(&o, eq).len(), 5);
    }

    #[test]
    fn natural_join_is_rename_product_select_project() {
        let c = customer().rename("id", "cust_id");
        let o = orders();
        let direct = c.natural_join(&o);
        let derived = c
            .product(&o.rename("cust_id", "o_cust_id"))
            .select(|t| t.get("cust_id") == t.get("o_cust_id"))
            .project(&["cust_id", "name", "segment", "region", "order_id", "amount"]);
        assert_eq!(direct, derived);
        // Order 106 has no matching customer, and customers 2 and 5 have no orders.
        assert_eq!(direct.len(), 5);
    }

    #[test]
    fn natural_join_without_shared_attributes_is_a_product() {
        let c = customer().alias("c");
        let o = orders().alias("o");
        assert_eq!(c.natural_join(&o), c.product(&o));
    }

    #[test]
    fn natural_join_commutes() {
        let c = customer().rename("id", "cust_id");
        let o = orders();
        assert_eq!(c.natural_join(&o), o.natural_join(&c));
    }

    #[test]
    fn keys() {
        let r = customer();
        assert!(r.is_superkey(&["id"]));
        assert!(r.is_candidate_key(&["id"]));
        assert!(r.is_superkey(&["id", "region"]));
        assert!(!r.is_candidate_key(&["id", "region"]));
        assert!(!r.is_superkey(&["region"]));
        assert!(r.is_candidate_key(&["name"]));
    }

    #[test]
    fn union_and_difference() {
        let r = customer();
        let retail = r.select(|t| t.text("segment") == "retail");
        let rest = r.difference(&retail);
        assert_eq!(retail.union(&rest), r);
        assert_eq!(rest.len(), 2);
    }
}
