# Exploring Memory Management & Ergonomics

**Goal:** Transform explicit "MetaQuiche" into implicit "Quiche" where memory management and casting "just work."

## The Comparison

### Current (MetaQuiche)
Explicit borrowing (`ref`, `mutref`), dereferencing (`deref`), and casting (`as`).
```python
def solve(board: mutref[Vec[Vec[i32]]]) -> bool:
    row: i32 = find_empty_row(ref(deref(board)))
    row_idx: usize = row as usize
```

### Target (Quiche)
Implicit ownership, borrowing, and type coercion.
```python
def solve(board: Vec[Vec[i32]]) -> bool:
    row: i32 = find_empty_row(board)  # Auto-borrow
    row_idx: usize = row              # Auto-cast
```

---

## Strategy 1: Compiler Intelligence (Lobster Model)
*The compiler acts as a "Rust Whisperer", analyzing usage to insert safe Rust constructs automatically.*

**Concept:**
Lobster uses a compile-time ownership analysis algorithm (Lifetime Analysis) that determines when to borrow and when to move.

**Application to Quiche:**
1. **Signature Analysis:**
   - If a function expects a reference (`&T`), and we pass an owned `T`, the compiler emits `&T`.
   - If a function expects `&mut T`, and we have `T` (mut), emits `&mut T`.

2. **Flow-Sensitive Borrowing:**
   - Track variable usage. If a variable is used later, pass by reference.
   - If it's the last usage, pass by move.

3. **Advantages:**
   - Zero runtime cost (compiles to standard, efficient Rust).
   - Keeps full interoperability with Rust ecosystem.

4. **Challenges:**
   - Complexity: Replicating a borrow checker in the Quiche compiler to "predict" valid Rust.
   - Ambiguity: Indexing (`board[i]`) often needs explicit `&` vs `clone` decisions.

## Strategy 2: Automatic Reference Counting (Vala/Swift Model)
*Wrap types in smart pointers handled by the language runtime.*

**Concept:**
Vala looks like C# but compiles to C using GObject's reference counting. Swift uses ARC.

**Application to Quiche:**
1. **The `Object` Wrapper:**
   - All high-level Quiche types are implicitly `Rc<RefCell<T>>` (or a thread-safe custom `Gc<T>`).
   
   ```rust
   // Quiche source
   x: Vec[i32]
   
   // Compiles to
   let x: Gc<Vec<i32>> = Gc::new(Vec::new());
   ```

2. **Auto-Deref:**
   - Accessing methods on `Gc<T>` automatically borrows the inner value.
   - Passing `Gc<T>` strictly copies the pointer (cheap), not the data.

3. **Advantages:**
   - Extremely ergonomic (Python-like semantics).
   - "It just works" sharing of data.

4. **Challenges:**
   - Runtime overhead (ref counting, borrow checking at runtime).
   - Interop: Need to "unwrap" to pass to native Rust functions expecting `&str` or `&[T]`.

---

## The Hybrid Approach (Proposed)

We can likely achieve 90% of the benefit with 10% of the complexity by mixing explicitly simple rules.

### 1. Auto-Casting (Numeric Tower)
Rust is strict (`usize` != `i32`). Quiche can be lenient:
- **Implicit Widening:** `i32` -> `i64` is safe.
- **Index Coercion:** If a type is `i32` but used in `[...]`, implicit cast to `usize`.
- **Assignment Coercion:** `x: i32 = my_usize` -> emits `x: i32 = my_usize as i32`.

### 2. Auto-Borrowing Rules
Instead of full flow analysis:
- **Call-Site Inference:** Look at the function signature we are calling.
  - Fn expects `ref[T]` -> emit `&expr`.
  - Fn expects `mutref[T]` -> emit `&mut expr`.
  - Fn expects `T` -> emit `expr` (moves) or `expr.clone()` (if we detect reused variable).

### 3. The `View` Type
Introduce a `View` (slice) awareness.
- `Vec` is owned.
- Python logic often just wants to *read*.
- Default arguments to "view" (reference) unless "taking" (move) is explicit.
