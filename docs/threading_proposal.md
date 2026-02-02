# Multi-Threading Support Proposal for Perceus-Mem

## Status: PROPOSAL
**Author**: Claude (Antigravity)  
**Date**: 2026-02-02  
**Reviewers**: @jagtesh

---

## Executive Summary

This proposal outlines how to add multi-threading support to `perceus-mem` while maintaining Quiche's core constraint: **no panics allowed**. We leverage the existing `QuicheBorrow` trait pattern and extend it to work across threading policies.

## Current State

### Single-Threaded (Working)

Quiche already has panic-free borrowing via `QuicheBorrow`:

```rust
// quiche-runtime/src/lib.rs
pub trait QuicheBorrow<T> {
    fn try_borrow_q(&self) -> Result<Ref<T>, QuicheException>;
    fn try_borrow_mut_q(&self) -> Result<RefMut<T>, QuicheException>;
}
```

This wraps `RefCell::try_borrow()` and `RefCell::try_borrow_mut()` to return `Result` instead of panicking.

### What's Missing

No equivalent for multi-threaded contexts. Current `policy.rs` defines `ThreadSafe` with `Arc + AtomicU32` but lacks a panic-free cell abstraction.

---

## Proposed Architecture

### Core Principle: Policy-Based Interior Mutability

Extend `AtomicPolicy` to abstract over the cell type, unifying single-threaded and multi-threaded access patterns.

```
┌─────────────────────────────────────────────────────────────┐
│                     AtomicPolicy Trait                      │
├─────────────────────────┬───────────────────────────────────┤
│     SingleThreaded      │          ThreadSafe               │
├─────────────────────────┼───────────────────────────────────┤
│  Ptr = Rc<T>            │  Ptr = Arc<T>                     │
│  Counter = Cell<u32>    │  Counter = AtomicU32              │
│  Cell = RefCell<T>      │  Cell = parking_lot::RwLock<T>    │
│  via QuicheBorrow       │  via QuicheBorrowSync (NEW)       │
└─────────────────────────┴───────────────────────────────────┘
```

### New Trait: `QuicheBorrowSync`

A thread-safe equivalent of `QuicheBorrow`:

```rust
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Panic-free borrowing for multi-threaded contexts.
/// Uses parking_lot::RwLock which never poisons (no panics).
pub trait QuicheBorrowSync<T> {
    fn try_read_q(&self) -> Result<RwLockReadGuard<'_, T>, QuicheException>;
    fn try_write_q(&self) -> Result<RwLockWriteGuard<'_, T>, QuicheException>;
}

impl<T> QuicheBorrowSync<T> for RwLock<T> {
    fn try_read_q(&self) -> Result<RwLockReadGuard<'_, T>, QuicheException> {
        self.try_read()
            .ok_or_else(|| QuicheException("Read lock contention".into()))
    }
    
    fn try_write_q(&self) -> Result<RwLockWriteGuard<'_, T>, QuicheException> {
        self.try_write()
            .ok_or_else(|| QuicheException("Write lock contention".into()))
    }
}
```

### Why parking_lot?

| Feature | `std::RwLock` | `parking_lot::RwLock` |
|---------|---------------|------------------------|
| Poisoning | Yes (can panic) | No (never panics) |
| `try_*` methods | Returns `Result` | Returns `Option` |
| Performance | Good | Better (benchmarked) |
| Async compatible | Yes | Yes |
| Rayon compatible | Yes | Yes |

**parking_lot is the de-facto standard** for production Rust code that needs non-poisoning locks.

---

## Unified Trait: `CellLike`

To allow generic code over both policies:

