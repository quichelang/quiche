//! Generation-validated handle type.

use crate::generation::GenIndex;
use std::marker::PhantomData;

/// A handle to an allocated value in a store.
///
/// Handles carry a generational index that is validated on access.
/// If the slot has been freed and reused, the handle becomes invalid
/// and will return `None` on access attempts.
///
/// Handles are always `Copy` regardless of whether `T` is `Copy`,
/// since they only contain an index, not the value itself.
///
/// # Example
///
/// ```
/// use perceus_mem::{Store, Handle};
///
/// let mut store: Store<i32> = Store::new();
/// let handle = store.alloc(42);
/// assert_eq!(store.get(&handle), Some(&42));
/// ```
pub struct Handle<T> {
    pub(crate) gen_index: GenIndex,
    pub(crate) _marker: PhantomData<fn() -> T>,
}

// Manual Clone implementation that doesn't require T: Clone
impl<T> Clone for Handle<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

// Manual Copy implementation that doesn't require T: Copy
impl<T> Copy for Handle<T> {}

// Manual PartialEq that doesn't require T: PartialEq
impl<T> PartialEq for Handle<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.gen_index == other.gen_index
    }
}

impl<T> Eq for Handle<T> {}

// Manual Hash that doesn't require T: Hash
impl<T> std::hash::Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.gen_index.hash(state);
    }
}

impl<T> Handle<T> {
    /// Create a new handle from a generational index.
    #[inline]
    pub(crate) const fn new(gen_index: GenIndex) -> Self {
        Self {
            gen_index,
            _marker: PhantomData,
        }
    }

    /// Get the underlying generational index.
    #[inline]
    pub const fn gen_index(&self) -> GenIndex {
        self.gen_index
    }

    /// Get the slot index.
    #[inline]
    pub const fn index(&self) -> u32 {
        self.gen_index.index()
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("gen_index", &self.gen_index)
            .finish()
    }
}
