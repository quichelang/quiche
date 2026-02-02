//! Generational store with slot-based allocation.

use crate::generation::{GenIndex, Generation};
use crate::handle::Handle;
use crate::policy::{AtomicPolicy, Counter, SingleThreaded, ThreadSafe};
use std::marker::PhantomData;

/// Slot state in the store.
enum Slot<T, P: AtomicPolicy> {
    /// Occupied slot with value and ref count.
    Occupied {
        value: T,
        generation: Generation,
        ref_count: P::Counter,
    },
    /// Free slot pointing to next free slot.
    Free {
        next_free: Option<u32>,
        generation: Generation,
    },
}

/// Generic store for objects with generation tracking.
///
/// The store manages allocation, deallocation, and access to values
/// using generation-validated handles.
///
/// # Type Parameters
///
/// - `T`: The type of values stored
/// - `P`: The atomicity policy (SingleThreaded or ThreadSafe)
pub struct GenericStore<T, P: AtomicPolicy = SingleThreaded> {
    slots: Vec<Slot<T, P>>,
    free_head: Option<u32>,
    len: usize,
    _policy: PhantomData<P>,
}

/// Single-threaded store (type alias for convenience).
pub type Store<T> = GenericStore<T, SingleThreaded>;

/// Thread-safe store (type alias for convenience).
pub type ThreadSafeStore<T> = GenericStore<T, ThreadSafe>;

impl<T, P: AtomicPolicy> GenericStore<T, P> {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: None,
            len: 0,
            _policy: PhantomData,
        }
    }

    /// Create a new store with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: None,
            len: 0,
            _policy: PhantomData,
        }
    }

    /// Returns the number of currently allocated values.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the store contains no values.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total capacity of the store.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Allocate a new value in the store, returning a handle.
    pub fn alloc(&mut self, value: T) -> Handle<T> {
        let (index, generation) = if let Some(free_idx) = self.free_head {
            // Reuse a free slot
            let slot = &mut self.slots[free_idx as usize];
            let (next_free, gen) = match slot {
                Slot::Free {
                    next_free,
                    generation,
                } => (*next_free, *generation),
                _ => unreachable!("Free head pointed to occupied slot"),
            };
            self.free_head = next_free;
            *slot = Slot::Occupied {
                value,
                generation: gen,
                ref_count: P::new_counter(1),
            };
            (free_idx, gen)
        } else {
            // Append a new slot
            let index = self.slots.len() as u32;
            let generation = Generation::new();
            self.slots.push(Slot::Occupied {
                value,
                generation,
                ref_count: P::new_counter(1),
            });
            (index, generation)
        };

        self.len += 1;
        Handle::new(GenIndex::new(index, generation))
    }

    /// Check if a handle is valid (points to an occupied slot with matching generation).
    #[inline]
    pub fn is_valid(&self, handle: &Handle<T>) -> bool {
        self.get(handle).is_some()
    }

    /// Get a reference to the value, if the handle is valid.
    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        let idx = handle.gen_index.index as usize;
        if idx >= self.slots.len() {
            return None;
        }
        match &self.slots[idx] {
            Slot::Occupied {
                value, generation, ..
            } if *generation == handle.gen_index.generation => Some(value),
            _ => None,
        }
    }

    /// Get a mutable reference to the value, if the handle is valid.
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        let idx = handle.gen_index.index as usize;
        if idx >= self.slots.len() {
            return None;
        }
        match &mut self.slots[idx] {
            Slot::Occupied {
                value, generation, ..
            } if *generation == handle.gen_index.generation => Some(value),
            _ => None,
        }
    }

    /// Get the reference count for a handle.
    pub fn ref_count(&self, handle: &Handle<T>) -> Option<u32> {
        let idx = handle.gen_index.index as usize;
        if idx >= self.slots.len() {
            return None;
        }
        match &self.slots[idx] {
            Slot::Occupied {
                generation,
                ref_count,
                ..
            } if *generation == handle.gen_index.generation => Some(ref_count.get()),
            _ => None,
        }
    }

    /// Increment the reference count for a handle (retain).
    ///
    /// Returns the new reference count, or None if the handle is invalid.
    pub fn retain(&mut self, handle: &Handle<T>) -> Option<u32> {
        let idx = handle.gen_index.index as usize;
        if idx >= self.slots.len() {
            return None;
        }
        match &self.slots[idx] {
            Slot::Occupied {
                generation,
                ref_count,
                ..
            } if *generation == handle.gen_index.generation => Some(ref_count.increment()),
            _ => None,
        }
    }

    /// Decrement the reference count for a handle (release).
    ///
    /// If the count reaches zero, the value is dropped and the slot is freed.
    /// Returns the new reference count, or None if the handle was invalid.
    pub fn release(&mut self, handle: Handle<T>) -> Option<u32> {
        let idx = handle.gen_index.index as usize;
        if idx >= self.slots.len() {
            return None;
        }

        // Check if valid and get ref count
        let should_free = match &self.slots[idx] {
            Slot::Occupied {
                generation,
                ref_count,
                ..
            } if *generation == handle.gen_index.generation => {
                let new_count = ref_count.decrement();
                if new_count == 0 {
                    Some(handle.gen_index.generation)
                } else {
                    return Some(new_count);
                }
            }
            _ => return None,
        };

        // Free the slot
        if let Some(gen) = should_free {
            let mut new_gen = gen;
            new_gen.increment();
            self.slots[idx] = Slot::Free {
                next_free: self.free_head,
                generation: new_gen,
            };
            self.free_head = Some(idx as u32);
            self.len -= 1;
            return Some(0);
        }

        None
    }

    /// Try to get mutable access if this is the only reference (FBIP pattern).
    ///
    /// Returns `Some(&mut T)` if ref_count == 1, otherwise `None`.
    pub fn try_get_unique(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        let idx = handle.gen_index.index as usize;
        if idx >= self.slots.len() {
            return None;
        }
        match &mut self.slots[idx] {
            Slot::Occupied {
                value,
                generation,
                ref_count,
            } if *generation == handle.gen_index.generation && ref_count.get() == 1 => Some(value),
            _ => None,
        }
    }
}

