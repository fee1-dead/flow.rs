use std::fmt::{Display, Write};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct UFix64(u64);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Fix64(i64);

macro_rules! fix_impl {
    ($ty:ident($inner:ident)) => {
        impl $ty {
            /// The maximum value this can represent.
            pub const MAX: Self = Self::from_raw($inner::MAX);
            /// The minimum value this can represent.
            pub const MIN: Self = Self::from_raw($inner::MIN);

            /// Creates Self from the raw representation. 
            pub const fn from_raw(n: $inner) -> Self {
                Self(n)
            }

            /// Retrieves the inner raw representation.
            pub const fn to_raw(self) -> $inner {
                self.0
            }
        }
        impl std::fmt::Debug for $ty {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Display::fmt(self, f)
            }
        }
    };
}

fix_impl!(Fix64(i64));
fix_impl!(UFix64(u64));

impl Display for Fix64 {
    /// ```
    /// # use cadence_json::Fix64;
    /// assert_eq!(Fix64::from_raw(1).to_string(), "0.00000001");
    /// assert_eq!(Fix64::from_raw(99999999).to_string(), "0.99999999");
    /// assert_eq!(Fix64::from_raw(999999999).to_string(), "9.99999999");
    /// assert_eq!(Fix64::from_raw(i64::MAX).to_string(), "92233720368.54775807");
    /// assert_eq!(Fix64::from_raw(i64::MIN).to_string(), "-92233720368.54775808");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0.to_string();
        let mut s = s.as_str();
        if s.as_bytes()[0] == b'-' {
            f.write_char('-')?;
            s = &s[1..];
        }
        if s.len() <= 8 {
            f.write_str("0.")?;
            for _ in s.len()..8 {
                f.write_char('0')?;
            }
            f.write_str(&s)
        } else {
            f.write_str(&s[..s.len() - 8])?;
            f.write_char('.')?;
            f.write_str(&s[s.len() - 8..])
        }
    }
}

impl Display for UFix64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.0.to_string();
        if s.len() <= 8 {
            f.write_str("0.")?;
            for _ in s.len()..8 {
                f.write_char('0')?;
            }
            f.write_str(&s)
        } else {
            f.write_str(&s[..s.len() - 8])?;
            f.write_char('.')?;
            f.write_str(&s[s.len() - 8..])
        }
    }
}