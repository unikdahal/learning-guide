//! Module 03 lab. Each article's code is an example in `examples/`; shared pieces live here.

pub mod encodings;

/// Number of 8-byte fields in the lab's wide record: 128 bytes, two cache lines.
pub const FIELDS: usize = 16;

/// A row-major table: each record's 16 fields stored together.
pub struct Rows(pub Vec<[i64; FIELDS]>);

/// The same table column-major: one vector per field.
pub struct Columns(pub Vec<Vec<i64>>);

impl Rows {
    pub fn generate(n: usize) -> Rows {
        Rows(
            (0..n)
                .map(|r| std::array::from_fn(|f| (r * 31 + f * 7) as i64 % 1000))
                .collect(),
        )
    }

    /// Sums the first `k` fields over all rows.
    pub fn sum_fields(&self, k: usize) -> i64 {
        let mut s = 0i64;
        for row in &self.0 {
            for v in &row[..k] {
                s = s.wrapping_add(*v);
            }
        }
        s
    }

    /// Reads every field of the rows at the given positions.
    pub fn fetch(&self, ids: &[u32]) -> i64 {
        let mut s = 0i64;
        for &id in ids {
            for v in &self.0[id as usize] {
                s = s.wrapping_add(*v);
            }
        }
        s
    }
}

impl Columns {
    pub fn from_rows(r: &Rows) -> Columns {
        Columns(
            (0..FIELDS)
                .map(|f| r.0.iter().map(|row| row[f]).collect())
                .collect(),
        )
    }

    pub fn sum_fields(&self, k: usize) -> i64 {
        let mut s = 0i64;
        for col in &self.0[..k] {
            for v in col {
                s = s.wrapping_add(*v);
            }
        }
        s
    }

    pub fn fetch(&self, ids: &[u32]) -> i64 {
        let mut s = 0i64;
        for &id in ids {
            for col in &self.0 {
                s = s.wrapping_add(col[id as usize]);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layouts_agree() {
        let r = Rows::generate(1000);
        let c = Columns::from_rows(&r);
        for k in 1..=FIELDS {
            assert_eq!(r.sum_fields(k), c.sum_fields(k), "k = {k}");
        }
        let ids: Vec<u32> = (0..1000).step_by(7).collect();
        assert_eq!(r.fetch(&ids), c.fetch(&ids));
    }
}