impl<T, P: AtomicPolicy> Default for GenericStore<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: std::fmt::Debug, P: AtomicPolicy> std::fmt::Debug for GenericStore<T, P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GenericStore")
            .field("len", &self.len)
            .field("capacity", &self.slots.capacity())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_and_get() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);
        assert_eq!(store.get(&h), Some(&42));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_generation_validation() {
        let mut store: Store<String> = Store::new();
        let h1 = store.alloc("hello".to_string());

        // Release the handle
        store.release(h1);

        // h1 should now be invalid
        assert_eq!(store.get(&h1), None);

        // Allocating again reuses the slot with new generation
        let h2 = store.alloc("world".to_string());
        assert_eq!(store.get(&h2), Some(&"world".to_string()));

        // h1 still invalid (different generation)
        assert_eq!(store.get(&h1), None);
    }

    #[test]
    fn test_retain_release() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        assert_eq!(store.ref_count(&h), Some(1));
        store.retain(&h);
        assert_eq!(store.ref_count(&h), Some(2));
        store.release(h);
        assert_eq!(store.ref_count(&h), Some(1));
        store.release(h);
        assert_eq!(store.ref_count(&h), None); // Freed
    }

    #[test]
    fn test_fbip_try_get_unique() {
        let mut store: Store<Vec<i32>> = Store::new();
        let h = store.alloc(vec![1, 2, 3]);

        // Only reference - should get unique access
        {
            let v = store.try_get_unique(&h).unwrap();
            v.push(4);
        }
        assert_eq!(store.get(&h), Some(&vec![1, 2, 3, 4]));

        // Retain to add another reference
        store.retain(&h);

        // Now try_get_unique should fail
        assert!(store.try_get_unique(&h).is_none());
    }
}
