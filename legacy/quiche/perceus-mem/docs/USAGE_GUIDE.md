# Perceus-Mem Usage Guide

## Quick Decision Tree

```
┌─ Is it a small, frequently-copied value (Vec2, Color, etc.)?
│  └─ YES → Use `Inline` trait (zero overhead, stack-allocated)
│
├─ Do you need multiple values with the same short lifetime?
│  └─ YES → Use `Region` (arena allocation, bulk dealloc)
│
├─ Is it a graph/tree with potential cycles?
│  └─ YES → Use `Store<T>` + `Handle<T>` with `Weak<T>` for back-references
│
├─ Do you need functional-style updates (immutable interface, mutable when unique)?
│  └─ YES → Use `Managed<T>` (FBIP with copy-on-write)
│
└─ General reference-counted shared ownership?
   └─ Use `Store<T>` + `Handle<T>` (explicit store, generation-validated)
```

---

## Primitives Overview

### 1. `Inline` Types - Zero Overhead

**When**: Small, frequently-copied values (math vectors, colors, coordinates)

```rust
use perceus_mem::Inline;

#[derive(Clone, Copy)]
struct Vec2 { x: f32, y: f32 }
impl Inline for Vec2 {}

// Always copied, never heap-allocated
let a = Vec2 { x: 1.0, y: 2.0 };
let b = a; // Copy, not reference
```

**Heuristics for automation**:
- Size ≤ 16 bytes
- Type is `Copy`
- No heap allocations inside

---

### 2. `Region` - Arena Allocation

**When**: Many short-lived allocations with same lifetime (per-frame, per-request)

```rust
use perceus_mem::Region;

fn process_frame(data: &[u8]) {
    let mut region = Region::new();
    
    // All allocations freed together when region drops
    let parsed = region.alloc(parse(data));
    let transformed = region.alloc(transform(parsed));
    let output = region.alloc(render(transformed));
    
    // Use output...
} // All freed here, no individual drops
```

**Heuristics for automation**:
- Value doesn't escape the current scope/function
- Multiple allocations with same owner
- Performance-critical paths (avoids per-object dealloc)

---

### 3. `Store<T>` + `Handle<T>` - Generational References

**When**: Long-lived objects, graph structures, entity systems

```rust
use perceus_mem::{Store, Handle, Weak};

struct Node {
    value: i32,
    children: Vec<Handle<Node>>,
    parent: Option<Weak<Node>>, // Weak to prevent cycles
}

let mut store: Store<Node> = Store::new();
let root = store.alloc(Node { value: 0, children: vec![], parent: None });

// Safe access - handles are validated by generation
if let Some(node) = store.get(&root) {
    println!("Value: {}", node.value);
}

// FBIP: mutate in-place if unique reference
if let Some(node) = store.try_get_unique(&root) {
    node.value = 42; // No clone needed
}
```

**Heuristics for automation**:
- Value is stored in a collection/graph
- Value has references to other managed values
- Lifetime extends beyond current scope

---

### 4. `Managed<T>` - FBIP Copy-on-Write

**When**: Functional-style programming, persistent data structures

```rust
use perceus_mem::Managed;

fn append(list: Managed<Vec<i32>>, item: i32) -> Managed<Vec<i32>> {
    let mut list = list;
    
    // If we're the only reference, mutate in-place
    // Otherwise, clone automatically
    list.get_mut_or_clone().push(item);
    
    list
}

let a = Managed::new(vec![1, 2, 3]);
let b = a.share(); // Both point to same data
let c = append(a, 4); // a was moved, b still has [1,2,3], c has [1,2,3,4]
```

**Heuristics for automation**:
- Functional programming patterns
- Immutable-looking code that wants to avoid copies when possible
- Persistent data structures

---

### 5. `Weak<T>` - Cycle Prevention

**When**: Back-references, parent pointers, observer patterns

```rust
use perceus_mem::{Store, Handle, Weak};

let mut store: Store<i32> = Store::new();
let handle = store.alloc(42);
let weak = Weak::from_handle(&handle);

// Weak doesn't prevent deallocation
store.release(handle);
assert!(!weak.is_alive(&store));

// Can upgrade if still alive
if let Some(strong) = weak.upgrade(&mut store) {
    // Use strong handle...
}
```

---

## Automation Strategies

### A. Compiler-Based (AST Analysis)

The Quiche compiler can analyze the AST to choose the right primitive:

```python
# In Quiche source
class Node:
    value: int
    children: list[Node]  # Compiler sees self-reference → uses Store + Handle
    
def process(data: bytes) -> Result:
    parsed = parse(data)      # Doesn't escape → Region candidate
    return Result(parsed)     # Escapes → promote to Store
```

**Implementation in `ast_transformer.rs`**:

```rust
fn analyze_allocation(expr: &Expr, ctx: &Context) -> AllocationStrategy {
    // 1. Escape analysis
    if !escapes_scope(expr, ctx) {
        return AllocationStrategy::Region;
    }
    
    // 2. Type analysis
    let ty = ctx.type_of(expr);
    if ty.is_small_copy() {
        return AllocationStrategy::Inline;
    }
    
    // 3. Reference pattern analysis
    if has_cyclic_references(ty, ctx) {
        return AllocationStrategy::StoreWithWeak;
    }
    
    // 4. Default
    AllocationStrategy::Store
}
```

### B. Procedural Macro Approach

A `#[managed]` macro that analyzes the struct:

```rust
#[managed(auto)] // Let the macro decide
struct GameEntity {
    position: Vec2,        // → Inline (small, Copy)
    mesh: Mesh,            // → Store + Handle (large, shared)
    parent: Option<Self>,  // → Weak (back-reference)
}

// Expands to:
struct GameEntity {
    position: Vec2,                    // Inline
    mesh: Handle<Mesh>,                // From store
    parent: Option<Weak<GameEntity>>,  // Weak reference
}
```

### C. Runtime Heuristics (Hybrid)

Use a "smart" wrapper that starts simple and promotes:

```rust
pub enum Smart<T> {
    Inline(T),           // Small values
    Regional(RegionRef), // Short-lived
    Managed(Managed<T>), // Long-lived with COW
    Stored(Handle<T>),   // In a store
}

impl<T> Smart<T> {
    fn promote_if_needed(&mut self, ctx: &AllocationContext) {
        // Runtime decision based on:
        // - Access patterns
        // - Reference count
        // - Lifetime hints
    }
}
```

---

## Recommended Approach for Quiche

Given that Quiche already has an AST transformation layer, the cleanest approach is:

1. **Default to `Managed<T>`** for all heap allocations (gives FBIP automatically)
2. **Use escape analysis** to downgrade to `Region` for non-escaping values
3. **Use type analysis** to recognize `Inline` candidates
4. **Use cycle detection** to insert `Weak` for back-references

This can be done entirely at compile-time in the `ast_transformer.rs`, no runtime overhead.

```
┌─────────────────────────────────────────────────────────┐
│                 Quiche Source Code                      │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              AST Transformation Layer                   │
│  ┌─────────────────────────────────────────────────┐   │
│  │ Escape Analysis → Region/Store decision         │   │
│  │ Type Analysis   → Inline detection              │   │
│  │ Cycle Detection → Weak insertion                │   │
│  │ FBIP Analysis   → try_get_unique opportunities  │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│                    Rust Output                          │
│  (Uses correct perceus-mem primitive automatically)     │
└─────────────────────────────────────────────────────────┘
```
