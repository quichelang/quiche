# Language Features

## What Works

### Core (Both Dialects)

- Native Rust types: `Vec`, `HashMap`, `Option`, `Result`, `String`, all integer/float types
- Traits, Structs, Enums (with unit, tuple, and named variants)
- Generics with trait bounds (`def foo[T: Display](x: T)`)
- Pattern matching with exhaustiveness checking and guards
- Lambda expressions (`|x, y| x + y`)
- Range syntax (`range(10)`, `range(5, 10)`, `range(0, 10, 2)`)
- Slice operators (`data[1..3]`, `data[2..]`, `data[..5]`)
- Module-level constants (`SCREAMING_CASE` or `Const[T]`)
- `@derive`, `@implement` decorators
- `Dyn[T]` trait objects
- Destructuring
- Rust library imports via `from rust.* import`

### Quiche Dialect (`.q` files)

- **Auto-borrowing** — compiler inserts `ref()`/`mutref()` automatically
- **List comprehensions** — `[x * 2 for x in nums]`
- **Dict comprehensions** — `{k.name: k for k in items}`
- **F-strings** — `f"Hello {name}"` and triple-quoted f-strings
- **Pythonic builtins** — `len()`, `print()`

### MetaQuiche Dialect (`.qrs` files)

- Explicit borrowing: `ref(x)`, `mutref(x)`, `deref(x)`
- Direct Rust type system access
- Used to implement the compiler itself

### Memory Management (`perceus-mem`)

- Generation-validated handles (prevents use-after-free)
- FBIP (Functional-But-In-Place) — in-place mutation when ref_count == 1
- Region-based allocation (arena-style bulk alloc/dealloc)
- Weak references for cycle prevention
- Policy-based threading (`SingleThreaded`, `ThreadSafe`)
- `Managed<T>` smart pointer wrapper
- `Store<T>` generational arena

### Runtime (`quiche-runtime`)

- Panic-free borrowing via `QuicheBorrow` trait
- Memory analyzer for escape analysis
- Test framework (`qtest`)
- Diagnostic emission (warnings, errors, notes)
- AST transformer for syntactic sugar
- Introspection / module registry

## Example: Quiche to Rust

This Quiche code:

```python
def main():
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    print(f"Result: {doubled}")
```

Produces Rust similar to:

```rust
fn main() {
    let nums: Vec<i32> = vec![1, 2, 3, 4, 5];
    let doubled: Vec<_> = nums.iter().map(|x| x * 2).collect();
    println!("Result: {:?}", doubled);
}
```

## What's Missing

- `try/except` exception handling
- Default function arguments
- `async/await`
- Threading / `@threadsafe` decorator
- `@macro` metaprogramming
- Pipe operators
- Simplified lifetime annotations
- JIT interpreter
- PyO3 interop

## Roadmap

See [status.md](status.md) for a detailed milestone breakdown.
