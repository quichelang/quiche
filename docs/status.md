# Project Status — February 2026

> **State:** Quiche compiler is functional, producing working binaries via Elevate IR. The `.q` dialect supports Pythonic features with auto-borrowing. The `quiche-lib` crate provides `Str`, `List`, and `Dict` newtypes.

---

## Architecture

```
.q source → Quiche Parser → Elevate IR → Type Inference + Ownership → Rust → rustc
```

| Crate | Purpose |
|-------|---------|
| `quiche` | Compiler — parser, Elevate bridge, CLI (`run`, `build`, `init`, `test`) |
| `quiche-lib` | Runtime types — `Str`, `List[T]`, `Dict[K,V]`, `QuicheType` trait |
| `elevate` (git) | Backend — type inference, ownership analysis, Rust codegen |

---

## What Works

### Language Features

- **Type definitions** — `type Point:` for structs, `type Color:` with variants for enums
- **Auto-borrowing** — compiler inserts `ref()`/`mutref()` automatically
- **List comprehensions** — `[x * 2 for x in nums]`
- **Dict comprehensions** — `{k.name: k for k in items}`
- **F-strings** — `f"Hello {name}"` and triple-quoted
- **Pattern matching** — exhaustiveness checking, guards
- **Generics** — `def foo[T: Display](x: T)`
- **Closures** — `|x: i64| x * 2`
- **Constants** — `SCREAMING_CASE` or `Const[T]`
- **Assert** — `assert expr` and `assert expr, "message"`
- **Rust interop** — `from rust.* import`, inline `rust("""...""")` blocks

### Built-in Types

| Type | Backing | Literal |
|------|---------|---------|
| `Str` | `Arc<str>` | `"hello"` |
| `List[T]` | `Vec<T>` via Deref | `[1, 2, 3]` |
| `Dict[K, V]` | `HashMap<K, V>` via Deref | `{"a": 1}` |

### CLI Commands

| Command | Purpose |
|---------|---------|
| `quiche file.q` | Compile and run |
| `quiche build file.q` | Compile to Rust |
| `quiche init path` | Scaffold a project |
| `quiche test` | Run all `tests/*.q` files |
| `--emit-rust` | Show generated Rust |
| `--emit-elevate` | Show Elevate source |
| `--emit-ast` | Dump parsed AST |

### Test Suite

~28 test files covering: arithmetic, structs, enums, memory, strings, dicts, lists, comprehensions, f-strings, closures, ranges, tuples, control flow, subscripts, and more.

---

## What's Not Yet Implemented

| Feature | Status |
|---------|--------|
| Default function arguments | Not supported |
| `async/await` | Not started |
| Threading / `@threadsafe` | Proposed |
| `@macro` metaprogramming | Designed |
| Pipe operators | Explored |
| PyO3 interop | Not started |

---

## Known Issues

- Match arms with `return` in all branches not recognized by return checker
- Some Elevate edge cases with clone-vs-borrow heuristics
- `to_string()` on numeric types requires Elevate capability resolution
