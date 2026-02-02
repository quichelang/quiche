# Quiche

A Python-like language that compiles to Rust.

> [!IMPORTANT]
> Quiche is an experimental language exploring how far safe Rust can be stretched to achieve Python-like expressiveness without sacrificing Rust-level performance.

## Why

I wanted an expressive, embeddable language with native Rust interop. Something with compile-time safety and speed comparable to Rust itself.

I tried templating languages, dynamically loading compiled Rust modules, macro DSLs. Each had drawbacks. Macros came closest but add a debugging layer that's hard to work with. I even created Darcy, a Clojure-like language aiming for safe interop.

What started as a macro project kept growing. Adding proper checks meant writing a linter. I missed Python's rapid prototyping. Then I remembered Ruff was written in Rust. Could I use its parser? That gave me freedom to create a language with Python syntax but native Rust types.

After many iterations, we've rewritten almost everything. The language no longer depends on Ruff. Currently 16 dependencies, easily reducible further.

## Design Philosophy

Rust -> MetaQuiche -> Quiche

- MetaQuiche: A lower level dialect for implementing core language features
    - Can interface with Rust code
    - Supports most Rust primitives including Traits, Structs, Enums, Generics
    - Compile-time safety
    - No reliance on runtime checks
    - "Panics" are compile-time errors
    - Python-compatible syntax
    - Fast compilation

- Quiche: A higher level dialect for implementing application features
    - Can interface with MetaQuiche code
    - Supports a subset of MetaQuiche features
    - Builds on top of MetaQuiche's safety guarantees
    - Automatic memory management and borrowing rules
    - Python compatibility layer (lists, dicts, stdlib functions, builtins, etc.)
    - Ability to write Python libraries

## Status

We have **MetaQuiche**, a Rust-compatible dialect. Current progress:

| Phase | Description | Status |
|-------|-------------|--------|
| 1. Bootstrap | Host compiler written in pure Rust | ✅ Complete |
| 2. Self-hosting | Compiler rewritten in MetaQuiche, compiles itself | ✅ Complete |
| 3. Zero dependencies | Remove external crates | 🔄 In progress |
| 4. Quiche language | Higher-level dialect with managed memory | 📋 Planned |

### Self-Hosting Stages

The compiler builds itself in three stages:

```
Stage 0 (Rust)  →  Stage 1 (MetaQuiche)  →  Stage 2 (Self-compiled)
     ↓                    ↓                        ↓
 quiche-host         compiles to            byte-identical
                    native binary            to Stage 1
```

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

### Building

```bash
# Build the compiler (debug, with verification)
make

# Or build step-by-step:
make stage1    # Build with host compiler
make stage2    # Build with stage1 (self-hosted)
make verify    # Verify stage1 == stage2 output
```

After building, the `quiche` binary is available at:
- `./bin/quiche` — symlink to latest build
- `./stage2` — symlink to stage2 debug build

### Running Scripts

```bash
# Run a Quiche script
./bin/quiche script.qrs

# Or directly with stage2
./stage2 examples/scripts/sudoku.qrs
```

### Release Build

```bash
# Build optimized release binary
make release

# Install to /usr/local/bin (requires sudo)
make install
```

### Creating Projects

```bash
# Create a new Quiche project
./bin/quiche new myproject
cd myproject
./bin/quiche build
./bin/quiche run
```

### Make Targets

| Target | Description |
|--------|-------------|
| `make` | Build and verify (default) |
| `make stage1` | Build with host compiler |
| `make stage2` | Build self-hosted compiler |
| `make verify` | Verify stage1/stage2 parity |
| `make release` | Build optimized release binary |
| `make install` | Install to /usr/local/bin |
| `make clean` | Remove all build artifacts |

## License

BSD-3
