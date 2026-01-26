# Quiche Language Design: Types & Memory

Quiche uses a **Type-First** design. Unlike Python, types are static and map directly to Rust types.

## Native Types

| Category | Types |
| :--- | :--- |
| **Integers** | `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize` |
| **Floats** | `f32`, `f64` |
| **Boolean** | `bool` |
| **String** | `String` (Owned), `str` (Slice, usually implicit) |
| **Containers** | `Vec[T]`, `HashMap[K, V]`, `Option[T]`, `Result[T, E]` |
| **Tuples** | `Tuple[A, B]` maps to `(A, B)` |

## Collections

### Vectors
-   Syntax: `[1, 2, 3]` literal.
-   Type: `Vec[i32]`.
-   Compilation: `vec![1, 2, 3]`.

### HashMaps
-   Syntax: `{"k": v}`.
-   Type: `Dict[String, i32]`.
-   Compilation: `std::collections::HashMap::from([...])`.

## Memory Management Strategy

Quiche aims to eliminate explicit lifetime annotations for the 90% case.

### 1. Tiered Strategy
-   **Copy Types** (`i32`, `f64`): Passed by copy. Zero overhead.
-   **Small Types** (`Vec`): Passed by move or cloned implicitly if needed (compiler insertion).
-   **Large Types**: Wrapped in smart pointers (`Rc` or `Arc`) if shared ownership is detected.

### 2. Ownership
-   Default behavior is **Move** (like Rust).
-   `x = y` moves strict ownership of `y` to `x`.
-   Use `.clone()` to duplicate.

### 3. Borrowing (Implicit)
-   Method calls `x.foo()` automatically borrow `&x` or `&mut x` based on the method signature.
-   Users rarely write `&x` explicitly, unless interfacing with foreign Rust code.
