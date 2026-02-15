# Elevate Architecture

> Quiche compiles `.q` source to Rust via the Elevate backend, which handles
> type inference, ownership analysis, and Rust code generation.

## Pipeline

```
.q source → Quiche Parser → Elevate IR → [Elevate] → Rust → rustc → binary
```

The Quiche compiler is a **thin frontend** — it parses `.q` files, desugars
Quiche-specific syntax (comprehensions, f-strings, auto-borrowing), and hands
off Elevate IR to the backend. Elevate handles the hard parts: type inference,
borrow analysis, clone insertion, and Rust codegen.

## Crate Map

| Crate | Role |
|-------|------|
| `quiche` | Frontend — parser, desugaring, CLI |
| `quiche-lib` | Runtime types — `Str`, `List[T]`, `Dict[K,V]`, `QuicheType` trait |
| `elevate` (git dep) | Backend — type inference, ownership, Rust codegen |

## How It Works

### Parser

The Quiche parser (`quiche/src/parser.rs`) is a hand-written recursive-descent
parser that produces Elevate AST nodes directly. It handles:

- Indentation-based blocks → brace blocks
- `type` keyword → structs and enums
- List/dict literals → `List::from(vec![...])` / `Dict::from(...)`
- F-strings → `format!()` macro calls
- Comprehensions → `.iter().map().collect()` chains
- `assert` → `assert!()` macro
- Auto-borrowing annotations

### Post-Processing

After compilation, `quiche/src/lib.rs` applies post-processing to the
generated Rust code:

- Wraps `vec![]` in `List::from()`
- Wraps `HashMap::from()` / `HashMap::from_iter()` in `Dict::from()`
- Rewrites `Vec<T>` → `List<T>`, `HashMap<K,V>` → `Dict<K,V>` in type annotations
- Injects `use quiche_lib::*` prelude

### Execution

The CLI compiles the final Rust code with `rustc`, linking against the
pre-built `libquiche_lib.rlib`, and runs the resulting binary.

## Design Principles

1. **Elevate is the backend.** The frontend produces IR. Elevate handles the hard parts.
2. **The compiler is a thin frontend.** Parse → desugar → Elevate IR. That's it.
3. **Parser stays in Rust.** A stable, well-tested foundation.
