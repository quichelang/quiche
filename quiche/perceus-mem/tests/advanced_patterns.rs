//! Advanced Tests Inspired by Established Allocator Libraries
//!
//! These tests are modeled after patterns found in:
//! - slotmap: Generational index validation, ABA problem prevention
//! - bumpalo: Arena stress tests, alignment verification
//! - typed-arena: Drop ordering, bulk allocation patterns
//! - std::rc::Rc: Reference counting semantics

use perceus_mem::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

// =============================================================================
// SLOTMAP-INSPIRED TESTS: Generational Index Validation
// =============================================================================

mod slotmap_patterns {
    use super::*;

    /// Test that stale keys are properly invalidated (ABA problem prevention)
    #[test]
    fn aba_problem_prevention() {
        let mut store: Store<String> = Store::new();

        // Insert "A"
        let key_a = store.alloc("A".to_string());
        assert_eq!(store.get(&key_a), Some(&"A".to_string()));

        // Remove "A"
        store.release(key_a);

        // Insert "B" - may reuse the same slot
        let key_b = store.alloc("B".to_string());

        // The old key should NOT work, even if slot is reused
        assert_eq!(store.get(&key_a), None);
        // New key should work
        assert_eq!(store.get(&key_b), Some(&"B".to_string()));
    }

    /// Test that removed-then-reinserted slots invalidate old keys
    #[test]
    fn generation_prevents_stale_access() {
        let mut store: Store<i32> = Store::new();
        let mut old_keys = Vec::new();

        // Create and remove many values, accumulating stale keys
        for i in 0..100 {
            let key = store.alloc(i);
            old_keys.push(key);
            store.release(key);
        }

        // Now insert new values
        let mut new_keys = Vec::new();
        for i in 100..200 {
            new_keys.push(store.alloc(i));
        }

        // ALL old keys should be invalid
        for key in &old_keys {
            assert_eq!(store.get(key), None, "Stale key should return None");
        }

        // All new keys should be valid
        for (i, key) in new_keys.iter().enumerate() {
            assert_eq!(store.get(key), Some(&(100 + i as i32)));
        }
    }

    /// Test key equality considers generation
    #[test]
    fn key_equality_includes_generation() {
        let mut store: Store<i32> = Store::new();

        let key1 = store.alloc(1);
        let key1_copy = key1;

        // Same key should be equal
        assert_eq!(key1, key1_copy);

        store.release(key1);
        let key2 = store.alloc(2);

        // If slot reused, keys should NOT be equal (different generation)
        if key1.index() == key2.index() {
            assert_ne!(
                key1, key2,
                "Same slot but different generation should be unequal"
            );
        }
    }

    /// Test that contains/has check works with generations
    #[test]
    fn is_valid_respects_generation() {
        let mut store: Store<i32> = Store::new();

        let key = store.alloc(42);
        assert!(store.is_valid(&key));

        store.release(key);
        assert!(!store.is_valid(&key));

        // Even after reallocating
        let _new_key = store.alloc(99);
        assert!(
            !store.is_valid(&key),
            "Old key should stay invalid after realloc"
        );
    }

    /// Test iteration stability after removals
    #[test]
    fn iteration_after_removals() {
        let mut store: Store<i32> = Store::new();
        let mut keys = Vec::new();

        // Insert 10 items
        for i in 0..10 {
            keys.push(store.alloc(i));
        }

        // Remove odd indices
        for i in (1..10).step_by(2) {
            store.release(keys[i]);
        }

        // Even indices should still be valid
        for i in (0..10).step_by(2) {
            assert!(store.is_valid(&keys[i]));
            assert_eq!(store.get(&keys[i]), Some(&(i as i32)));
        }

        // Odd indices should be invalid
        for i in (1..10).step_by(2) {
            assert!(!store.is_valid(&keys[i]));
        }
    }

    /// Test that clear doesn't cause generation overflow issues
    #[test]
    fn repeated_clear_and_reuse() {
        let mut store: Store<i32> = Store::new();

        for round in 0..10 {
            let mut keys = Vec::new();

            for i in 0..100 {
                keys.push(store.alloc(round * 100 + i));
            }

            // Verify all values
            for (i, key) in keys.iter().enumerate() {
                assert_eq!(store.get(key), Some(&(round * 100 + i as i32)));
            }

            // Release all
            for key in keys {
                store.release(key);
            }

            assert!(store.is_empty());
        }
    }

