use std::cmp::PartialEq;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::Hash;
use std::ops::{Add, Deref};
use std::sync::Arc;

/// Quiche's string type — an immutable, reference-counted string.
///
/// Wraps `Arc<str>` for cheap cloning and zero-copy sharing.
/// Derefs to `&str` so all standard string methods are available.
#[derive(Clone, Debug, Eq, Hash)]
pub struct Str(pub Arc<str>);

impl Deref for Str {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", &*self.0)
    }
}

impl PartialEq for Str {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}

impl PartialEq<&str> for Str {
    fn eq(&self, other: &&str) -> bool {
        &*self.0 == *other
    }
}

impl From<&str> for Str {
    fn from(s: &str) -> Self {
        Str(Arc::from(s))
    }
}

impl From<String> for Str {
    fn from(s: String) -> Self {
        Str(Arc::from(s.as_str()))
    }
}

impl Add for Str {
    type Output = Str;
    fn add(self, other: Str) -> Str {
        let mut s = self.0.to_string();
        s.push_str(&other.0);
        Str(Arc::from(s.as_str()))
    }
}

impl Add<&str> for Str {
    type Output = Str;
    fn add(self, other: &str) -> Str {
        let mut s = self.0.to_string();
        s.push_str(other);
        Str(Arc::from(s.as_str()))
    }
}

/// Construct a `Str` from any `Display` value.
pub fn str<T: std::fmt::Display>(x: T) -> Str {
    Str(Arc::from(x.to_string().as_str()))
}

use crate::impl_quiche_type;
impl_quiche_type!(Str, Arc<str>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn str_from_literal() {
        let s = str("hello");
        assert_eq!(s, "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn str_display() {
        let s = str("world");
        assert_eq!(format!("{s}"), "world");
    }

    #[test]
    fn str_clone_shares() {
        let a = str("shared");
        let b = a.clone();
        assert_eq!(a, b);
        assert!(Arc::ptr_eq(&a.0, &b.0));
    }

    #[test]
    fn str_add() {
        let a = str("Hello");
        let b = str(" World");
        let c = a + b;
        assert_eq!(c, "Hello World");
    }

    #[test]
    fn str_add_ref() {
        let s = str("Hello");
        let r = s + " World";
        assert_eq!(r, "Hello World");
    }

    #[test]
    fn str_deref_methods() {
        let s = str("Hello World");
        assert!(s.contains("World"));
        assert!(s.starts_with("Hello"));
        assert_eq!(s.to_uppercase(), "HELLO WORLD");
    }

    #[test]
    fn str_from_number() {
        let s = str(42);
        assert_eq!(s, "42");
    }
}
