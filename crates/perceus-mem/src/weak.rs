//! Weak references for cycle prevention.

use crate::generation::GenIndex;
use crate::handle::Handle;
use crate::policy::AtomicPolicy;
use crate::store::GenericStore;
use std::marker::PhantomData;

/// A weak (non-owning) reference to a value in a store.
///
/// Weak references do not prevent deallocation. They can be upgraded
/// to strong handles if the value is still alive.
///
/// This is inspired by Vala's weak references and helps prevent
/// reference cycles.
///
/// # Example
///
/// ```
/// use perceus_mem::{Store, Weak};
///
/// let mut store: Store<i32> = Store::new();
/// let handle = store.alloc(42);
/// let weak = Weak::from_handle(&handle);
///
/// // Weak can check if still alive
/// assert!(weak.is_alive(&store));
///
/// store.release(handle);
///
/// // Now it's dead
/// assert!(!weak.is_alive(&store));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Weak<T> {
    gen_index: GenIndex,
    _marker: PhantomData<T>,
}

impl<T> Weak<T> {
    /// Create a weak reference from a strong handle.
    #[inline]
    pub fn from_handle(handle: &Handle<T>) -> Self {
        Self {
            gen_index: handle.gen_index,
            _marker: PhantomData,
        }
    }

    /// Check if the referent is still alive.
    pub fn is_alive<P: AtomicPolicy>(&self, store: &GenericStore<T, P>) -> bool {
        store.is_valid(&Handle::new(self.gen_index))
    }

    /// Try to get a reference to the value.
    ///
    /// Returns `Some(&T)` if the value is still alive, `None` otherwise.
    pub fn get<'a, P: AtomicPolicy>(&self, store: &'a GenericStore<T, P>) -> Option<&'a T> {
        store.get(&Handle::new(self.gen_index))
    }

    /// Upgrade to a strong handle, incrementing the reference count.
    ///
    /// Returns `Some(Handle<T>)` if the value is still alive, `None` otherwise.
    pub fn upgrade<P: AtomicPolicy>(&self, store: &mut GenericStore<T, P>) -> Option<Handle<T>> {
        let handle = Handle::new(self.gen_index);
        if store.retain(&handle).is_some() {
            Some(handle)
        } else {
            None
        }
    }

    /// Get the underlying generational index.
    #[inline]
    pub const fn gen_index(&self) -> GenIndex {
        self.gen_index
    }
}

impl<T> std::fmt::Debug for Weak<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Weak")
            .field("gen_index", &self.gen_index)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[test]
    fn test_weak_from_handle() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        assert!(weak.is_alive(&store));
        assert_eq!(weak.get(&store), Some(&42));
    }

    #[test]
    fn test_weak_after_release() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        store.release(handle);

        assert!(!weak.is_alive(&store));
        assert_eq!(weak.get(&store), None);
    }

    #[test]
    fn test_weak_upgrade() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        // Original ref count = 1
        assert_eq!(store.ref_count(&handle), Some(1));

        // Upgrade increments ref count
        let upgraded = weak.upgrade(&mut store).unwrap();
        assert_eq!(store.ref_count(&handle), Some(2));

        // Release original
        store.release(handle);
        assert_eq!(store.ref_count(&upgraded), Some(1));

        // Value still alive via upgraded handle
        assert_eq!(store.get(&upgraded), Some(&42));
    }

    #[test]
    fn test_weak_upgrade_after_release() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        store.release(handle);

        // Cannot upgrade dead weak ref
        assert!(weak.upgrade(&mut store).is_none());
    }
}
