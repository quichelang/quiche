# Agent Guidelines for Quiche

## Quick Start

```bash
# Build everything
make clean && make stage1 && make stage2

# Stage 1: Host compiler (Rust) compiles native compiler (Quiche)
# Stage 2: Native compiler (Stage 1 output) compiles itself
```

## Project Structure

- `crates/metaquiche-host/` - Rust host compiler (Stage 0)
- `crates/metaquiche-native/` - Quiche self-hosted compiler
- `crates/quiche-runtime/` - Runtime macros and traits
- `crates/quiche-parser/` - AST definitions
- `docs/language_design/` - Language specification

## Do's ✅

- **Run both stages** after changes: `make stage1 && make stage2`
- **Use `ref(x)`** for borrowing, not `as_ref(x)` (legacy)
- **Use `x: ref[i32]` and `x: mutref[i32]`** for borrowing instead of title-case variants
- **Use `deref(x)`** for dereferencing boxed values
- **Check `.qrs` files** for Quiche source code
- **Use jj** for version control (see [docs/jj-guide.md](docs/jj-guide.md))

## Don'ts ❌

- **Don't confuse `ref(x)` with `.as_ref()`** - former is Quiche operator, latter is Rust method
- **Don't edit generated `.rs` files** in `target/` - edit `.qrs` sources
- **Don't use `cargo test` directly** - it needs env vars; use make targets

## Reference Operators

| Quiche | Rust Output | Use |
|--------|-------------|-----|
| `ref(x)` | `qref!(x)` | Immutable borrow |
| `mutref(x)` | `mutref!(x)` | Mutable borrow |
| `deref(x)` | `deref!(x)` | Dereference |

## Key Files

| File | Purpose |
|------|---------|
| `codegen.qrs` | Main code generation logic |
| `type_utils.qrs` | Type inference utilities |
| `main.qrs` | CLI entry point |

## Common Issues

**Stage 2 fails but Stage 1 passes**: The native compiler generates different output than the host. Check that both compilers handle the construct the same way.

**"cannot find macro"**: Check imports in the crate's `lib.rs` or `main.rs` wrapper modules.

## Shared Templates

All code generation strings MUST be defined in `crates/metaquiche-shared/templates.toml`:

- **Project templates**: Cargo.toml, build.rs, wrapper files
- **Codegen templates**: Function definitions, docstrings, operators, literals
- **Runtime module**: Quiche macros (qref!, mutref!, deref!, strcat!)

Both host and native compilers must use these shared templates to ensure **byte-identical output** between stage1 and stage2. When adding new emit strings:

1. Add the template to `templates.toml` under `[codegen.your_new_template]`
2. Update both host (Rust) and native (Quiche) compilers to use the template
3. Verify with `make stage1 && make stage2` and diff the outputs

## Safe File Editing

When using `multi_replace_file_content` or `replace_file_content`:

- **View before edit**: Always `view_file` the exact lines first
- **Copy exact content**: Use the verbatim string from `view_file` as `TargetContent`
- **Prefer single-chunk edits**: Use `replace_file_content` over `multi_replace_file_content` when possible
- **Verify after edits**: Run `make stage1` after each batch to catch corruption early
- **Use AllowMultiple carefully**: Only for patterns you're certain appear identically

> **⚠️ Known Issue**: `multi_replace_file_content` with many chunks can cause content corruption (old+new values concatenated) if any `TargetContent` doesn't match exactly. When in doubt, use smaller batches with verification between each.