```rust
pub trait CellLike<T> {
    type ReadGuard<'a>: Deref<Target = T> where T: 'a, Self: 'a;
    type WriteGuard<'a>: DerefMut<Target = T> where T: 'a, Self: 'a;
    
    fn new(val: T) -> Self;
    fn try_read(&self) -> Result<Self::ReadGuard<'_>, QuicheException>;
    fn try_write(&self) -> Result<Self::WriteGuard<'_>, QuicheException>;
}

// Single-threaded implementation
impl<T> CellLike<T> for RefCell<T> {
    type ReadGuard<'a> = Ref<'a, T> where T: 'a;
    type WriteGuard<'a> = RefMut<'a, T> where T: 'a;
    
    fn new(val: T) -> Self { RefCell::new(val) }
    
    fn try_read(&self) -> Result<Self::ReadGuard<'_>, QuicheException> {
        self.try_borrow()
            .map_err(|_| QuicheException("Already mutably borrowed".into()))
    }
    
    fn try_write(&self) -> Result<Self::WriteGuard<'_>, QuicheException> {
        self.try_borrow_mut()
            .map_err(|_| QuicheException("Already borrowed".into()))
    }
}

// Multi-threaded implementation
impl<T> CellLike<T> for parking_lot::RwLock<T> {
    type ReadGuard<'a> = RwLockReadGuard<'a, T> where T: 'a;
    type WriteGuard<'a> = RwLockWriteGuard<'a, T> where T: 'a;
    
    fn new(val: T) -> Self { RwLock::new(val) }
    
    fn try_read(&self) -> Result<Self::ReadGuard<'_>, QuicheException> {
        self.try_read()
            .ok_or_else(|| QuicheException("Read lock contention".into()))
    }
    
    fn try_write(&self) -> Result<Self::WriteGuard<'_>, QuicheException> {
        self.try_write()
            .ok_or_else(|| QuicheException("Write lock contention".into()))
    }
}
```

---

## Extended AtomicPolicy

```rust
pub trait AtomicPolicy: 'static {
    /// Reference-counted pointer (Rc or Arc)
    type Ptr<T: 'static>: Clone;
    
    /// Counter for reference counting
    type Counter: Counter;
    
    /// Interior mutability cell (RefCell or RwLock)
    type Cell<T>: CellLike<T>;
    
    fn new_counter(initial: u32) -> Self::Counter;
    fn new_cell<T>(val: T) -> Self::Cell<T>;
}

// Single-threaded policy
impl AtomicPolicy for SingleThreaded {
    type Ptr<T: 'static> = Rc<T>;
    type Counter = Cell<u32>;
    type Cell<T> = RefCell<T>;
    
    fn new_counter(initial: u32) -> Self::Counter { Cell::new(initial) }
    fn new_cell<T>(val: T) -> Self::Cell<T> { RefCell::new(val) }
}

// Thread-safe policy
impl AtomicPolicy for ThreadSafe {
    type Ptr<T: 'static> = Arc<T>;
    type Counter = AtomicU32;
    type Cell<T> = parking_lot::RwLock<T>;
    
    fn new_counter(initial: u32) -> Self::Counter { AtomicU32::new(initial) }
    fn new_cell<T>(val: T) -> Self::Cell<T> { RwLock::new(val) }
}
```

---

## Managed<T, P> with Policy Support

```rust
pub struct Managed<T, P: AtomicPolicy = SingleThreaded> {
    inner: P::Ptr<ManagedInner<T, P>>,
}

struct ManagedInner<T, P: AtomicPolicy> {
    data: P::Cell<T>,
    refcount: P::Counter,
}

impl<T, P: AtomicPolicy> Managed<T, P> {
    pub fn new(val: T) -> Self {
        Self {
            inner: /* wrap in P::Ptr */ {
                ManagedInner {
                    data: P::new_cell(val),
                    refcount: P::new_counter(1),
                }
            }
        }
    }
    
    /// Read access (shared borrow)
    pub fn read(&self) -> Result<impl Deref<Target = T> + '_, QuicheException> {
        self.inner.data.try_read()
    }
    
    /// Write access (exclusive borrow) with FBIP semantics
    pub fn write(&self) -> Result<impl DerefMut<Target = T> + '_, QuicheException> {
        // If refcount > 1, clone first (FBIP)
        // Then return write guard
        self.inner.data.try_write()
    }
}
```

---

## Codegen Integration

### Mode Detection

Codegen in `codegen.qrs` detects threading mode via:

1. **Explicit decorator**: `@threadsafe`
2. **Async context**: `async def` functions
3. **Parallel primitives**: `parallel for`, `spawn`

### Generated Code