    /// Test that handle can be used as HashMap key
    #[test]
    fn handles_as_hashmap_keys() {
        let mut store: Store<String> = Store::new();
        let mut metadata: HashMap<Handle<String>, i32> = HashMap::new();

        let key1 = store.alloc("one".to_string());
        let key2 = store.alloc("two".to_string());
        let key3 = store.alloc("three".to_string());

        metadata.insert(key1, 1);
        metadata.insert(key2, 2);
        metadata.insert(key3, 3);

        assert_eq!(metadata.get(&key1), Some(&1));
        assert_eq!(metadata.get(&key2), Some(&2));
        assert_eq!(metadata.get(&key3), Some(&3));

        // Verify that copies work as keys too
        let key1_copy = key1;
        assert_eq!(metadata.get(&key1_copy), Some(&1));
    }
}

// =============================================================================
// BUMPALO-INSPIRED TESTS: Arena Allocation Patterns
// =============================================================================

mod bumpalo_patterns {
    use super::*;

    /// Test alignment requirements for different types
    #[test]
    fn alignment_requirements() {
        // Test alignment for each type separately to avoid borrow issues
        struct Align1(u8);
        struct Align2(u16);
        struct Align4(u32);
        struct Align8(u64);

        #[repr(align(16))]
        struct Align16([u8; 16]);

        #[repr(align(32))]
        struct Align32([u8; 32]);

        // Each allocation tested independently to avoid borrow conflicts
        {
            let mut region = Region::new();
            let ptr = region.alloc(Align1(1)) as *const Align1 as usize;
            assert_eq!(ptr % std::mem::align_of::<Align1>(), 0);
        }
        {
            let mut region = Region::new();
            let ptr = region.alloc(Align2(2)) as *const Align2 as usize;
            assert_eq!(ptr % std::mem::align_of::<Align2>(), 0);
        }
        {
            let mut region = Region::new();
            let ptr = region.alloc(Align4(4)) as *const Align4 as usize;
            assert_eq!(ptr % std::mem::align_of::<Align4>(), 0);
        }
        {
            let mut region = Region::new();
            let ptr = region.alloc(Align8(8)) as *const Align8 as usize;
            assert_eq!(ptr % std::mem::align_of::<Align8>(), 0);
        }
        {
            let mut region = Region::new();
            let ptr = region.alloc(Align16([0; 16])) as *const Align16 as usize;
            assert_eq!(ptr % std::mem::align_of::<Align16>(), 0);
        }
        {
            let mut region = Region::new();
            let ptr = region.alloc(Align32([0; 32])) as *const Align32 as usize;
            assert_eq!(ptr % std::mem::align_of::<Align32>(), 0);
        }
    }

    /// Test that allocating many small objects doesn't fragment too badly
    #[test]
    fn bulk_small_allocations() {
        let mut region = Region::new();

        // Allocate many small items
        for _ in 0..10000 {
            region.alloc(1u8);
        }

        // Region should use reasonable capacity (less than 1 byte per byte of data is impossible,
        // but we should be using a few kilobytes, not megabytes)
        let capacity = region.total_capacity();
        assert!(
            capacity <= 128 * 1024,
            "Capacity should be reasonable: {}",
            capacity
        );
    }

    /// Test that large allocations work correctly
    #[test]
    fn large_allocations() {
        // Test each large allocation separately to avoid borrow issues
        {
            let mut region = Region::with_chunk_size(1024);
            let large1 = region.alloc([0u8; 4096]);
            assert_eq!(large1.len(), 4096);
            assert_eq!(large1[0], 0);
        }
        {
            let mut region = Region::with_chunk_size(1024);
            let large2 = region.alloc([1u8; 8192]);
            assert_eq!(large2.len(), 8192);
            assert_eq!(large2[0], 1);
        }
    }

    /// Test interleaved small and large allocations
    #[test]
    fn interleaved_sizes() {
        let mut region = Region::with_chunk_size(256);

        for i in 0..100 {
            if i % 10 == 0 {
                // Large allocation
                region.alloc([i as u8; 512]);
            } else {
                // Small allocation
                region.alloc(i as u32);
            }
        }
    }

    /// Test that drop is called in reverse order (LIFO)
    #[test]
    fn drop_order_is_lifo() {
        let order = Rc::new(Cell::new(Vec::new()));

        struct DropTracker {
            id: i32,
            order: Rc<Cell<Vec<i32>>>,
        }

        impl Drop for DropTracker {
            fn drop(&mut self) {
                let mut v = self.order.take();
                v.push(self.id);
                self.order.set(v);
            }
        }

        {
            let mut region = Region::new();
            region.alloc(DropTracker {
                id: 1,
                order: order.clone(),
            });
            region.alloc(DropTracker {
                id: 2,
                order: order.clone(),
            });
            region.alloc(DropTracker {
                id: 3,
                order: order.clone(),
            });
        }

        let dropped = order.take();
        // LIFO order: 3, 2, 1
        assert_eq!(dropped, vec![3, 2, 1]);
    }

