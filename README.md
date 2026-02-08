# Quiche 🥧

**Write Python. Run Rust.**

Quiche is a language that looks and feels like Python but compiles to **100% safe Rust** — no `unsafe`, no runtime panics, no garbage collector. You get the clean syntax you love with the raw performance and safety guarantees you need.

```python
class Student(Struct):
    name: String
    age: u8

    def bio(self) -> String:
        return f"{self.name} is {self.age} years old"

def main():
    students = [
        Student(name="Alice", age=20),
        Student(name="Bob", age=22)
    ]
    for s in students:
        print(s.bio())
```

**That's it.** No lifetimes. No borrows. No `Arc<Mutex<T>>`. No fighting the compiler. Just write what you mean.

## Why?

Rust is an incredible language — but for quick scripts, prototypes, and simple programs, the learning curve is brutal:

| The pain | Quiche's answer |
|----------|----------------|
| Ownership & borrowing | **Automatic** — the compiler figures it out |
| Lifetime annotations | **None** — inferred at compile time |
| Type annotations | **Optional** — infer what you can, annotate what you want |
| `.clone()` everywhere | **Auto-cloning** — Quiche inserts clones only when needed |
| `&str` vs `String` | **Just `String`** — Quiche picks the right one |
| Macro syntax (`println!`) | **Just functions** — `print("hello")` |
| `match` exhaustiveness | **Handled** — Quiche generates correct patterns |

The output is **real Rust** — you can inspect it, learn from it, and ship it.

## Quick Start

```bash
# Install (requires Rust toolchain)
cargo build -p quiche --release

# Run a script
quiche hello.q

# See the generated Rust
quiche hello.q --emit-rust

# Inspect the parsed AST
quiche hello.q --emit-elevate
```

## What It Can Do

### Functions — just define them

```python
def add(a: i64, b: i64) -> i64:
    return a + b

def greet(name: String):
    print(f"Hello, {name}!")
```

### Structs — classes that just work

```python
class Point(Struct):
    x: i32
    y: i32

    def distance(self, other: Point) -> f64:
        dx: f64 = (self.x - other.x) as f64
        dy: f64 = (self.y - other.y) as f64
        return (dx * dx + dy * dy).sqrt()
```

### Enums — algebraic types, Python-style

```python
class Shape(Enum):
    Circle = (f64,)          # radius
    Rect = (f64, f64)        # width, height

class Printable(Trait):
    def describe(self) -> String: pass

@impl(Printable)
class Shape:
    def describe(self) -> String:
        match self:
            case Circle(r): return f"Circle r={r}"
            case Rect(w, h): return f"Rect {w}x{h}"
```

### Collections — Python syntax, Rust speed

```python
nums = [1, 2, 3, 4, 5]
doubled = [x * 2 for x in nums]
add = |x: i32, y: i32| x + y
```

### Imports — pull from the Rust ecosystem

```python
from std.collections import HashMap
from std.io import BufReader
```

## How It Works

```
hello.q  →  Quiche Parser  →  Elevate AST  →  Rust Codegen  →  rustc  →  binary
```

1. **Parse** — Quiche's hand-written recursive descent parser reads `.q` files
2. **Lower** — The [Elevate](https://github.com/jagtesh/elevate) compiler handles type inference, ownership analysis, and auto-cloning
3. **Emit** — Clean, idiomatic Rust is generated
4. **Compile** — Standard `rustc` produces an optimized native binary

The generated Rust is readable and correct — no `unsafe` blocks, no `unwrap()`, no panics.

## Experiment Flags

Quiche uses the Elevate compiler backend, which has several experimental features you can try:

```bash
quiche hello.q --exp-infer-local-bidi        # Bidirectional type inference
quiche hello.q --exp-numeric-coercion        # Auto numeric type coercion
quiche hello.q --exp-move-mut-args           # Mutable ownership transfer
quiche hello.q --fail-on-hot-clone           # Error on implicit clones
```

Run `quiche --help` for the full list.

## Project Status

| Feature | Status |
|---------|--------|
| Functions, structs, enums, traits | ✅ |
| Pattern matching | ✅ |
| List comprehensions | ✅ |
| F-strings | ✅ |
| Closures / lambdas | ✅ |
| Auto-borrowing & auto-cloning | ✅ |
| If/elif/else, for, while, match | ✅ |
| Imports from Rust stdlib | ✅ |
| Type inference | 🔄 Experimental |
| Crate-level compilation | 🔜 Next |
| Package manager integration | 🔜 Planned |

## Documentation

- **[Examples](examples/)** — Working `.q` scripts
- **[Tests](tests/)** — Test suite
- **[Language Design](docs/language_design/)** — Specification and design docs
- **[Editor Extensions](editors/)** — VSCode and Zed support

## License

BSD-3
