//! Thread-safety policy abstraction.
//!
//! This module provides the `AtomicPolicy` trait for abstracting over
//! single-threaded (Rc) and thread-safe (Arc) reference counting.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

/// Trait abstracting over Rc/Arc and interior mutability.
///
/// This allows the same store implementation to work for both
/// single-threaded and thread-safe scenarios without code duplication.
pub trait AtomicPolicy: 'static {
    /// Reference-counted pointer type (Rc or Arc).
    type Ptr<T: 'static>: Clone;

    /// Counter type for reference counting (Cell<u32> or AtomicU32).
    type Counter: Counter;

    /// Create a new counter initialized to the given value.
    fn new_counter(initial: u32) -> Self::Counter;
}

/// Trait for counter operations, abstracting Cell vs Atomic.
pub trait Counter: Default {
    fn get(&self) -> u32;
    fn set(&self, val: u32);
    fn increment(&self) -> u32;
    fn decrement(&self) -> u32;
}

// ============================================================================
// SingleThreaded Policy
// ============================================================================

/// Single-threaded policy using Rc and Cell.
///
/// Best for single-threaded applications where performance is critical.
/// Uses Cell for interior mutability (no panics).
#[derive(Debug, Clone, Copy, Default)]
pub struct SingleThreaded;

impl AtomicPolicy for SingleThreaded {
    type Ptr<T: 'static> = Rc<T>;
    type Counter = Cell<u32>;

    #[inline]
    fn new_counter(initial: u32) -> Self::Counter {
        Cell::new(initial)
    }
}

impl Counter for Cell<u32> {
    #[inline]
    fn get(&self) -> u32 {
        Cell::get(self)
    }

    #[inline]
    fn set(&self, val: u32) {
        Cell::set(self, val);
    }

    #[inline]
    fn increment(&self) -> u32 {
        let val = self.get();
        self.set(val + 1);
        val + 1
    }

    #[inline]
    fn decrement(&self) -> u32 {
        let val = self.get();
        debug_assert!(val > 0, "Decrementing zero reference count");
        self.set(val - 1);
        val - 1
    }
}

// ============================================================================
// ThreadSafe Policy
// ============================================================================

/// Thread-safe policy using Arc and AtomicU32.
///
/// Safe for sharing across thread boundaries.
/// Slightly higher overhead due to atomic operations.
#[derive(Debug, Clone, Copy, Default)]
pub struct ThreadSafe;

impl AtomicPolicy for ThreadSafe {
    type Ptr<T: 'static> = Arc<T>;
    type Counter = AtomicU32;

    #[inline]
    fn new_counter(initial: u32) -> Self::Counter {
        AtomicU32::new(initial)
    }
}

impl Counter for AtomicU32 {
    #[inline]
    fn get(&self) -> u32 {
        self.load(Ordering::Acquire)
    }

    #[inline]
    fn set(&self, val: u32) {
        self.store(val, Ordering::Release);
    }

    #[inline]
    fn increment(&self) -> u32 {
        self.fetch_add(1, Ordering::AcqRel) + 1
    }

    #[inline]
    fn decrement(&self) -> u32 {
        let prev = self.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(prev > 0, "Decrementing zero reference count");
        prev - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_counter() {
        let counter = Cell::new(0);
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicU32::new(0);
        assert_eq!(counter.get(), 0);
        assert_eq!(counter.increment(), 1);
        assert_eq!(counter.increment(), 2);
        assert_eq!(counter.decrement(), 1);
        assert_eq!(counter.get(), 1);
    }
}
