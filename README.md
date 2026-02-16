# Quiche 🥧

**Write Python. Run Rust.**

Quiche is a language that looks and feels like Python but compiles to **100% safe Rust** - no `unsafe`, no runtime panics, no garbage collector. You get the clean syntax you love with the raw performance and safety guarantees you need.

> Note: Quiche (via Elevate) depends on `rustdex`, which relies on rustdoc JSON features only available on the Rust **nightly** toolchain. Building or running Quiche/Elevate therefore requires the Rust nightly toolchain and the `rust-docs-json` component. Example setup:

```bash
rustup toolchain install nightly
rustup component add rust-docs-json --toolchain nightly
# use nightly when building/running: `cargo +nightly build` / `cargo +nightly run`

This repository includes a `rust-toolchain.toml` that requests `nightly` and the `rust-docs-json` component; running `cargo` inside the repo will pick that toolchain automatically.
```

```python
type Student:
    name: String
    age: i64

def bio(s: Student) -> String:
    return f"{s.name} is {s.age} years old"

def main():
    students = [
        Student(name="Alice", age=20),
        Student(name="Bob", age=22)
    ]
    for s in students:
        print(bio(s))
```

**That's it.** No lifetimes. No borrows. No `Arc<Mutex<T>>`. No fighting the compiler. Just write what you mean.

## Why?

Rust is an incredible language - but for quick scripts, prototypes, and simple programs, the learning curve is brutal:

| The pain | Quiche's answer |
|----------|----------------|
| Ownership & borrowing | **Automatic** - the compiler figures it out |
| Lifetime annotations | **None** - inferred at compile time |
| Type annotations | **Optional** - infer what you can, annotate what you want |
| `.clone()` everywhere | **Auto-cloning** - Quiche inserts clones only when needed |
| `&str` vs `String` | **Just `String`** - Quiche picks the right one |
| Macro syntax (`println!`) | **Just functions** - `print("hello")` |
| `match` exhaustiveness | **Handled** - Quiche generates correct patterns |

The output is **real Rust** - you can inspect it, learn from it, and ship it.

## Quick Start

```bash
# Install (requires Rust toolchain)
make install

# Run a script
quiche hello.q

# See the generated Rust
quiche hello.q --emit-rust

# View the Elevate IR as source
quiche hello.q --emit-elevate

# Dump the raw AST with metadata (debug)
quiche hello.q --emit-ast
```

## What It Can Do

### Functions - just define them

```python
def add(a: i64, b: i64) -> i64:
    return a + b

def greet(name: String):
    print(f"Hello, {name}!")
```

### Structs - types that just work

```python
type Point:
    x: i64
    y: i64

def distance(self: Point, other: Point) -> f64:
    dx: f64 = (self.x - other.x) as f64
    dy: f64 = (self.y - other.y) as f64
    return (dx * dx + dy * dy).sqrt()
```

### Enums - algebraic types, Python-style

```python
type Shape:
    Circle = (f64,)          # radius
    Rect = (f64, f64)        # width, height

def describe(s: Shape) -> String:
    match s:
        case Circle(r): return f"Circle r={r}"
        case Rect(w, h): return f"Rect {w}x{h}"
```

### Collections - Python syntax, Rust speed

```python
nums = [1, 2, 3, 4, 5]
doubled = [x * 2 for x in nums]
add = |x: i32, y: i32| x + y
```

### Imports - pull from the Rust ecosystem

```python
from std.collections import HashMap
from std.io import BufReader
```

### Row Polymorphism - duck typing that's actually safe

This is where Quiche shines. You can write generic functions that work on **any type with the right shape** - no interfaces, no trait bounds, no ceremony:

```python
type Dog:
    name: String

def speak_dog(d: Dog) -> String:
    return f"{d.name} says Woof!"

type Cat:
    name: String

def speak_cat(c: Cat) -> String:
    return f"{c.name} says Meow!"

# Generic functions with trait bounds
def check_eq[T: PartialEq](a: T, b: T) -> bool:
    return a == b

def main():
    print(speak_dog(Dog(name="Rex")))
    print(speak_cat(Cat(name="Whiskers")))
    print(check_eq(1, 1))
```

Elevate generates **specialized code per type** - there's zero runtime overhead. You get the flexibility of Python with the safety of Rust.

## Design Philosophy

Quiche uses a single `type` keyword for both structs and enums - the compiler infers the right Rust type from the shape of the definition. This is inspired by OCaml/F# and keeps the syntax minimal.

| Python | Quiche | Why |
|--------|--------|-----|
| Classes with inheritance | **`type` keyword (structs + enums)** | Composition over inheritance. No OOP class hierarchy. |
| Dynamic typing | **Static type inference** | Types are inferred where possible, annotated where needed. |
| Duck typing (runtime) | **Row polymorphism (compile-time)** | Same flexibility, but the compiler catches errors before you run. |
| Mutable by default | **Immutable by default** | Safer defaults. Quiche adds `mut` only where needed. |

Quiche is more **functional** than Python - influenced by Rust's algebraic types, pattern matching, and immutability. There's no `class` inheritance, no `super()`, no `__init__`. Instead you get structs with functions, enums with pattern matching, and generics with trait bounds.


## How It Works

```
hello.q  ->  Quiche Parser  ->  Elevate AST  ->  Rust Codegen  ->  rustc  ->  binary
```

1. **Parse** - Quiche's hand-written recursive descent parser reads `.q` files
2. **Lower** - The [Elevate](https://github.com/quichelang/elevate) compiler handles type inference, ownership analysis, and auto-cloning
3. **Emit** - Clean, idiomatic Rust is generated
4. **Compile** - Standard `rustc` produces an optimized native binary

The generated Rust is readable and correct - no `unsafe` blocks, no `unwrap()`, no panics.

## Experiment Flags

Quiche uses the Elevate compiler backend, which has several experimental features you can try:

```bash
quiche hello.q --exp-move-mut-args           # Mutable ownership transfer
quiche hello.q --exp-type-system             # Advanced type system features
quiche hello.q --fail-on-hot-clone           # Error on implicit clones
```

Run `quiche --help` for the full list.

## Project Status

| Feature | Status |
|---------|--------|
| Functions, structs, enums | ✅ |
| `type` keyword (structs + enums) | ✅ |
| Generics with trait bounds | ✅ |
| Pattern matching | ✅ |
| Closures (`\|x\| expr`) | ✅ |
| F-strings | ✅ |
| Auto-borrowing & auto-cloning | ✅ |
| If/elif/else, for, while, match | ✅ |
| Assert statements | ✅ |
| Imports from Rust stdlib | ✅ |
| Type inference | ✅ |
| Source-mapped diagnostics | ✅ |
| `--emit-elevate` (Elevate source) | ✅ |
| `--emit-ast` (debug AST dump) | ✅ |
| Editor plugins | 🔜 TBD |
| Crate-level compilation | 🔜 Next |
| Package manager integration | 🔜 Planned |

## Documentation

- **[Examples](examples/)** - Working `.q` scripts
- **[Tests](tests/)** - Test suite
- **[Language Design](docs/language_design/)** - Specification and design docs

## License

BSD-3
