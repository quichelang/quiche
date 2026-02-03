//! Region-based arena allocation.
//!
//! Regions provide efficient bulk allocation where all values are
//! freed together when the region is dropped.

use std::alloc::{alloc, dealloc, Layout};
use std::cell::Cell;
use std::marker::PhantomData;
use std::mem::{align_of, size_of, ManuallyDrop};
use std::ptr::NonNull;

/// Default chunk size for regions (64KB).
const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

/// A memory region for bulk allocation.
///
/// All allocations within a region are freed together when the region
/// is dropped, making this very efficient for short-lived allocations.
///
/// # Example
///
/// ```
/// use perceus_mem::Region;
///
/// let mut region = Region::new();
/// let x = region.alloc(42);
/// let y = region.alloc("hello");
/// // Both x and y are freed when region drops
/// ```
pub struct Region {
    chunks: Vec<Chunk>,
    current_offset: Cell<usize>,
    drop_list: Vec<DropEntry>,
}

struct Chunk {
    ptr: NonNull<u8>,
    layout: Layout,
}

struct DropEntry {
    ptr: *mut u8,
    drop_fn: unsafe fn(*mut u8),
}

impl Region {
    /// Create a new region with default chunk size.
    pub fn new() -> Self {
        Self::with_chunk_size(DEFAULT_CHUNK_SIZE)
    }

    /// Create a new region with the specified chunk size.
    pub fn with_chunk_size(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 16).expect("Invalid chunk layout");
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr).expect("Allocation failed");

        Self {
            chunks: vec![Chunk { ptr, layout }],
            current_offset: Cell::new(0),
            drop_list: Vec::new(),
        }
    }

    /// Allocate a value in this region.
    ///
    /// The value's destructor will be called when the region is dropped.
    pub fn alloc<T>(&mut self, value: T) -> &mut T {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_raw(layout);

        // Write the value
        let typed_ptr = ptr.as_ptr() as *mut T;
        unsafe {
            typed_ptr.write(value);
        }

        // Register destructor if T needs dropping
        if std::mem::needs_drop::<T>() {
            self.drop_list.push(DropEntry {
                ptr: typed_ptr as *mut u8,
                drop_fn: drop_in_place::<T>,
            });
        }

        unsafe { &mut *typed_ptr }
    }

    /// Allocate raw memory with the given layout.
    fn alloc_raw(&mut self, layout: Layout) -> NonNull<u8> {
        let chunk = self.chunks.last().unwrap();
        let chunk_ptr = chunk.ptr.as_ptr() as usize;
        let chunk_end = chunk_ptr + chunk.layout.size();

        // Align the current offset
        let current = chunk_ptr + self.current_offset.get();
        let aligned = (current + layout.align() - 1) & !(layout.align() - 1);
        let next = aligned + layout.size();

        if next <= chunk_end {
            // Fits in current chunk
            self.current_offset.set(next - chunk_ptr);
            NonNull::new(aligned as *mut u8).unwrap()
        } else {
            // Need a new chunk
            let new_size = std::cmp::max(layout.size() + layout.align(), DEFAULT_CHUNK_SIZE);
            let new_layout = Layout::from_size_align(new_size, 16).expect("Invalid layout");
            let ptr = unsafe { alloc(new_layout) };
            let ptr = NonNull::new(ptr).expect("Allocation failed");

            self.chunks.push(Chunk {
                ptr,
                layout: new_layout,
            });

            // Allocate from new chunk
            let aligned = (ptr.as_ptr() as usize + layout.align() - 1) & !(layout.align() - 1);
            self.current_offset
                .set(aligned - ptr.as_ptr() as usize + layout.size());
            NonNull::new(aligned as *mut u8).unwrap()
        }
    }

    /// Create a handle to a region-allocated value.
    pub fn alloc_handle<T>(&mut self, value: T) -> RegionHandle<'_, T> {
        let reference = self.alloc(value);
        RegionHandle {
            ptr: reference as *mut T,
            _region: PhantomData,
        }
    }

    /// Returns the total number of bytes allocated across all chunks.
    pub fn total_capacity(&self) -> usize {
        self.chunks.iter().map(|c| c.layout.size()).sum()
    }

    /// Returns the number of registered drop entries.
    pub fn drop_count(&self) -> usize {
        self.drop_list.len()
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        // Drop all values in reverse order
        for entry in self.drop_list.drain(..).rev() {
            unsafe {
                (entry.drop_fn)(entry.ptr);
            }
        }

        // Free all chunks
        for chunk in self.chunks.drain(..) {
            unsafe {
                dealloc(chunk.ptr.as_ptr(), chunk.layout);
            }
        }
    }
}

/// Helper function to generate drop_in_place calls.
unsafe fn drop_in_place<T>(ptr: *mut u8) {
    std::ptr::drop_in_place(ptr as *mut T);
}

/// Handle to a region-allocated value.
///
/// The handle is valid as long as the region is alive.
pub struct RegionHandle<'r, T> {
    ptr: *mut T,
    _region: PhantomData<&'r Region>,
}

impl<'r, T> RegionHandle<'r, T> {
    /// Get a reference to the value.
    #[inline]
    pub fn get(&self) -> &T {
        unsafe { &*self.ptr }
    }

    /// Get a mutable reference to the value.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<'r, T> std::ops::Deref for RegionHandle<'r, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<'r, T> std::ops::DerefMut for RegionHandle<'r, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

// RegionHandle is Copy if T doesn't need special handling
impl<'r, T> Clone for RegionHandle<'r, T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _region: PhantomData,
        }
    }
}

impl<'r, T> Copy for RegionHandle<'r, T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_region_alloc() {
        let mut region = Region::new();
        let x = region.alloc(42);
        assert_eq!(*x, 42);

        let y = region.alloc("hello");
        assert_eq!(*y, "hello");
    }

    #[test]
    fn test_region_handle() {
        let mut region = Region::new();
        let mut h = region.alloc_handle(vec![1, 2, 3]);

        h.push(4);
        assert_eq!(&*h, &vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_region_drops_values() {
        let dropped = Rc::new(Cell::new(false));

        struct DropTracker(Rc<Cell<bool>>);
        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        {
            let mut region = Region::new();
            region.alloc(DropTracker(dropped.clone()));
            assert!(!dropped.get());
        }

        assert!(dropped.get());
    }

    #[test]
    fn test_region_multiple_chunks() {
        let mut region = Region::with_chunk_size(64);

        // Allocate more than fits in one chunk
        for i in 0..100 {
            region.alloc(i as i64);
        }

        assert!(region.chunks.len() > 1);
    }
}
