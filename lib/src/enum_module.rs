//! Quiche `Enum` module — Elixir-style collection operations.
//!
//! Standalone functions that work on `List[T]`, pipeable via `|>`.

use crate::List;

/// Static module for collection operations, used as `Enum.filter(...)` in Quiche.
pub struct Enum;

impl Enum {
    /// Filter elements that satisfy a predicate.
    pub fn filter<T>(list: List<T>, mut f: impl FnMut(&T) -> bool) -> List<T> {
        List(list.0.into_iter().filter(|x| f(x)).collect())
    }

    /// Transform each element.
    pub fn map<T, U>(list: List<T>, f: impl FnMut(T) -> U) -> List<U> {
        List(list.0.into_iter().map(f).collect())
    }

    /// Reduce a list to a single value with an accumulator.
    pub fn reduce<T, A>(list: List<T>, acc: A, f: impl FnMut(A, T) -> A) -> A {
        list.0.into_iter().fold(acc, f)
    }

    /// Sort elements (requires Ord).
    pub fn sort<T: Ord>(mut list: List<T>) -> List<T> {
        list.0.sort();
        list
    }

    /// Reverse a list.
    pub fn reverse<T>(mut list: List<T>) -> List<T> {
        list.0.reverse();
        list
    }

    /// Find the first element matching a predicate.
    pub fn find<T>(list: List<T>, mut f: impl FnMut(&T) -> bool) -> Option<T> {
        list.0.into_iter().find(|x| f(x))
    }

    /// Check if any element satisfies a predicate.
    pub fn any<T>(list: List<T>, mut f: impl FnMut(&T) -> bool) -> bool {
        list.0.iter().any(|x| f(x))
    }

    /// Check if all elements satisfy a predicate.
    pub fn all<T>(list: List<T>, mut f: impl FnMut(&T) -> bool) -> bool {
        list.0.iter().all(|x| f(x))
    }

    /// Count elements.
    pub fn count<T>(list: &List<T>) -> i64 {
        list.0.len() as i64
    }

    /// Take the first N elements.
    pub fn take<T>(list: List<T>, n: i64) -> List<T> {
        List(list.0.into_iter().take(n as usize).collect())
    }

    /// Drop the first N elements.
    pub fn drop<T>(list: List<T>, n: i64) -> List<T> {
        List(list.0.into_iter().skip(n as usize).collect())
    }

    /// Flatten a list of lists.
    pub fn flat_map<T, U>(list: List<T>, mut f: impl FnMut(T) -> List<U>) -> List<U> {
        List(list.0.into_iter().flat_map(|x| f(x).0).collect())
    }

    /// Zip two lists into a list of pairs.
    pub fn zip<A, B>(a: List<A>, b: List<B>) -> List<(A, B)> {
        List(a.0.into_iter().zip(b.0).collect())
    }

    /// Join a list of strings with a separator.
    pub fn join(list: List<crate::Str>, sep: crate::Str) -> crate::Str {
        let parts: Vec<&str> = list.iter().map(|s| &**s).collect();
        crate::Str(std::sync::Arc::from(parts.join(&*sep).as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::str;

    #[test]
    fn enum_filter() {
        let nums = List(vec![1i64, 2, 3, 4, 5]);
        let evens = Enum::filter(nums, |x| x % 2 == 0);
        assert_eq!(evens.0, vec![2, 4]);
    }

    #[test]
    fn enum_map() {
        let nums = List(vec![1i64, 2, 3]);
        let doubled = Enum::map(nums, |x| x * 2);
        assert_eq!(doubled.0, vec![2, 4, 6]);
    }

    #[test]
    fn enum_sort() {
        let nums = List(vec![3i64, 1, 2]);
        let sorted = Enum::sort(nums);
        assert_eq!(sorted.0, vec![1, 2, 3]);
    }

    #[test]
    fn enum_reduce() {
        let nums = List(vec![1i64, 2, 3, 4]);
        let sum = Enum::reduce(nums, 0i64, |acc, x| acc + x);
        assert_eq!(sum, 10);
    }

    #[test]
    fn enum_any_all() {
        let nums = List(vec![1i64, 2, 3]);
        assert!(Enum::any(nums.clone(), |x| *x == 2));
        assert!(Enum::all(nums, |x| *x > 0));
    }

    #[test]
    fn enum_join() {
        let words = List(vec![str("a"), str("b"), str("c")]);
        let joined = Enum::join(words, str(", "));
        assert_eq!(&*joined, "a, b, c");
    }

    #[test]
    fn enum_take_drop() {
        let nums = List(vec![1i64, 2, 3, 4, 5]);
        let first3 = Enum::take(nums.clone(), 3);
        assert_eq!(first3.0, vec![1, 2, 3]);
        let last2 = Enum::drop(nums, 3);
        assert_eq!(last2.0, vec![4, 5]);
    }
}