    /// Test that capacity grows appropriately
    #[test]
    fn capacity_growth() {
        let mut region = Region::with_chunk_size(64);
        let initial_capacity = region.total_capacity();

        // Fill up the first chunk
        for _ in 0..100 {
            region.alloc([0u8; 8]);
        }

        let grown_capacity = region.total_capacity();
        assert!(grown_capacity > initial_capacity);
    }
}

// =============================================================================
// RC-INSPIRED TESTS: Reference Counting Semantics
// =============================================================================

mod rc_patterns {
    use super::*;

    /// Test that Managed behaves like Rc for basic sharing
    #[test]
    fn managed_like_rc_sharing() {
        let m1 = Managed::new(vec![1, 2, 3]);
        let m2 = m1.share();
        let m3 = m2.share();

        // All refer to same data
        assert_eq!(*m1.get(), vec![1, 2, 3]);
        assert_eq!(*m2.get(), vec![1, 2, 3]);
        assert_eq!(*m3.get(), vec![1, 2, 3]);

        // Same ref count
        assert_eq!(m1.ref_count(), 3);
        assert_eq!(m2.ref_count(), 3);
        assert_eq!(m3.ref_count(), 3);
    }

    /// Test try_unwrap-like behavior (into_inner)
    #[test]
    fn into_inner_like_try_unwrap() {
        // Unique - should not clone
        let m = Managed::new("unique".to_string());
        let s = m.into_inner();
        assert_eq!(s, "unique");

        // Shared - should clone
        let m1 = Managed::new("shared".to_string());
        let m2 = m1.share();
        let s1 = m1.into_inner();
        assert_eq!(s1, "shared");
        assert_eq!(*m2.get(), "shared");
    }

    /// Test make_mut-like behavior (get_mut_or_clone)
    #[test]
    fn get_mut_or_clone_like_make_mut() {
        // Unique case
        let mut m = Managed::new(vec![1, 2, 3]);
        m.get_mut_or_clone().push(4);
        assert_eq!(m.ref_count(), 1);

        // Shared case - triggers clone
        let mut m1 = Managed::new(vec![1, 2, 3]);
        let m2 = m1.share();
        m1.get_mut_or_clone().push(4);

        // m1 should have its own copy now
        assert_eq!(m1.ref_count(), 1);
        assert_eq!(m2.ref_count(), 1);
        assert_eq!(*m1.get(), vec![1, 2, 3, 4]);
        assert_eq!(*m2.get(), vec![1, 2, 3]);
    }

    /// Test that drop happens at correct time
    #[test]
    fn drop_when_last_reference_goes() {
        let dropped = Rc::new(Cell::new(false));

        struct Tracker(Rc<Cell<bool>>);
        impl Drop for Tracker {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let m1 = Managed::new(Tracker(dropped.clone()));
        let m2 = m1.share();
        let m3 = m1.share();

        drop(m1);
        assert!(!dropped.get());

        drop(m2);
        assert!(!dropped.get());

        drop(m3);
        assert!(dropped.get());
    }

    /// Test circular reference prevention pattern
    #[test]
    fn break_cycles_with_weak() {
        let mut store: Store<String> = Store::new();

        let h1 = store.alloc("node1".to_string());
        let h2 = store.alloc("node2".to_string());

        // Create weak references for "back edges"
        let weak1 = Weak::from_handle(&h1);
        let weak2 = Weak::from_handle(&h2);

        // Weaks don't prevent deallocation
        store.release(h1);
        assert!(!weak1.is_alive(&store));

        // h2 is still accessible
        assert!(weak2.is_alive(&store));
        assert_eq!(store.get(&h2), Some(&"node2".to_string()));
    }
}

// =============================================================================
// TYPED-ARENA INSPIRED TESTS: Bulk Allocation
// =============================================================================

mod typed_arena_patterns {
    use super::*;

    /// Test allocating many of the same type efficiently
    #[test]
    fn bulk_allocation_same_type() {
        let mut region = Region::new();

        struct Node {
            value: i32,
            next: Option<*const Node>,
        }

        // Use alloc_handle to avoid borrow issues, or allocate one at a time
        for i in 0..1000 {
            let node = region.alloc(Node {
                value: i,
                next: None,
            });
            assert_eq!(node.value, i);
        }
    }

