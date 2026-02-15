use std::cmp::PartialEq;
use std::fmt::Debug;
use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, DerefMut};

/// Quiche's list type — a growable, ordered collection.
///
/// Wraps `Vec<T>` with chainable methods for functional-style operations.
/// Derefs to `Vec<T>` so all standard vector methods are available.
#[derive(Clone, Debug)]
pub struct List<T>(pub Vec<T>);

impl<T> Deref for List<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T: Debug> Display for List<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> From<Vec<T>> for List<T> {
    fn from(v: Vec<T>) -> Self {
        List(v)
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        List(iter.into_iter().collect())
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List(Vec::new())
    }

    /// Push a value onto the list (mutating).
    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> List<U> {
        List(self.0.into_iter().map(f).collect())
    }

    pub fn filter<F: FnMut(&T) -> bool>(self, f: F) -> Self {
        List(self.0.into_iter().filter(f).collect())
    }

    pub fn flat_map<U, F: FnMut(T) -> List<U>>(self, mut f: F) -> List<U> {
        List(self.0.into_iter().flat_map(|x| f(x).0).collect())
    }

    pub fn concat(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }
}

impl<T: PartialEq> List<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.0.contains(value)
    }
}

impl<T> List<List<T>> {
    pub fn flatten(self) -> List<T> {
        List(self.0.into_iter().flat_map(|l| l.0).collect())
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

use crate::QuicheType;
impl<T> QuicheType for List<T> {
    type Inner = Vec<T>;
    fn inner(self) -> Vec<T> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_new_empty() {
        let l: List<i64> = List::new();
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn list_push() {
        let mut l = List::new();
        l.push(1);
        l.push(2);
        l.push(3);
        assert_eq!(l.0, vec![1, 2, 3]);
    }

    #[test]
    fn list_map() {
        let l = List(vec![1, 2, 3]);
        let doubled = l.map(|x| x * 2);
        assert_eq!(doubled.0, vec![2, 4, 6]);
    }

    #[test]
    fn list_filter() {
        let l = List(vec![1, 2, 3, 4, 5]);
        let evens = l.filter(|x| x % 2 == 0);
        assert_eq!(evens.0, vec![2, 4]);
    }

    #[test]
    fn list_flat_map() {
        let l = List(vec![1, 2, 3]);
        let expanded = l.flat_map(|x| List(vec![x, x * 10]));
        assert_eq!(expanded.0, vec![1, 10, 2, 20, 3, 30]);
    }

    #[test]
    fn list_flatten() {
        let l = List(vec![List(vec![1, 2]), List(vec![3, 4])]);
        let flat = l.flatten();
        assert_eq!(flat.0, vec![1, 2, 3, 4]);
    }

    #[test]
    fn list_concat() {
        let a = List(vec![1, 2]);
        let b = List(vec![3, 4]);
        let c = a.concat(b);
        assert_eq!(c.0, vec![1, 2, 3, 4]);
    }

    #[test]
    fn list_deref_vec_methods() {
        let l = List(vec![10, 20, 30]);
        assert_eq!(l.len(), 3);
        assert_eq!(l[1], 20);
        assert!(!l.is_empty());
    }

    #[test]
    fn list_from_vec() {
        let v = vec![1, 2, 3];
        let l: List<i64> = v.into();
        assert_eq!(l.0, vec![1, 2, 3]);
    }

    #[test]
    fn list_display() {
        let l = List(vec![1, 2, 3]);
        assert_eq!(format!("{l}"), "[1, 2, 3]");
    }
}
