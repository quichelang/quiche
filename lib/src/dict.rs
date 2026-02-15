use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
/// Quiche's dictionary type — a key-value store.
///
/// Wraps `HashMap<K, V>` with chainable builder methods.
/// Derefs to `HashMap<K, V>` so all standard map methods are available.
#[derive(Clone, Debug)]
pub struct Dict<K, V>(pub HashMap<K, V>);

impl<K, V> Deref for Dict<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    type Target = HashMap<K, V>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V> DerefMut for Dict<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn deref_mut(&mut self) -> &mut HashMap<K, V> {
        &mut self.0
    }
}

impl<K, V> Display for Dict<K, V>
where
    K: Eq + Hash + Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.0)
    }
}

impl<K, V> PartialEq for Dict<K, V>
where
    K: Eq + Hash + PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<K, V> From<HashMap<K, V>> for Dict<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn from(m: HashMap<K, V>) -> Self {
        Dict(m)
    }
}

impl<K, V> IntoIterator for Dict<K, V>
where
    K: Eq + Hash,
{
    type Item = (K, V);
    type IntoIter = std::collections::hash_map::IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, V> FromIterator<(K, V)> for Dict<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Dict(iter.into_iter().collect())
    }
}

impl<K, V> Dict<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    pub fn new() -> Self {
        Dict(HashMap::new())
    }

    pub fn set(mut self, key: K, value: V) -> Self {
        self.0.insert(key, value);
        self
    }

    pub fn has(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    pub fn remove_key(mut self, key: &K) -> Self {
        self.0.remove(key);
        self
    }

    pub fn get_value(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<K, V> Default for Dict<K, V>
where
    K: Eq + Hash,
    V: PartialEq,
{
    fn default() -> Self {
        Self::new()
    }
}

use crate::QuicheType;
impl<K: Eq + Hash, V: PartialEq> QuicheType for Dict<K, V> {
    type Inner = HashMap<K, V>;
    fn inner(self) -> HashMap<K, V> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dict_new_empty() {
        let d: Dict<String, i64> = Dict::new();
        assert_eq!(d.len(), 0);
    }

    #[test]
    fn dict_set_chain() {
        let d = Dict::new().set("a".to_string(), 1).set("b".to_string(), 2);
        assert_eq!(d.len(), 2);
        assert_eq!(d.get_value(&"a".to_string()), Some(&1));
        assert_eq!(d.get_value(&"b".to_string()), Some(&2));
    }

    #[test]
    fn dict_has() {
        let d = Dict::new().set("key", 42);
        assert!(d.has(&"key"));
        assert!(!d.has(&"missing"));
    }

    #[test]
    fn dict_remove_key() {
        let d = Dict::new().set("a", 1).set("b", 2);
        let d = d.remove_key(&"a");
        assert!(!d.has(&"a"));
        assert!(d.has(&"b"));
    }

    #[test]
    fn dict_from_hashmap() {
        let mut m = HashMap::new();
        m.insert("x", 10);
        let d: Dict<&str, i64> = m.into();
        assert_eq!(d.get_value(&"x"), Some(&10));
    }

    #[test]
    fn dict_deref_methods() {
        let d = Dict::new().set(1, "one").set(2, "two");
        assert!(d.contains_key(&1));
        assert_eq!(d.len(), 2);
    }
}
