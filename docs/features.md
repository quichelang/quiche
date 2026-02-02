# Language Features

## What Works

- Native Rust library imports
- Traits, Structs, Enums (with discriminated types)
- Generics, Range syntax, Slices
- Destructuring
- All native Rust types (Vec, HashMap, Option, Result)
- Python-like `re` module for regex
- Lambda expressions
- Pattern matching

## Example: MetaQuiche to Rust

This MetaQuiche code:

```python
nums: Vec[i32] = Vec.new()
nums.push(1)
nums.push(2)
doubled: Vec[i32] = nums.iter().map(lambda x: x * 2).collect()
```

Produces the following Rust:

```rust
let mut nums: Vec<i32> = Vec::new();
nums.push(1);
nums.push(2);
let doubled: Vec<i32> = nums.iter().map(|x| x * 2).collect();
```

## What's Missing

- No standalone standard library (uses Rust libraries directly)
- No Python-style lists/dicts (use `Vec`, `HashMap`)
- No list comprehensions (use `.iter().map().collect()`)

## Roadmap

- async/await with Tokio
- Multiprocessing via Rayon
- Native Polars support
- Experimental memory allocator (generational refcounting + regional allocation)
- Zero panics, 100% safe, systematic resource cleanup
- Automatic cloning (CoW)
- PyO3 interop for Python extensions
- JIT interpreter with Rust interop
