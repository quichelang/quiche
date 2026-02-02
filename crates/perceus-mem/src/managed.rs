//! Reference-counted wrapper with FBIP (Functional-But-In-Place) support.

use std::cell::Cell;
use std::ops::Deref;
use std::rc::Rc;

/// A reference-counted value with FBIP support.
///
/// `Managed<T>` provides shared ownership with automatic copy-on-write
/// semantics when you need mutable access but aren't the sole owner.
///
/// # Example
///
/// ```
/// use perceus_mem::Managed;
///
/// let data = Managed::new(vec![1, 2, 3]);
/// let shared = data.share();
///
/// // Both point to the same data
/// assert_eq!(*data.get(), vec![1, 2, 3]);
/// assert_eq!(*shared.get(), vec![1, 2, 3]);
/// ```
pub struct Managed<T> {
    inner: Rc<ManagedInner<T>>,
}

struct ManagedInner<T> {
    value: std::cell::UnsafeCell<T>,
}

impl<T> Managed<T> {
    /// Create a new managed value.
    #[inline]
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(ManagedInner {
                value: std::cell::UnsafeCell::new(value),
            }),
        }
    }

    /// Get a shared reference to the value.
    #[inline]
    pub fn get(&self) -> &T {
        unsafe { &*self.inner.value.get() }
    }

    /// Clone the handle, incrementing the reference count.
    #[inline]
    pub fn share(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }

    /// Get the current reference count.
    #[inline]
    pub fn ref_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }

    /// Returns true if this is the only reference.
    #[inline]
    pub fn is_unique(&self) -> bool {
        Rc::strong_count(&self.inner) == 1
    }

    /// Try to get mutable access if this is the only reference.
    ///
    /// Returns `Some(&mut T)` if ref_count == 1, `None` otherwise.
    /// This is the core FBIP operation - in-place mutation when possible.
    #[inline]
    pub fn try_get_mut(&mut self) -> Option<&mut T> {
        if self.is_unique() {
            Some(unsafe { &mut *self.inner.value.get() })
        } else {
            None
        }
    }
}

impl<T: Clone> Managed<T> {
    /// Get mutable access, cloning if necessary.
    ///
    /// If this is the only reference, returns a mutable reference.
    /// Otherwise, clones the value and returns a mutable reference to the clone.
    /// This is the copy-on-write pattern.
    pub fn get_mut_or_clone(&mut self) -> &mut T {
        if !self.is_unique() {
            // Clone the value into a new Managed
            *self = Managed::new(self.get().clone());
        }
        // Now we're definitely unique
        unsafe { &mut *self.inner.value.get() }
    }

    /// Consume this managed value, returning the inner value.
    ///
    /// If this is the only reference, unwraps without cloning.
    /// Otherwise, clones the value.
    pub fn into_inner(self) -> T {
        match Rc::try_unwrap(self.inner) {
            Ok(inner) => inner.value.into_inner(),
            Err(rc) => unsafe { (*rc.value.get()).clone() },
        }
    }
}

impl<T> Clone for Managed<T> {
    #[inline]
    fn clone(&self) -> Self {
        self.share()
    }
}

impl<T> Deref for Managed<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Managed<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Managed")
            .field("value", self.get())
            .field("ref_count", &self.ref_count())
            .finish()
    }
}

impl<T: PartialEq> PartialEq for Managed<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for Managed<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_managed_new() {
        let m = Managed::new(42);
        assert_eq!(*m.get(), 42);
        assert_eq!(m.ref_count(), 1);
    }

    #[test]
    fn test_managed_share() {
        let m1 = Managed::new(42);
        let m2 = m1.share();

        assert_eq!(m1.ref_count(), 2);
        assert_eq!(m2.ref_count(), 2);
        assert_eq!(*m1.get(), *m2.get());
    }

    #[test]
    fn test_fbip_unique_mutation() {
        let mut m = Managed::new(vec![1, 2, 3]);

        // Only one reference - should get mutable access
        assert!(m.is_unique());
        m.try_get_mut().unwrap().push(4);
        assert_eq!(*m.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_fbip_shared_no_mutation() {
        let mut m1 = Managed::new(vec![1, 2, 3]);
        let _m2 = m1.share();

        // Shared - should not get mutable access
        assert!(!m1.is_unique());
        assert!(m1.try_get_mut().is_none());
    }

    #[test]
    fn test_copy_on_write() {
        let mut m1 = Managed::new(vec![1, 2, 3]);
        let m2 = m1.share();

        // This will clone because m1 is shared
        m1.get_mut_or_clone().push(4);

        // m1 now has its own copy
        assert_eq!(*m1.get(), vec![1, 2, 3, 4]);
        // m2 still has the original
        assert_eq!(*m2.get(), vec![1, 2, 3]);
    }

    #[test]
    fn test_into_inner_unique() {
        let m = Managed::new(String::from("hello"));
        assert!(m.is_unique());
        let s = m.into_inner();
        assert_eq!(s, "hello");
    }

    #[test]
    fn test_into_inner_shared() {
        let m1 = Managed::new(String::from("hello"));
        let m2 = m1.share();

        // Will clone because shared
        let s = m1.into_inner();
        assert_eq!(s, "hello");
        assert_eq!(*m2.get(), "hello");
    }
}
