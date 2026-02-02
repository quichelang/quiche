# Quiche

A Python-like language that compiles to Rust.

## Why

I wanted an expressive, embeddable language with native Rust interop. Something with compile-time safety and speed comparable to Rust itself.

I tried templating languages, dynamically loading compiled Rust modules, macro DSLs. Each had drawbacks. Macros came closest but add a debugging layer that's hard to work with. I even created Darcy, a Clojure-like language aiming for safe interop.

What started as a macro project kept growing. Adding proper checks meant writing a linter. I missed Python's rapid prototyping. Then I remembered Ruff was written in Rust. Could I use its parser? That gave me freedom to create a language with Python syntax but native Rust types.

After many iterations, we've rewritten almost everything. The language no longer depends on Ruff. Currently 16 dependencies, easily reducible further.

## Status

We have MetaQuiche, a Rust-compatible dialect:

- **Stage 0**: Bootstrap compiler written in pure Rust
- **Stage 1**: Post-AST logic rewritten in MetaQuiche
- **Stage 2**: MetaQuiche compiles its own source with identical output

Verify with `make verify`.

### What Works

- Native Rust library imports
- Traits, Structs, Enums (with discriminated types)
- Generics, Range syntax, Slices
- Destructuring
- All native Rust types (Vec, HashMap, Option, Result)
- Python-like `re` module for regex

### What's Missing

No standalone standard library (uses Rust libraries directly). No Python-style lists/dicts, no comprehensions. You can use `Vec.new().map(lambda x: ...)` instead. Example:

This MetaQuiche code

```python
nums: Vec[i32] = Vec.new()
nums.push(1)
nums.push(2)
doubled: Vec[i32] = nums.iter().map(lambda x: x * 2).collect()
```

Produces the following Rust code

```rust
let doubled: Vec<i32> = nums.iter().map((|x| x * 2)).collect();
```

## Future

- async/await with Tokio
- Multiprocessing via Rayon
- Native Polars support
- Experimental memory allocator (generational refcounting + regional allocation)
- Zero panics, 100% safe, systematic resource cleanup
- Automatic cloning (CoW)
- PyO3 interop for Python extensions
- JIT interpreter with Rust interop

## Quick Start

```bash
# Build and verify self-hosting
make clean && make verify

# Run a script
quiche script.qrs

# Create a new project
quiche new myproject
```

## License

MIT
