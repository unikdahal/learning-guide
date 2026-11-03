//! SQL's three-valued logic (Kleene's strong logic): TRUE, FALSE and UNKNOWN.
//!
//! UNKNOWN behaves like "could be either". `x AND y` is FALSE if either side is FALSE, whatever
//! the other side is; it is TRUE only if both are TRUE; otherwise it is UNKNOWN. `OR` is the mirror
//! image. `NOT UNKNOWN` is UNKNOWN. A WHERE clause keeps a row only when its condition is TRUE.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Truth {
    True,
    False,
    Unknown,
}

use Truth::*;

impl Truth {
    pub const ALL: [Truth; 3] = [True, False, Unknown];

    pub fn and(self, other: Truth) -> Truth {
        match (self, other) {
            (False, _) | (_, False) => False,
            (True, True) => True,
            _ => Unknown,
        }
    }

    pub fn or(self, other: Truth) -> Truth {
        match (self, other) {
            (True, _) | (_, True) => True,
            (False, False) => False,
            _ => Unknown,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Truth {
        match self {
            True => False,
            False => True,
            Unknown => Unknown,
        }
    }

    /// `a = b` where either side may be NULL (`None`).
    pub fn eq<T: PartialEq>(a: Option<T>, b: Option<T>) -> Truth {
        match (a, b) {
            (Some(x), Some(y)) => Truth::from(x == y),
            _ => Unknown,
        }
    }

    /// `a IS NOT DISTINCT FROM b`: like `=`, except two NULLs count as equal. Never UNKNOWN.
    pub fn not_distinct<T: PartialEq>(a: Option<T>, b: Option<T>) -> Truth {
        Truth::from(a == b)
    }

    /// SQL `x IN (list)`: TRUE if some element equals x, else UNKNOWN if any comparison was
    /// UNKNOWN, else FALSE. It is the OR of `x = e` over the list.
    pub fn is_in<T: PartialEq + Copy>(x: Option<T>, list: &[Option<T>]) -> Truth {
        list.iter().fold(False, |acc, &e| acc.or(Truth::eq(x, e)))
    }

    /// What a WHERE clause does with the result: keep the row only if TRUE.
    pub fn keeps_row(self) -> bool {
        self == True
    }
}

impl From<bool> for Truth {
    fn from(b: bool) -> Self {
        if b { True } else { False }
    }
}

impl fmt::Display for Truth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            True => "TRUE",
            False => "FALSE",
            Unknown => "UNKNOWN",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn de_morgan_still_holds() {
        for a in Truth::ALL {
            for b in Truth::ALL {
                assert_eq!(a.and(b).not(), a.not().or(b.not()));
                assert_eq!(a.or(b).not(), a.not().and(b.not()));
            }
        }
    }

    #[test]
    fn excluded_middle_fails() {
        // In two-valued logic, p OR NOT p is always TRUE. With UNKNOWN it is not.
        assert_eq!(Unknown.or(Unknown.not()), Unknown);
        assert!(!Unknown.or(Unknown.not()).keeps_row());
    }

    #[test]
    fn null_equals_nothing_not_even_null() {
        assert_eq!(Truth::eq::<i64>(None, None), Unknown);
        assert_eq!(Truth::not_distinct::<i64>(None, None), True);
    }

    #[test]
    fn not_in_with_a_null_is_never_true() {
        let list = [Some(1), Some(3), None];
        // 2 is not in the list, but the NULL might be 2, so NOT IN is UNKNOWN, not TRUE.
        assert_eq!(Truth::is_in(Some(2), &list).not(), Unknown);
        // 1 is in the list, so NOT IN is FALSE.
        assert_eq!(Truth::is_in(Some(1), &list).not(), False);
        // Either way, a WHERE ... NOT IN keeps nothing.
        assert!(!Truth::is_in(Some(2), &list).not().keeps_row());
    }
}
