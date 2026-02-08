//! Comprehensive Integration Tests for perceus-mem
//!
//! This test suite provides world-class coverage of all memory management primitives.
//! Tests are organized by module and cover:
//! - Normal operation
//! - Edge cases
//! - Error handling
//! - Thread safety (where applicable)
//! - Performance characteristics

use perceus_mem::*;
use std::cell::Cell;
use std::rc::Rc;

// =============================================================================
// HANDLE TESTS
// =============================================================================

mod handle_tests {
    use super::*;

    #[test]
    fn handle_is_copy() {
        let mut store: Store<i32> = Store::new();
        let h1 = store.alloc(42);
        let h2 = h1; // Copy
        let h3 = h1; // Copy again

        assert_eq!(store.get(&h1), Some(&42));
        assert_eq!(store.get(&h2), Some(&42));
        assert_eq!(store.get(&h3), Some(&42));
    }

    #[test]
    fn handle_clone_matches_copy() {
        let mut store: Store<i32> = Store::new();
        let h1 = store.alloc(42);
        let h2 = h1.clone();

        assert_eq!(h1, h2);
    }

    #[test]
    fn handle_equality() {
        let mut store: Store<i32> = Store::new();
        let h1 = store.alloc(42);
        let h2 = store.alloc(42); // Same value, different handle

        assert_ne!(h1, h2);
        assert_eq!(h1, h1);
    }

    #[test]
    fn handle_hashing() {
        use std::collections::HashMap;
        let mut store: Store<String> = Store::new();
        let h1 = store.alloc("one".to_string());
        let h2 = store.alloc("two".to_string());

        let mut map: HashMap<Handle<String>, i32> = HashMap::new();
        map.insert(h1, 1);
        map.insert(h2, 2);

        assert_eq!(map.get(&h1), Some(&1));
        assert_eq!(map.get(&h2), Some(&2));
    }

    #[test]
    fn handle_debug_format() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);
        let debug_str = format!("{:?}", h);
        assert!(debug_str.contains("Handle"));
        assert!(debug_str.contains("gen_index"));
    }

    #[test]
    fn handle_index_accessor() {
        let mut store: Store<i32> = Store::new();
        let h1 = store.alloc(1);
        let h2 = store.alloc(2);

        // Indices should be assigned
        let _ = h1.index();
        let _ = h2.index();
    }

    #[test]
    fn handle_works_with_non_clone_types() {
        struct NoCopy(i32);
        let mut store: Store<NoCopy> = Store::new();
        let h = store.alloc(NoCopy(42));
        let h2 = h; // Handle is Copy even though T is not
        assert_eq!(store.get(&h2).map(|x| x.0), Some(42));
    }

    #[test]
    fn handle_works_with_zero_sized_types() {
        let mut store: Store<()> = Store::new();
        let h = store.alloc(());
        assert_eq!(store.get(&h), Some(&()));
    }

    #[test]
    fn handle_gen_index_accessor() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);
        let gen_idx = h.gen_index();
        assert_eq!(gen_idx.index(), h.index());
    }
}

// =============================================================================
// STORE TESTS
// =============================================================================

mod store_tests {
    use super::*;

