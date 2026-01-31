# Quiche Language Design: Types & Memory

Quiche uses a **Type-First** design. Unlike Python, types are static and map directly to Rust types.

## Native Types

| Category | Types |
| :--- | :--- |
| **Integers** | `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize` |
| **Floats** | `f32`, `f64` |
| **Boolean** | `bool` |
| **String** | `String` (Owned, default for literals). Borrowed `&str` requires manual conversion (or helpers) for now. |
| **Containers** | `Vec[T]`, `HashMap[K, V]`, `Option[T]`, `Result[T, E]` |
| **Tuples** | `Tuple[A, B]` maps to `(A, B)` |

## Collections

### Vectors
-   Syntax: `[1, 2, 3]` literal.
-   Type: `Vec[i32]`.
-   Compilation: `vec![1, 2, 3]`.

### HashMaps
-   Syntax: `{"k": v}`.
-   Type: `Map[String, i32]`.
-   Compilation: `std::collections::HashMap::from([...])`.

## Structs & Traits

### Structs
Structs are defined as Python classes inheriting from `Struct`. Rust derives are applied as metadata decorators.

```python
@derive("Debug", "Clone")
class Button(Struct):
    label: String
```

### Traits
Traits are defined as classes inheriting from `Trait`.

```python
class Drawable(Trait):
    def render(self, x: i32, y: i32) -> None: ...
```

### Trait Implementation
Implementations use the `@implement` decorator on a placeholder class (usually `class _`).

```python
@implement(Drawable, for_=Button)
class _:
    def render(self, x: i32, y: i32) -> None:
        # Implementation
        pass
```

## References & Lifetimes

Quiche makes references and borrowing explicit but concise.

### Canonical Forms
- `Ref[L, T]`: Shared reference with lifetime `L`.
- `MutRef[L, T]`: Mutable reference with lifetime `L`.

### Short Forms (Preferred)
- `ref[T]`: Equivalent to `Ref[_, T]` (inferred lifetime).
- `mutref[T]`: Equivalent to `MutRef[_, T]`.

### Borrowing & Dereferencing
Borrowing and dereferencing are explicit expressions, not hidden.

```python
mut n = 10

r = ref(n)        # Borrow immutable
m = mutref(n)     # Borrow mutable

val = deref(r)    # Read
deref(m) = 20      # Write
```

### Function Signatures
Lifetimes in signatures can be elided or explicit.

```python
# Implicit lifetimes
def first(xs: ref[list[int]]) -> ref[int]:
    return ref(xs[0])

# Explicit lifetimes
def first_explicit(L: type[L], xs: ref[L, list[int]]) -> ref[L, int]:
    return ref(xs[0])
```
