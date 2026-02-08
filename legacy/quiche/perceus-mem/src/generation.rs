//! Generation counter for tracking allocation slot reuse.

use std::fmt;

/// Generation counter - incremented on each allocation slot reuse.
///
/// When a slot is freed and reused, its generation is incremented.
/// This allows handles to detect if they're pointing to stale data.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Generation(u32);

impl Generation {
    /// Create a new generation starting at 0.
    #[inline]
    pub const fn new() -> Self {
        Self(0)
    }

    /// Increment the generation, wrapping on overflow.
    #[inline]
    pub fn increment(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    /// Get the raw value.
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Gen({})", self.0)
    }
}

/// Generational index combining a slot index and generation.
///
/// This is the core type that enables safe handle-based access.
/// A `GenIndex` is valid only if its generation matches the slot's current generation.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GenIndex {
    /// Index into the store's slot array.
    pub(crate) index: u32,
    /// Generation at the time this index was created.
    pub(crate) generation: Generation,
}

impl GenIndex {
    /// Create a new generational index.
    #[inline]
    pub(crate) const fn new(index: u32, generation: Generation) -> Self {
        Self { index, generation }
    }

    /// Get the slot index.
    #[inline]
    pub const fn index(self) -> u32 {
        self.index
    }

    /// Get the generation.
    #[inline]
    pub const fn generation(self) -> Generation {
        self.generation
    }
}

impl fmt::Debug for GenIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GenIndex({}, {:?})", self.index, self.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_increment() {
        let mut gen = Generation::new();
        assert_eq!(gen.value(), 0);
        gen.increment();
        assert_eq!(gen.value(), 1);
    }

    #[test]
    fn test_generation_wrapping() {
        let mut gen = Generation(u32::MAX);
        gen.increment();
        assert_eq!(gen.value(), 0);
    }

    #[test]
    fn test_gen_index_equality() {
        let idx1 = GenIndex::new(5, Generation(10));
        let idx2 = GenIndex::new(5, Generation(10));
        let idx3 = GenIndex::new(5, Generation(11));

        assert_eq!(idx1, idx2);
        assert_ne!(idx1, idx3);
    }
}