    #[test]
    fn store_new_is_empty() {
        let store: Store<i32> = Store::new();
        assert!(store.is_empty());
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn store_with_capacity() {
        let store: Store<i32> = Store::with_capacity(100);
        assert!(store.capacity() >= 100);
        assert!(store.is_empty());
    }

    #[test]
    fn store_alloc_increases_len() {
        let mut store: Store<i32> = Store::new();
        assert_eq!(store.len(), 0);

        store.alloc(1);
        assert_eq!(store.len(), 1);

        store.alloc(2);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn store_get_returns_correct_value() {
        let mut store: Store<String> = Store::new();
        let h = store.alloc("hello".to_string());
        assert_eq!(store.get(&h), Some(&"hello".to_string()));
    }

    #[test]
    fn store_get_mut_allows_modification() {
        let mut store: Store<Vec<i32>> = Store::new();
        let h = store.alloc(vec![1, 2, 3]);

        store.get_mut(&h).unwrap().push(4);
        assert_eq!(store.get(&h), Some(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn store_release_frees_slot() {
        let mut store: Store<String> = Store::new();
        let h = store.alloc("test".to_string());

        assert_eq!(store.len(), 1);
        store.release(h);
        assert_eq!(store.len(), 0);
    }

    #[test]
    fn store_release_returns_new_refcount() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        let result = store.release(h);
        assert_eq!(result, Some(0)); // Ref count went to 0
    }

    #[test]
    fn store_retain_increments_refcount() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        assert_eq!(store.ref_count(&h), Some(1));

        store.retain(&h);
        assert_eq!(store.ref_count(&h), Some(2));

        store.retain(&h);
        assert_eq!(store.ref_count(&h), Some(3));
    }

    #[test]
    fn store_release_with_multiple_refs() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        store.retain(&h);
        store.retain(&h);
        assert_eq!(store.ref_count(&h), Some(3));

        store.release(h);
        assert_eq!(store.ref_count(&h), Some(2));

        store.release(h);
        assert_eq!(store.ref_count(&h), Some(1));

        store.release(h);
        assert_eq!(store.ref_count(&h), None); // No longer valid
    }

    #[test]
    fn store_slot_reuse_increments_generation() {
        let mut store: Store<i32> = Store::new();

        let h1 = store.alloc(1);
        let h1_gen = h1.gen_index().generation().value();

        store.release(h1);

        let h2 = store.alloc(2);
        let h2_gen = h2.gen_index().generation().value();

        // Same slot index, but different generation
        assert_eq!(h1.index(), h2.index());
        assert!(h2_gen > h1_gen);
    }

    #[test]
    fn store_stale_handle_returns_none() {
        let mut store: Store<i32> = Store::new();

        let h1 = store.alloc(1);
        store.release(h1);
        let _h2 = store.alloc(2);

        // h1 is now stale - same slot but old generation
        assert_eq!(store.get(&h1), None);
        assert!(!store.is_valid(&h1));
    }

    #[test]
    fn store_is_valid_checks_generation() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        assert!(store.is_valid(&h));

        store.release(h);

        assert!(!store.is_valid(&h));
    }

    #[test]
    fn store_fbip_try_get_unique_with_single_ref() {
        let mut store: Store<Vec<i32>> = Store::new();
        let h = store.alloc(vec![1, 2, 3]);

        // Single ref - should succeed
        let unique = store.try_get_unique(&h);
        assert!(unique.is_some());
        unique.unwrap().push(4);

        assert_eq!(store.get(&h), Some(&vec![1, 2, 3, 4]));
    }

    #[test]
    fn store_fbip_try_get_unique_with_multiple_refs() {
        let mut store: Store<Vec<i32>> = Store::new();
        let h = store.alloc(vec![1, 2, 3]);

        store.retain(&h); // Now 2 refs

        // Multiple refs - should fail
        assert!(store.try_get_unique(&h).is_none());
    }

    #[test]
    fn store_many_allocations() {
        let mut store: Store<i32> = Store::new();
        let mut handles = Vec::new();

        for i in 0..1000 {
            handles.push(store.alloc(i));
        }

        assert_eq!(store.len(), 1000);

        for (i, h) in handles.iter().enumerate() {
            assert_eq!(store.get(h), Some(&(i as i32)));
        }
    }

    #[test]
    fn store_alloc_release_cycle() {
        let mut store: Store<i32> = Store::new();

        for cycle in 0..100 {
            let h = store.alloc(cycle);
            assert_eq!(store.get(&h), Some(&cycle));
            store.release(h);
            assert_eq!(store.len(), 0);
        }
    }

    #[test]
    fn store_retain_on_invalid_handle_returns_none() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);
        store.release(h);

        assert_eq!(store.retain(&h), None);
    }

    #[test]
    fn store_release_invalid_handle_returns_none() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);
        store.release(h);

        assert_eq!(store.release(h), None);
    }

    #[test]
    fn store_drop_calls_destructors() {
        let dropped = Rc::new(Cell::new(0));

        struct DropCounter(Rc<Cell<i32>>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        {
            let mut store: Store<DropCounter> = Store::new();
            store.alloc(DropCounter(dropped.clone()));
            store.alloc(DropCounter(dropped.clone()));
            store.alloc(DropCounter(dropped.clone()));
            // Store drops here
        }

        assert_eq!(dropped.get(), 3);
    }

    #[test]
    fn store_release_calls_destructor() {
        let dropped = Rc::new(Cell::new(false));

        struct DropTracker(Rc<Cell<bool>>);
        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let mut store: Store<DropTracker> = Store::new();
        let h = store.alloc(DropTracker(dropped.clone()));

        assert!(!dropped.get());
        store.release(h);
        assert!(dropped.get());
    }

    #[test]
    fn store_debug_format() {
        let mut store: Store<i32> = Store::new();
        store.alloc(42);
        let debug_str = format!("{:?}", store);
        assert!(debug_str.contains("GenericStore"));
    }

    #[test]
    fn store_default_trait() {
        let store: Store<i32> = Store::default();
        assert!(store.is_empty());
    }

    #[test]
    fn store_interleaved_alloc_release() {
        let mut store: Store<i32> = Store::new();

        let h1 = store.alloc(1);
        let h2 = store.alloc(2);

        store.release(h1);

        let h3 = store.alloc(3); // Should reuse h1's slot

        assert!(store.is_valid(&h2));
        assert!(store.is_valid(&h3));
        assert!(!store.is_valid(&h1)); // Stale
    }

    #[test]
    fn store_large_ref_counts() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        for _ in 0..1000 {
            store.retain(&h);
        }

        assert_eq!(store.ref_count(&h), Some(1001));

        for _ in 0..1000 {
            store.release(h);
        }

        assert_eq!(store.ref_count(&h), Some(1));
    }
}

