# Quiche Language Design: Types & Memory

Quiche uses a **Type-First** design. Types are static and map directly to Rust types.

## Primitive Types

| Category | Types |
| :--- | :--- |
| **Integers** | `i8`, `i16`, `i32`, `i64`, `isize`, `u8`, `u16`, `u32`, `u64`, `usize` |
| **Floats** | `f32`, `f64` |
| **Boolean** | `bool` |
| **String** | `Str` (default for literals, backed by `Arc<str>`). `String` available for owned growable strings. |

## Collections

### List

| Quiche | Rust | Notes |
|--------|------|-------|
| `[1, 2, 3]` | `List::from(vec![1, 2, 3])` | `List[T]` wraps `Vec<T>` via `Deref`/`DerefMut` |
| `List[i64]` | `List<i64>` | All `Vec` methods available via deref |

### Dict

| Quiche | Rust | Notes |
|--------|------|-------|
| `{"a": 1}` | `Dict::from(HashMap::from(...))` | `Dict[K, V]` wraps `HashMap<K, V>` |
| `Dict[Str, i64]` | `Dict<Str, i64>` | All `HashMap` methods available via deref |

### Other Collections

| Type | Rust | Notes |
|------|------|-------|
| `Option[T]` | `Option<T>` | `Some(x)`, `None` |
| `Result[T, E]` | `Result<T, E>` | `Ok(x)`, `Err(e)` |
| `Tuple[A, B]` | `(A, B)` | Destructuring supported |

## User-Defined Types

### Structs

Structs are defined using the `type` keyword with field declarations:

```python
type Button:
    label: Str

type Point[T]:
    x: T
    y: T
```

### Enums

Enums are defined using the `type` keyword with variant assignments:

```python
type Status:
    Pending = ()          # Unit variant → Status::Pending
    Active = ()           # Unit variant → Status::Active
    WithData = (Str,)     # Tuple variant → Status::WithData(Str)
```

> **Important:** Variants require `= ()` for unit variants or `= (T,)` for tuple variants.

## Generics

Square brackets after the name (Python 3.12 style):

```python
type Point[T]:
    x: T
    y: T

def first[T](items: List[T]) -> T:
    return items[0]

def compare[T: PartialOrd](a: T, b: T) -> bool:
    return a < b
```

## Trait Objects

Dynamic dispatch via `Dyn[Trait]`:

| Quiche | Rust Output |
|--------|-------------|
| `Dyn[Display]` | `dyn Display` |
| `Ref[Dyn[Display]]` | `&dyn Display` |
| `Box[Dyn[Logger]]` | `Box<dyn Logger>` |
