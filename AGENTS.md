# Agent Guidelines for Quiche

We play by the scout's rules. Always leave the place cleaner than you found it.

If you discover a core feature is missing/broken in the compiler, build it. We will FORGE the change we want to see!

## Quick Start

```bash
# Build everything
make clean && make stage1 && make stage2

# Stage 1: Host compiler (Rust) compiles native compiler (Quiche)
# Stage 2: Native compiler (Stage 1 output) compiles itself
```

## Project Structure

| Directory | Purpose |
|-----------|---------|
| `metaquiche/metaquiche-host/` | Rust host compiler (Stage 0) |
| `metaquiche/metaquiche-native/` | Quiche self-hosted compiler |
| `metaquiche/metaquiche-parser/` | Parser and AST definitions |
| `quiche/quiche-runtime/` | Runtime macros and traits |
| `quiche/quiche-compiler/` | Quiche compiler (copy of native) |
| `quiche/parsley/` | Parser library for Quiche |
| `quiche/perceus-mem/` | Memory management library |
| `docs/language_design/` | Language specification |

## Key Files

| File | Purpose |
|------|---------|
| `codegen.qrs` | Main code generation logic |
| `type_utils.qrs` | Type inference utilities |
| `main.qrs` | CLI entry point |

## Do's ✅

- **Run both stages** after changes: `make stage1 && make stage2`
- **Run regression tests**: `make test`
- **Use `ref(x)`** for borrowing, not `as_ref(x)` (legacy)
- **Use `deref(x)`** for dereferencing boxed values
- **Check `.qrs` files** for Quiche source code
- **Use jj** for version control (see [docs/jj-guide.md](docs/jj-guide.md))
- **One commit per feature**: After each logical feature, run `jj new -m "description"` to start a fresh change. Do NOT keep amending the same working copy with `jj describe`.
- **Split if needed**: Use `JJ_EDITOR=true jj split -r <rev> -m "msg" -- <fileglobs>` for non-interactive file-based splits.
- **Never monolith**: A commit with 5+ unrelated changes must be split before moving on.

## Don'ts ❌

- **Don't confuse `ref(x)` with `.as_ref()`** — former is Quiche operator, latter is Rust method
- **Don't edit generated `.rs` files** in `target/` — edit `.qrs` sources
- **Don't use `cargo test` directly** — it needs env vars; use make targets

## Common Issues

| Problem | Solution |
|---------|----------|
| Stage 2 fails but Stage 1 passes | Native/host codegen mismatch. Check both compilers. |
| "cannot find macro" | Check imports in `lib.rs` or `main.rs` wrapper modules. |

---

# Language Reference

## Reference Operators

| Quiche | Rust Output | Use |
|--------|-------------|-----|
| `ref(x)` | `qref!(x)` | Immutable borrow |
| `mutref(x)` | `mutref!(x)` | Mutable borrow |
| `deref(x)` | `deref!(x)` | Dereference |

## Syntax: Python → MetaQuiche

| Python | MetaQuiche | Notes |
|--------|------------|-------|
| `x = 42` | `x: i32 = 42` | Type annotations required |
| `def foo(x):` | `def foo(x: i32) -> i32:` | Types required |
| `list[int]` | `Vec[i32]` | Rust types, square bracket generics |
| `&x` | `ref(x)` | Borrow reference |
| `*x` | `deref(x)` | Dereference |
| `x[1:3]` | `x[1..3]` | Slice syntax |
| `range(10)` | `range(10)` | → `0..10` |

## Module-Level Constants

```python
# Auto-const via SCREAMING_SNAKE_CASE
MAX_SIZE: i32 = 100          # → pub const MAX_SIZE: i32 = 100;

# Explicit via Const[T] type annotation
config_ver: Const[i32] = 1   # → pub const config_ver: i32 = 1;
```

## Struct/Enum/Trait

```python
class Point(Struct):
    x: i32
    y: i32

class Color(Enum):
    Red = ()           # Unit variant
    Green = (i32,)     # Tuple variant with payload

class Display(Trait):
    def show(self) -> String: pass

@impl(Display)
class Point:
    def show(self) -> String:
        return strcat(self.x.to_string(), ",", self.y.to_string())
```

---

# Advanced Topics

## Shared Templates

All codegen strings MUST be in `metaquiche/metaquiche-shared/templates/`:

1. Add template to appropriate `.toml` file under `[codegen.your_new_template]`
2. Update both host (Rust) and native (Quiche) compilers
3. Verify with `make stage1 && make stage2 && make verify`

## Safe File Editing

- **View before edit**: Always `view_file` the exact lines first
- **Prefer single-chunk edits**: Use `replace_file_content` when possible
- **Verify after edits**: Run `make stage1` after each batch

> **⚠️ Known Issue**: `multi_replace_file_content` with many chunks can cause corruption if `TargetContent` doesn't match exactly.

## MetaQuiche Limitations

| Feature | Status | Workaround |
|---------|--------|------------|
| List comprehensions | ❌ | `.iter().map().collect()` |
| `try/except` | ❌ | `Result<T, E>` + `.unwrap_or()` |
| Default args | ❌ | Overloaded functions |
| f-strings | ❌ | `strcat!()` macro |

## Quiche Language (`.q` files)

Higher-level dialect with auto-borrowing. Files use `.q` extension.

| Quiche | MetaQuiche |
|--------|------------|
| `find_empty(board)` | `find_empty(ref(deref(board)))` |
| `board[i][j] = val` | `deref(board)[i][j] = val` |
| `[1, 2, 3]` | `vec![1, 2, 3]` |

- **Auto-borrowing**: Compiler inserts `ref()`/`mutref()` as needed
- **No cloning/mut**: Do not add `.clone()` or `mutref()` in the `.q` files. The compiler will add them as needed.
- **Memory Management**: Perceus-mem for zero-cost automatic memory management