// =============================================================================
// MANAGED TESTS
// =============================================================================

mod managed_tests {
    use super::*;

    #[test]
    fn managed_new_has_ref_count_one() {
        let m = Managed::new(42);
        assert_eq!(m.ref_count(), 1);
        assert!(m.is_unique());
    }

    #[test]
    fn managed_get_returns_value() {
        let m = Managed::new("hello".to_string());
        assert_eq!(*m.get(), "hello");
    }

    #[test]
    fn managed_deref_works() {
        let m = Managed::new(42);
        assert_eq!(*m, 42);
    }

    #[test]
    fn managed_share_increments_refcount() {
        let m1 = Managed::new(42);
        let m2 = m1.share();

        assert_eq!(m1.ref_count(), 2);
        assert_eq!(m2.ref_count(), 2);
    }

    #[test]
    fn managed_clone_is_share() {
        let m1 = Managed::new(42);
        let m2 = m1.clone();

        assert_eq!(m1.ref_count(), 2);
        assert_eq!(m2.ref_count(), 2);
    }

    #[test]
    fn managed_multiple_shares() {
        let m = Managed::new("test".to_string());
        let _s1 = m.share();
        let _s2 = m.share();
        let _s3 = m.share();

        assert_eq!(m.ref_count(), 4);
    }

    #[test]
    fn managed_is_unique_after_drops() {
        let m1 = Managed::new(42);
        {
            let _m2 = m1.share();
            let _m3 = m1.share();
            assert!(!m1.is_unique());
        }
        // m2 and m3 dropped
        assert!(m1.is_unique());
    }