    /// Test that all types in a region are properly dropped
    #[test]
    fn heterogeneous_drops() {
        let string_count = Rc::new(Cell::new(0));
        let vec_count = Rc::new(Cell::new(0));

        struct StringTracker(Rc<Cell<i32>>);
        impl Drop for StringTracker {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        struct VecTracker(Rc<Cell<i32>>);
        impl Drop for VecTracker {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        {
            let mut region = Region::new();
            for _ in 0..10 {
                region.alloc(StringTracker(string_count.clone()));
                region.alloc(VecTracker(vec_count.clone()));
            }
        }

        assert_eq!(string_count.get(), 10);
        assert_eq!(vec_count.get(), 10);
    }
}

// =============================================================================
// STRESS TESTS: High-Volume Operations
// =============================================================================

mod stress_tests {
    use super::*;

    /// Stress test: Many allocations and deallocations
    #[test]
    fn store_high_churn() {
        let mut store: Store<i32> = Store::new();

        for iteration in 0..100 {
            let mut handles = Vec::new();

            // Allocate batch
            for i in 0..1000 {
                handles.push(store.alloc(iteration * 1000 + i));
            }

            // Release half randomly
            for i in (0..1000).step_by(2) {
                store.release(handles[i]);
            }

            // Allocate more
            for i in 0..500 {
                handles.push(store.alloc(iteration * 10000 + i));
            }

            // Release everything
            for h in handles.into_iter().skip(1).step_by(2) {
                store.release(h);
            }
            // Allocate and release more handles
            for _ in 0..500 {
                let h = store.alloc(0);
                store.release(h);
            }
        }
    }

    /// Stress test: Deep reference count
    #[test]
    fn managed_deep_ref_count() {
        let m = Managed::new(42);
        let mut shares = Vec::new();

        // Create many shares
        for _ in 0..10000 {
            shares.push(m.share());
        }

        assert_eq!(m.ref_count(), 10001);

        // Drop half
        let half = shares.len() / 2;
        shares.truncate(half);

        assert_eq!(m.ref_count(), half + 1);
    }

    /// Stress test: Region with many types
    #[test]
    fn region_many_types() {
        let mut region = Region::new();

        for _ in 0..1000 {
            region.alloc(1u8);
            region.alloc(2u16);
            region.alloc(3u32);
            region.alloc(4u64);
            region.alloc("string".to_string());
            region.alloc(vec![1, 2, 3, 4, 5]);
            region.alloc(Box::new(42));
        }
    }

    /// Stress test: Weak references across many allocations
    #[test]
    fn weak_stress() {
        let mut store: Store<i32> = Store::new();
        let mut weaks = Vec::new();
        let mut handles = Vec::new();

        // Create handles and weak refs
        for i in 0..1000 {
            let h = store.alloc(i);
            weaks.push(Weak::from_handle(&h));
            handles.push(h);
        }

        // All weaks should be alive
        for w in &weaks {
            assert!(w.is_alive(&store));
        }

        // Release all handles
        for h in handles {
            store.release(h);
        }

        // All weaks should be dead
        for w in &weaks {
            assert!(!w.is_alive(&store));
        }
    }
}

// =============================================================================
// PROPERTY-BASED TESTS (simplified, not using quickcheck)
// =============================================================================

mod property_tests {
    use super::*;

    /// Property: After alloc, get should always succeed
    #[test]
    fn property_alloc_then_get_succeeds() {
        let mut store: Store<i32> = Store::new();

        for value in 0..1000 {
            let h = store.alloc(value);
            assert_eq!(store.get(&h), Some(&value));
        }
    }

    /// Property: After release, get should always fail
    #[test]
    fn property_release_then_get_fails() {
        let mut store: Store<i32> = Store::new();

        for value in 0..1000 {
            let h = store.alloc(value);
            store.release(h);
            assert_eq!(store.get(&h), None);
        }
    }

    /// Property: ref_count == 1 means is_unique
    #[test]
    fn property_refcount_one_means_unique() {
        let m = Managed::new(vec![1, 2, 3]);
        assert!(m.is_unique());
        assert_eq!(m.ref_count(), 1);

        let m2 = m.share();
        assert!(!m.is_unique());
        assert!(!m2.is_unique());

        drop(m2);
        assert!(m.is_unique());
    }

    /// Property: weak upgrade only succeeds while value is alive
    #[test]
    fn property_weak_upgrade_validity() {
        let mut store: Store<String> = Store::new();

        for _ in 0..100 {
            let h = store.alloc("test".to_string());
            let w = Weak::from_handle(&h);

            // Should succeed while alive
            let upgraded = w.upgrade(&mut store);
            assert!(upgraded.is_some());

            // Release both
            store.release(h);
            store.release(upgraded.unwrap());

            // Should fail now
            assert!(w.upgrade(&mut store).is_none());
        }
    }
}