```python
# Quiche source
@threadsafe
def process_data(items):
    for item in items:
        item.value += 1
```

```rust
// Generated (ThreadSafe policy)
fn process_data(items: Managed<Vec<Item>, ThreadSafe>) -> Result<(), QuicheException> {
    for item in items.read()?.iter() {
        // Read-only iteration
    }
    // Or with mutation:
    for item in items.write()?.iter_mut() {
        item.value += 1;
    }
    Ok(())
}
```

### Default Behavior

| Context | Policy | Rationale |
|---------|--------|-----------|
| Regular functions | `SingleThreaded` | Zero overhead, no locks |
| `@threadsafe` functions | `ThreadSafe` | Explicit opt-in |
| `async def` | `ThreadSafe` | Tokio may move tasks |
| `parallel for` | `ThreadSafe` | Rayon work-stealing |

---

## Rayon/Tokio Compatibility

### Rayon (Data Parallelism)

```rust
use rayon::prelude::*;

// Managed<Vec<T>, ThreadSafe> is Send + Sync (via Arc + RwLock)
let data: Managed<Vec<i32>, ThreadSafe> = Managed::new(vec![1, 2, 3, 4]);

// Safe parallel iteration
data.read()?.par_iter().for_each(|x| println!("{}", x));
```

### Tokio (Async)

```rust
use tokio;

let data: Managed<Vec<i32>, ThreadSafe> = Managed::new(vec![1, 2, 3]);

tokio::spawn(async move {
    // data can be moved into async task
    let guard = data.read()?;
    println!("{:?}", *guard);
    Ok::<_, QuicheException>(())
});
```

Both work because `Arc<RwLock<T>>` is `Send + Sync`.

---

## Implementation Plan

### Phase 1: Add parking_lot Dependency
- [ ] Add `parking_lot = "0.12"` to `perceus-mem/Cargo.toml`
- [ ] Verify build passes

### Phase 2: Define CellLike Trait
- [ ] Create `src/cell.rs` with `CellLike` trait
- [ ] Implement for `RefCell<T>`
- [ ] Implement for `parking_lot::RwLock<T>`

### Phase 3: Extend AtomicPolicy
- [ ] Add `type Cell<T>` to `AtomicPolicy` trait
- [ ] Update `SingleThreaded` implementation
- [ ] Update `ThreadSafe` implementation

### Phase 4: Update Managed<T, P>
- [ ] Add policy type parameter
- [ ] Use `P::Cell<T>` for internal storage
- [ ] Implement `read()` / `write()` methods

### Phase 5: Codegen Integration
- [ ] Add `@threadsafe` decorator detection
- [ ] Emit correct policy type based on context
- [ ] Handle async/parallel primitives

### Phase 6: Testing
- [ ] Unit tests for both policies
- [ ] Integration test with Rayon
- [ ] Integration test with Tokio
- [ ] Stress tests for contention handling

---

## Dependencies

```toml
[dependencies]
parking_lot = "0.12"  # ~100KB, minimal transitive deps
```

**parking_lot is used by**: tokio, rayon, crossbeam, serde, and many other production crates.

---

## Open Questions

1. **Lock contention handling**: Should `try_read`/`try_write` block briefly before returning error? (Timeout option?)

2. **Deadlock prevention**: Should we add deadlock detection in debug builds?

3. **Performance monitoring**: Should we track lock contention metrics for profiling?

---

## Summary

| Feature | Single-Threaded | Multi-Threaded |
|---------|-----------------|----------------|
| Pointer | `Rc<T>` | `Arc<T>` |
| Counter | `Cell<u32>` | `AtomicU32` |
| Cell | `RefCell<T>` | `parking_lot::RwLock<T>` |
| Panics | Never (via `QuicheBorrow`) | Never (no poisoning) |
| Overhead | Zero | Minimal |
| Rayon | N/A | ✅ Compatible |
| Tokio | N/A | ✅ Compatible |

This architecture provides:
- **100% safe Rust** (no `unsafe` in perceus-mem)
- **Zero panics** in both modes
- **Unified API** via `CellLike` trait
- **Async/parallel compatibility** via parking_lot
- **Codegen transparency** (same patterns, different policies)