    #[test]
    fn managed_try_get_mut_unique() {
        let mut m = Managed::new(vec![1, 2, 3]);
        assert!(m.is_unique());

        let inner = m.try_get_mut().unwrap();
        inner.push(4);

        assert_eq!(*m.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn managed_try_get_mut_shared() {
        let mut m1 = Managed::new(vec![1, 2, 3]);
        let _m2 = m1.share();

        assert!(!m1.is_unique());
        assert!(m1.try_get_mut().is_none());
    }

    #[test]
    fn managed_get_mut_or_clone_unique() {
        let mut m = Managed::new(vec![1, 2, 3]);

        // Should not clone - we're unique
        m.get_mut_or_clone().push(4);

        assert_eq!(*m.get(), vec![1, 2, 3, 4]);
        assert_eq!(m.ref_count(), 1); // Still unique
    }

    #[test]
    fn managed_get_mut_or_clone_shared() {
        let mut m1 = Managed::new(vec![1, 2, 3]);
        let m2 = m1.share();

        assert_eq!(m1.ref_count(), 2);

        // Should clone - we're shared
        m1.get_mut_or_clone().push(4);

        // m1 now has its own copy
        assert_eq!(*m1.get(), vec![1, 2, 3, 4]);
        assert_eq!(m1.ref_count(), 1); // Now unique

        // m2 unchanged
        assert_eq!(*m2.get(), vec![1, 2, 3]);
        assert_eq!(m2.ref_count(), 1); // Now unique
    }

    #[test]
    fn managed_into_inner_unique() {
        let m = Managed::new("hello".to_string());
        let s = m.into_inner();
        assert_eq!(s, "hello");
    }

    #[test]
    fn managed_into_inner_shared() {
        let m1 = Managed::new("hello".to_string());
        let m2 = m1.share();

        let s = m1.into_inner();
        assert_eq!(s, "hello");

        // m2 still valid
        assert_eq!(*m2.get(), "hello");
    }

    #[test]
    fn managed_equality() {
        let m1 = Managed::new(42);
        let m2 = Managed::new(42);
        let m3 = Managed::new(99);
        let m4 = m1.share();

        assert_eq!(m1, m2); // Same value
        assert_ne!(m1, m3); // Different value
        assert_eq!(m1, m4); // Shared reference
    }

    #[test]
    fn managed_debug_format() {
        let m = Managed::new(42);
        let debug_str = format!("{:?}", m);
        assert!(debug_str.contains("Managed"));
        assert!(debug_str.contains("42"));
        assert!(debug_str.contains("ref_count"));
    }

    #[test]
    fn managed_nested() {
        let inner = Managed::new(vec![1, 2, 3]);
        let outer = Managed::new(inner);

        assert_eq!(outer.ref_count(), 1);
        assert_eq!(outer.get().ref_count(), 1);
    }

    #[test]
    fn managed_with_reference_types() {
        let m1 = Managed::new(Box::new(42));
        let m2 = m1.share();

        assert_eq!(**m1.get(), 42);
        assert_eq!(**m2.get(), 42);
    }

    #[test]
    fn managed_drop_when_unique() {
        let dropped = Rc::new(Cell::new(false));

        struct DropTracker(Rc<Cell<bool>>);
        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        {
            let _m = Managed::new(DropTracker(dropped.clone()));
        }

        assert!(dropped.get());
    }

    #[test]
    fn managed_no_drop_while_shared() {
        let dropped = Rc::new(Cell::new(false));

        struct DropTracker(Rc<Cell<bool>>);
        impl Drop for DropTracker {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }

        let m1 = Managed::new(DropTracker(dropped.clone()));
        let m2 = m1.share();

        drop(m1);
        assert!(!dropped.get()); // Still held by m2

        drop(m2);
        assert!(dropped.get()); // Now dropped
    }

    #[test]
    fn managed_stress_test_many_shares() {
        let m = Managed::new(42);
        let mut shares = Vec::new();

        for _ in 0..1000 {
            shares.push(m.share());
        }

        assert_eq!(m.ref_count(), 1001);

        shares.clear();
        assert!(m.is_unique());
    }

    #[test]
    fn managed_cow_pattern() {
        // Classic copy-on-write pattern
        let original = Managed::new(vec![1, 2, 3]);
        let mut copy = original.share();

        // Reading doesn't require copy
        assert_eq!(*copy.get(), vec![1, 2, 3]);

        // Writing triggers copy
        copy.get_mut_or_clone().push(4);

        // Now they're independent
        assert_eq!(*original.get(), vec![1, 2, 3]);
        assert_eq!(*copy.get(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn managed_chain_of_shares() {
        let m1 = Managed::new("test".to_string());
        let m2 = m1.share();
        let m3 = m2.share();
        let m4 = m3.share();

        assert_eq!(m1.ref_count(), 4);
        assert_eq!(m2.ref_count(), 4);
        assert_eq!(m3.ref_count(), 4);
        assert_eq!(m4.ref_count(), 4);
    }

    #[test]
    fn managed_modify_unique_then_share() {
        let mut m = Managed::new(vec![1]);

        m.try_get_mut().unwrap().push(2);
        m.try_get_mut().unwrap().push(3);

        let shared = m.share();

        assert_eq!(*m.get(), vec![1, 2, 3]);
        assert_eq!(*shared.get(), vec![1, 2, 3]);
    }

    #[test]
    fn managed_with_option() {
        let m = Managed::new(Some(42));
        assert_eq!(*m.get(), Some(42));

        let m_none: Managed<Option<i32>> = Managed::new(None);
        assert_eq!(*m_none.get(), None);
    }

    #[test]
    fn managed_with_result() {
        let m_ok: Managed<Result<i32, &str>> = Managed::new(Ok(42));
        let m_err: Managed<Result<i32, &str>> = Managed::new(Err("error"));

        assert!(m_ok.get().is_ok());
        assert!(m_err.get().is_err());
    }
}

// =============================================================================
// REGION TESTS
// =============================================================================

mod region_tests {
    use super::*;

    #[test]
    fn region_new_default() {
        let region = Region::new();
        assert!(region.total_capacity() >= 64 * 1024); // Default 64KB
        assert_eq!(region.drop_count(), 0);
    }

    #[test]
    fn region_with_custom_size() {
        let region = Region::with_chunk_size(1024);
        assert!(region.total_capacity() >= 1024);
    }

    #[test]
    fn region_alloc_returns_reference() {
        let mut region = Region::new();
        let x = region.alloc(42);
        assert_eq!(*x, 42);
    }

    #[test]
    fn region_alloc_multiple_values_separate() {
        // Avoid borrowck issues by checking values separately
        let mut region = Region::new();

        region.alloc(1);
        region.alloc(2);
        region.alloc(3);

        // Just verify no panic
    }

    #[test]
    fn region_alloc_handle() {
        let mut region = Region::new();
        let mut h = region.alloc_handle(vec![1, 2, 3]);

        assert_eq!(*h, vec![1, 2, 3]);
        h.push(4);
        assert_eq!(*h, vec![1, 2, 3, 4]);
    }

    #[test]
    fn region_handle_deref() {
        let mut region = Region::new();
        let h = region.alloc_handle("test".to_string());

        // Deref to &str methods
        assert_eq!(h.len(), 4);
        assert!(h.starts_with("te"));
    }

    #[test]
    fn region_handle_deref_mut() {
        let mut region = Region::new();
        let mut h = region.alloc_handle(vec![1, 2, 3]);

        h.push(4);
        h.push(5);

        assert_eq!(*h, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn region_drops_values_on_drop() {
        let dropped = Rc::new(Cell::new(0));

        struct DropCounter(Rc<Cell<i32>>);
        impl Drop for DropCounter {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        {
            let mut region = Region::new();
            region.alloc(DropCounter(dropped.clone()));
            region.alloc(DropCounter(dropped.clone()));
            region.alloc(DropCounter(dropped.clone()));

            assert_eq!(dropped.get(), 0);
        }

        assert_eq!(dropped.get(), 3);
    }

    #[test]
    fn region_drop_count_tracks_destructors() {
        let mut region = Region::new();

        assert_eq!(region.drop_count(), 0);

        region.alloc(String::from("needs drop"));
        assert_eq!(region.drop_count(), 1);

        region.alloc(42i32); // No drop needed for i32
        assert_eq!(region.drop_count(), 1);

        region.alloc(vec![1, 2, 3]); // Needs drop
        assert_eq!(region.drop_count(), 2);
    }

    #[test]
    fn region_multiple_chunks() {
        let mut region = Region::with_chunk_size(64);

        // Allocate more than fits in one chunk
        for i in 0..100 {
            region.alloc(i as i64);
        }

        // Just verify it doesn't panic
    }

    #[test]
    fn region_large_allocation() {
        let mut region = Region::with_chunk_size(64);

        // Allocate something larger than chunk size
        let big = region.alloc([0u8; 1024]);
        assert_eq!(big.len(), 1024);
    }

    #[test]
    fn region_handle_copy() {
        let mut region = Region::new();
        let h1 = region.alloc_handle(42);
        let h2 = h1; // Copy

        assert_eq!(*h1, 42);
        assert_eq!(*h2, 42);
    }

    #[test]
    fn region_default_trait() {
        let region: Region = Region::default();
        assert!(region.total_capacity() > 0);
    }

    #[test]
    fn region_stress_test() {
        let mut region = Region::new();

        for i in 0..10000 {
            region.alloc(i);
        }

        // Just verify it doesn't crash
    }

    #[test]
    fn region_mixed_allocations() {
        let mut region = Region::new();

        for i in 0..100 {
            if i % 3 == 0 {
                region.alloc(i as i32);
            } else if i % 3 == 1 {
                region.alloc(format!("string{}", i));
            } else {
                region.alloc(vec![i; i]);
            }
        }
    }

    #[test]
    fn region_empty_usage() {
        let region = Region::new();
        // Just create and drop - should not panic
        drop(region);
    }

    #[test]
    fn region_alloc_zst() {
        let mut region = Region::new();
        region.alloc(());
        // Should work with zero-sized types
    }

    #[test]
    fn region_handle_get_and_get_mut() {
        let mut region = Region::new();
        let mut h = region.alloc_handle(vec![1, 2]);

        // get_mut allows modification
        h.get_mut().push(3);

        // get returns reference
        assert_eq!(h.get(), &vec![1, 2, 3]);
    }
}

// =============================================================================
// WEAK REFERENCE TESTS
// =============================================================================

mod weak_tests {
    use super::*;

    #[test]
    fn weak_from_handle() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        assert!(weak.is_alive(&store));
    }

    #[test]
    fn weak_get_returns_value() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        assert_eq!(weak.get(&store), Some(&42));
    }

    #[test]
    fn weak_is_dead_after_release() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        store.release(handle);

        assert!(!weak.is_alive(&store));
        assert_eq!(weak.get(&store), None);
    }

    #[test]
    fn weak_upgrade_increments_refcount() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        assert_eq!(store.ref_count(&handle), Some(1));

        let upgraded = weak.upgrade(&mut store).unwrap();
        assert_eq!(store.ref_count(&handle), Some(2));

        store.release(upgraded);
        assert_eq!(store.ref_count(&handle), Some(1));
    }

    #[test]
    fn weak_upgrade_fails_after_release() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        store.release(handle);

        assert!(weak.upgrade(&mut store).is_none());
    }

    #[test]
    fn weak_keeps_value_alive_when_upgraded() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        let upgraded = weak.upgrade(&mut store).unwrap();

        // Release original
        store.release(handle);

        // Value still accessible via upgraded handle
        assert_eq!(store.get(&upgraded), Some(&42));

        // Weak can still see it
        assert!(weak.is_alive(&store));
    }

    #[test]
    fn weak_is_copy() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak1 = Weak::from_handle(&handle);
        let weak2 = weak1; // Copy
        let weak3 = weak1; // Copy again

        assert!(weak1.is_alive(&store));
        assert!(weak2.is_alive(&store));
        assert!(weak3.is_alive(&store));
    }

    #[test]
    fn weak_equality() {
        let mut store: Store<i32> = Store::new();
        let h1 = store.alloc(42);
        let h2 = store.alloc(42);

        let w1 = Weak::from_handle(&h1);
        let w2 = Weak::from_handle(&h1);
        let w3 = Weak::from_handle(&h2);

        assert_eq!(w1, w2);
        assert_ne!(w1, w3);
    }

    #[test]
    fn weak_hash() {
        use std::collections::HashSet;

        let mut store: Store<i32> = Store::new();
        let h1 = store.alloc(1);
        let h2 = store.alloc(2);

        let mut set: HashSet<Weak<i32>> = HashSet::new();
        set.insert(Weak::from_handle(&h1));
        set.insert(Weak::from_handle(&h2));
        set.insert(Weak::from_handle(&h1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn weak_debug_format() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        let debug_str = format!("{:?}", weak);
        assert!(debug_str.contains("Weak"));
    }

    #[test]
    fn weak_gen_index() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        assert_eq!(weak.gen_index(), handle.gen_index());
    }

    #[test]
    fn weak_detects_slot_reuse() {
        let mut store: Store<i32> = Store::new();

        let h1 = store.alloc(1);
        let weak = Weak::from_handle(&h1);

        store.release(h1);
        let _h2 = store.alloc(2); // Reuses slot

        // Weak reference is now invalid (generation mismatch)
        assert!(!weak.is_alive(&store));
        assert_eq!(weak.get(&store), None);
    }

    #[test]
    fn weak_multiple_upgrades() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak = Weak::from_handle(&handle);

        let u1 = weak.upgrade(&mut store).unwrap();
        let u2 = weak.upgrade(&mut store).unwrap();
        let u3 = weak.upgrade(&mut store).unwrap();

        assert_eq!(store.ref_count(&handle), Some(4)); // Original + 3 upgrades

        store.release(u1);
        store.release(u2);
        store.release(u3);

        assert_eq!(store.ref_count(&handle), Some(1));
    }

    #[test]
    fn weak_clone() {
        let mut store: Store<i32> = Store::new();
        let handle = store.alloc(42);
        let weak1 = Weak::from_handle(&handle);
        let weak2 = weak1.clone();

        assert_eq!(weak1, weak2);
        assert!(weak2.is_alive(&store));
    }
}

// =============================================================================
// INLINE TRAIT TESTS
// =============================================================================

mod inline_trait_tests {
    use super::*;

