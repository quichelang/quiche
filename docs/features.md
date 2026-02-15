# Language Features

## Core Language

- **Python syntax, Rust semantics** — indentation-based blocks, `def`, `type`, `match`
- **Static typing** with type inference — `x = 42` infers `i64`
- **Pattern matching** with exhaustiveness checking and guards
- **Generics** with trait bounds — `def foo[T: Display](x: T)`
- **Closures** — `|x: i64| x * 2`
- **Range** — `range(10)`, `range(5, 10)`, `range(0, 10, 2)`
- **Slices** — `data[1..3]`, `data[2..]`, `data[..5]`
- **Constants** — `SCREAMING_CASE` or `Const[T]` annotations
- **Decorators** — `@derive`, `@impl`
- **Trait objects** — `Dyn[T]`
- **Destructuring** — tuples and structs
- **Rust imports** — `from rust.* import`

## Type System

### Primitive Types

`Str` (string literals), `i64` (default integer), `f64`, `bool`

### Collection Types

| Quiche | Underlying | Literal |
|--------|-----------|---------|
| `List[T]` | `Vec<T>` via `Deref` | `[1, 2, 3]` |
| `Dict[K, V]` | `HashMap<K, V>` via `Deref` | `{"a": 1, "b": 2}` |
| `Option[T]` | `Option<T>` | `Some(x)`, `None` |
| `Result[T, E]` | `Result<T, E>` | `Ok(x)`, `Err(e)` |

### User-Defined Types

The `type` keyword defines structs and enums:

```python
type Point:
    x: i64
    y: i64

type Color:
    Red = ()
    Green = ()
    Blue = (i64,)
```

## Quiche Dialect Features

- **Auto-borrowing** — compiler inserts `ref()`/`mutref()` automatically
- **List comprehensions** — `[x * 2 for x in nums]`
- **Dict comprehensions** — `{k.name: k for k in items}`
- **F-strings** — `f"Hello {name}"` and triple-quoted f-strings
- **Pythonic builtins** — `len()`, `print()`

## Compilation

Quiche compiles `.q` source → Elevate IR → Rust → native binary. No VM, no GC.

```python
def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    print(f"Result: {doubled}")
```

Produces:

```rust
fn main() {
    let nums: List<i64> = List::from(vec![1, 2, 3, 4, 5]);
    let doubled: List<_> = nums.iter().map(|x| x * 2).collect();
    println!("Result: {:?}", doubled);
}
```

## What's Not Yet Implemented

- Default function arguments
- `async/await`
- Threading / `@threadsafe`
- `@macro` metaprogramming
- Pipe operators
- PyO3 interop
