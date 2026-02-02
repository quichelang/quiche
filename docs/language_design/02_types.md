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

### Enums
Enums are defined as classes inheriting from `Enum`. Variants use assignment syntax with tuples.

```python
class Status(Enum):
    Pending = ()          # Unit variant → Status::Pending
    Active = ()           # Unit variant → Status::Active
    WithData = (String,)  # Tuple variant → Status::WithData(String)
```

Compiles to:
```rust
pub enum Status {
    Pending,
    Active,
    WithData(String),
}
```

> **Important:** Bare identifiers like `Pending` without `= ()` are NOT valid enum syntax.

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

## Generics

Quiche uses **Python 3.12 style** type parameters - square brackets after the name.

### Generic Structs

```python
class Point[T](Struct):
    x: T
    y: T
```

Compiles to:

```rust
pub struct Point<T> {
    pub x: T,
    pub y: T,
}
```

### Generic Functions

Type parameters come after the function name, before the parentheses:

```python
def first[T](items: ref[Vec[T]]) -> ref[T]:
    return ref(items[0])

def identity[T](x: T) -> T:
    return x
```

Compiles to:

```rust
pub fn first<T>(items: &Vec<T>) -> &T {
    &items[0]
}

pub fn identity<T>(x: T) -> T {
    x
}
```

### Trait Bounds

Constrain generics with trait bounds using colon syntax:

```python
def to_string[T: Display](value: T) -> String:
    return format("{}", value)

def compare[T: PartialOrd](a: T, b: T) -> bool:
    return a < b
```

Compiles to:

```rust
pub fn to_string<T: Display>(value: T) -> String {
    format!("{}", value)
}

pub fn compare<T: PartialOrd>(a: T, b: T) -> bool {
    a < b
}
```

## References

Quiche makes references and borrowing explicit but concise.

### Canonical Forms
- `Ref[T]`: Shared reference.
- `MutRef[T]`: Mutable reference.

### Short Forms (Preferred)
- `ref[T]`: Equivalent to `Ref[T]`.
- `mutref[T]`: Equivalent to `MutRef[T]`.

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
```python
def first(xs: ref[list[i32]]) -> ref[i32]:
    return ref(xs[0])
```

## Trait Objects

When you need dynamic dispatch (runtime polymorphism), use `Dyn[Trait]` to create trait objects.

### The Dyn Type Wrapper

| Quiche | Rust Output |
|--------|-------------|
| `Dyn[Display]` | `dyn Display` |
| `Ref[Dyn[Display]]` | `&dyn Display` |
| `Box[Dyn[Logger]]` | `Box<dyn Logger>` |
| `MutRef[Dyn[Writer]]` | `&mut dyn Writer` |

### Example: Trait Object Parameter

```python
def print_it(x: Ref[Dyn[Display]]) -> None:
    println!("{}", x)

def log_to(logger: Box[Dyn[Logger]]) -> None:
    logger.log("message")
```

Compiles to:

```rust
pub fn print_it(x: &dyn Display) {
    println!("{}", x);
}

pub fn log_to(logger: Box<dyn Logger>) {
    logger.log("message");
}
```

> **Note**: `Dyn` is composable with `Ref`, `MutRef`, and `Box`. Use `Ref[Dyn[T]]` for borrowed trait objects or `Box[Dyn[T]]` for owned ones.