    #[test]
    fn primitives_are_inline() {
        // Just verify that these types implement Inline
        fn assert_inline<T: Inline>() {}

        assert_inline::<i8>();
        assert_inline::<i16>();
        assert_inline::<i32>();
        assert_inline::<i64>();
        assert_inline::<i128>();
        assert_inline::<isize>();
        assert_inline::<u8>();
        assert_inline::<u16>();
        assert_inline::<u32>();
        assert_inline::<u64>();
        assert_inline::<u128>();
        assert_inline::<usize>();
        assert_inline::<f32>();
        assert_inline::<f64>();
        assert_inline::<bool>();
        assert_inline::<char>();
        assert_inline::<()>();
    }

    #[test]
    fn tuples_are_inline() {
        fn assert_inline<T: Inline>() {}

        assert_inline::<(i32,)>();
        assert_inline::<(i32, i32)>();
        assert_inline::<(i32, f32, bool)>();
        assert_inline::<(u8, u8, u8, u8)>();
    }

    #[test]
    fn arrays_are_inline() {
        fn assert_inline<T: Inline>() {}

        assert_inline::<[i32; 0]>();
        assert_inline::<[i32; 1]>();
        assert_inline::<[i32; 100]>();
        assert_inline::<[f64; 4]>();
    }

    #[test]
    fn vec2_is_inline() {
        fn assert_inline<T: Inline>() {}
        assert_inline::<Vec2>();

        let v = Vec2::new(1.0, 2.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
    }

    #[test]
    fn vec3_is_inline() {
        fn assert_inline<T: Inline>() {}
        assert_inline::<Vec3>();

        let v = Vec3::new(1.0, 2.0, 3.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
    }

    #[test]
    fn vec4_is_inline() {
        fn assert_inline<T: Inline>() {}
        assert_inline::<Vec4>();

        let v = Vec4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
        assert_eq!(v.z, 3.0);
        assert_eq!(v.w, 4.0);
    }

    #[test]
    fn color_is_inline() {
        fn assert_inline<T: Inline>() {}
        assert_inline::<Color>();

        let c = Color::new(255, 128, 64, 255);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_rgb() {
        let c = Color::rgb(255, 0, 0);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 0);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_constants() {
        assert_eq!(Color::BLACK, Color::new(0, 0, 0, 255));
        assert_eq!(Color::WHITE, Color::new(255, 255, 255, 255));
        assert_eq!(Color::RED, Color::new(255, 0, 0, 255));
        assert_eq!(Color::GREEN, Color::new(0, 255, 0, 255));
        assert_eq!(Color::BLUE, Color::new(0, 0, 255, 255));
        assert_eq!(Color::TRANSPARENT, Color::new(0, 0, 0, 0));
    }

    #[test]
    fn vec2_constants() {
        assert_eq!(Vec2::ZERO, Vec2::new(0.0, 0.0));
        assert_eq!(Vec2::ONE, Vec2::new(1.0, 1.0));
    }

    #[test]
    fn vec3_constants() {
        assert_eq!(Vec3::ZERO, Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(Vec3::ONE, Vec3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn vec4_constants() {
        assert_eq!(Vec4::ZERO, Vec4::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(Vec4::ONE, Vec4::new(1.0, 1.0, 1.0, 1.0));
    }

    #[test]
    fn vec_default() {
        assert_eq!(Vec2::default(), Vec2::new(0.0, 0.0));
        assert_eq!(Vec3::default(), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(Vec4::default(), Vec4::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(Color::default(), Color::new(0, 0, 0, 0));
    }

    #[test]
    fn inline_types_are_copy() {
        let v1 = Vec2::new(1.0, 2.0);
        let v2 = v1; // Copy
        let v3 = v1; // Copy again

        assert_eq!(v1, v2);
        assert_eq!(v1, v3);
    }
}

// =============================================================================
// THREAD SAFETY TESTS
// =============================================================================

mod thread_safety_tests {
    use super::*;

    #[test]
    fn thread_safe_store_basic() {
        let mut store: ThreadSafeStore<i32> = ThreadSafeStore::new();
        let handle = store.alloc(42);
        assert_eq!(store.get(&handle), Some(&42));
    }

    #[test]
    fn thread_safe_store_retain_release() {
        let mut store: ThreadSafeStore<i32> = ThreadSafeStore::new();
        let handle = store.alloc(42);

        store.retain(&handle);
        assert_eq!(store.ref_count(&handle), Some(2));

        store.release(handle);
        assert_eq!(store.ref_count(&handle), Some(1));
    }

    #[test]
    fn single_thread_parallel_operations() {
        // Simulate parallel-like operations in a single thread
        let mut store: Store<i32> = Store::new();
        let mut handles = Vec::new();

        // Batch alloc
        for i in 0..100 {
            handles.push(store.alloc(i));
        }

        // Batch retain
        for h in &handles {
            store.retain(h);
        }

        // Verify all have ref_count 2
        for h in &handles {
            assert_eq!(store.ref_count(h), Some(2));
        }

        // Batch release
        for h in handles {
            store.release(h);
            store.release(h);
        }

        assert!(store.is_empty());
    }

    #[test]
    fn thread_safe_store_many_ops() {
        let mut store: ThreadSafeStore<String> = ThreadSafeStore::new();
        let mut handles = Vec::new();

        for i in 0..100 {
            handles.push(store.alloc(format!("item_{}", i)));
        }

        for h in &handles {
            store.retain(h);
        }

        assert_eq!(store.len(), 100);
    }
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn store_with_zero_sized_type() {
        let mut store: Store<()> = Store::new();
        let h1 = store.alloc(());
        let h2 = store.alloc(());

        assert_eq!(store.get(&h1), Some(&()));
        assert_eq!(store.get(&h2), Some(&()));
        assert_ne!(h1, h2);
    }

    #[test]
    fn managed_empty_vec() {
        let m = Managed::new(Vec::<i32>::new());
        assert!(m.get().is_empty());
    }

    #[test]
    fn managed_empty_string() {
        let m = Managed::new(String::new());
        assert!(m.get().is_empty());
    }

    #[test]
    fn store_many_generations() {
        let mut store: Store<i32> = Store::new();

        // Force many generation increments on the same slot
        for i in 0..1000 {
            let h = store.alloc(i);
            store.release(h);
        }

        // Should still work
        let final_h = store.alloc(9999);
        assert_eq!(store.get(&final_h), Some(&9999));
    }

    #[test]
    fn handle_with_large_type() {
        struct Large([u8; 10000]);

        let mut store: Store<Large> = Store::new();
        let h = store.alloc(Large([42; 10000]));

        assert_eq!(store.get(&h).unwrap().0[0], 42);
        assert_eq!(store.get(&h).unwrap().0[9999], 42);
    }

    #[test]
    fn managed_deeply_nested() {
        let m1 = Managed::new(1i32);
        let m2 = Managed::new(m1);
        let m3 = Managed::new(m2);
        let m4 = Managed::new(m3);

        // 4 levels deep - all should work
        // m4 -> Managed<Managed<Managed<Managed<i32>>>>
        // Need to unwrap each layer: .get() gives &inner, * dereferences
        // m4.get() -> &Managed<Managed<Managed<i32>>>
        // m4.get().get() -> &Managed<Managed<i32>>
        // m4.get().get().get() -> &Managed<i32>
        // m4.get().get().get().get() -> &i32
        assert_eq!(*m4.get().get().get().get(), 1);
    }

    #[test]
    fn store_rapid_realloc() {
        let mut store: Store<i32> = Store::new();

        for _ in 0..100 {
            let h1 = store.alloc(1);
            let h2 = store.alloc(2);
            let h3 = store.alloc(3);

            store.release(h2);

            let h4 = store.alloc(4); // Reuses h2's slot

            store.release(h1);
            store.release(h3);
            store.release(h4);
        }

        assert!(store.is_empty());
    }
}

// =============================================================================
// PERFORMANCE TESTS (not benchmarks, just sanity checks)
// =============================================================================

mod performance_sanity {
    use super::*;

    #[test]
    fn store_large_allocation() {
        let mut store: Store<String> = Store::new();
        let mut handles = Vec::new();

        for i in 0..10000 {
            handles.push(store.alloc(format!("item_{}", i)));
        }

        assert_eq!(store.len(), 10000);

        // Random access
        assert_eq!(
            store.get(&handles[5000]).map(|s| s.as_str()),
            Some("item_5000")
        );
    }

    #[test]
    fn managed_many_shares() {
        let original = Managed::new("test".to_string());
        let mut shares = Vec::new();

        for _ in 0..10000 {
            shares.push(original.share());
        }

        assert_eq!(original.ref_count(), 10001);

        // All point to same data
        for s in &shares {
            assert_eq!(*s.get(), "test");
        }
    }

    #[test]
    fn region_rapid_allocation() {
        let mut region = Region::new();

        for i in 0..10000 {
            region.alloc(i);
        }

        // Just verify it completed
    }

    #[test]
    fn store_churn() {
        let mut store: Store<i32> = Store::new();

        for _ in 0..100 {
            let mut handles = Vec::new();

            for i in 0..100 {
                handles.push(store.alloc(i));
            }

            for h in handles {
                store.release(h);
            }
        }

        assert!(store.is_empty());
    }
}

// =============================================================================
// GENERATION TYPE TESTS (via public API)
// =============================================================================

mod generation_via_public_api {
    use super::*;

    #[test]
    fn generation_increases_on_slot_reuse() {
        let mut store: Store<i32> = Store::new();

        let h1 = store.alloc(1);
        let gen1 = h1.gen_index().generation().value();

        store.release(h1);

        let h2 = store.alloc(2);
        let gen2 = h2.gen_index().generation().value();

        assert!(gen2 > gen1);
    }

    #[test]
    fn genindex_from_handle() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        let gen_idx = h.gen_index();

        // Should be able to get index and generation
        let _idx = gen_idx.index();
        let _gen = gen_idx.generation();
    }

    #[test]
    fn generation_debug_format() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        let gen = h.gen_index().generation();
        let debug_str = format!("{:?}", gen);

        assert!(debug_str.contains("Gen"));
    }

    #[test]
    fn genindex_debug_format() {
        let mut store: Store<i32> = Store::new();
        let h = store.alloc(42);

        let gen_idx = h.gen_index();
        let debug_str = format!("{:?}", gen_idx);

        assert!(debug_str.contains("GenIndex"));
    }
}
