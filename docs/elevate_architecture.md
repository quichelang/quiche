# Elevate Architecture: Self-Hosted Quiche via Elevate IR

> The future of Quiche: a self-hosting compiler where every stage outputs
> Elevate IR, and Elevate handles the heavy lifting of type inference,
> ownership analysis, and Rust code generation.

## The Pipeline

All stages share the same backend. The compilers are thin **frontends**
that parse `.q` source and hand off a serialized AST to the shared bridge
crate. The bridge desugars into Elevate IR, and Elevate does the rest:

```
                    ┌──────────┐     ┌────────┐
  .q source ──────▶ │ Frontend │ ──▶ │ Bridge │ ──▶ Elevate IR ──▶ [Elevate] ──▶ Rust
                    └──────────┘     └────────┘
                     Stage 0: Rust    Quiche AST → Elevate IR
                     Stage 1+: Quiche (shared, single crate)
```

### Bootstrap Stages

| Stage | Frontend | Written In | Compiled By | Purpose |
|-------|----------|-----------|-------------|---------|
| **0** | `quiche-host` | Rust | `cargo build` | Bootstrap: compiles `.q` → Elevate IR |
| **1** | `quiche-compiler` | Quiche (`.q`) | Stage 0 | First self-compiled compiler |
| **2** | `quiche-compiler` | Quiche (`.q`) | Stage 1 | Verification build |

**Self-hosting check**: Stage 1 output == Stage 2 output.

The parser (`metaquiche-parser`) remains in Rust — a stable, well-tested
foundation shared by all stages. Each frontend only needs to **parse**.
The bridge handles desugaring, and Elevate handles everything downstream.

## Crate Map

### Keep

| Crate | Role |
|-------|------|
| `metaquiche-parser` | Shared parser (Rust). Handles `.q` and `.qrs` syntax. |
| `metaquiche-shared` | Templates, telemetry, i18n — shared utilities. |
| `quiche-elevate-bridge` | **Shared bridge**: Quiche AST → Elevate IR. Single source of desugaring truth. Both frontends call into this. |
| `elevate` | Backend: type inference, ownership analysis, Rust codegen. |
| `quiche-runtime` | Runtime macros and traits (`qref!`, `deref!`, etc.). |
| `perceus-mem` | Zero-cost automatic memory management. |
| `parsley` | Argument parser library for Quiche. |

### Rename

| Before | After | Change |
|--------|-------|--------|
| `metaquiche-host` | `quiche-host` | Retarget output from Rust → Elevate IR. Stage 0 frontend. |

### Delete

| Crate | Reason |
|-------|--------|
| `metaquiche-native` | Entirely replaced by Elevate. Its sole purpose was self-hosted Rust codegen. |
| `quiche-compiler/compiler/codegen.qrs` (77KB) | Legacy MetaQuiche codegen. Elevate replaces it. |
| `quiche-compiler/compiler/type_utils.qrs` | Used only by legacy codegen. |
| `quiche-compiler/compiler/mod.qrs` | Legacy module declaration. |
| `quiche-compiler/compiler/extern_defs.qrs` | Legacy extern definitions. |

### Evolve

| Crate | Change |
|-------|--------|
| `quiche-compiler` | Rewrite `main.qrs` → `main.q` (simplified syntax, auto-borrowing). CLI stays: `new`, `build`, `run`, `test`, `qtest`. |
| `quiche-elevate-bridge` | Promoted to shared crate. Owns all desugaring logic. Both frontends parse → serialize AST → call bridge → get Elevate IR. Eliminates duplicate `desugar.rs` in host and compiler. |

## How It Works

### The Bridge: Single Source of Truth

`quiche-elevate-bridge` owns all desugaring logic. It accepts a Quiche AST
(serialized via bincode for cross-process use) and outputs
Elevate IR. Both frontends call into this same crate:

```
  quiche-host (Rust)       ──┐
                             ├──▶ Quiche AST ──▶ [bridge] ──▶ Elevate IR ──▶ [Elevate] ──▶ Rust
  quiche-compiler (Quiche) ──┘
```

This eliminates duplicate desugaring code. The bridge already handles:
structs, enums, traits, impl blocks, functions, closures, f-strings,
list/dict comprehensions, range, imports, match expressions, and more.

With a serialized AST format, the bridge could even run as a separate
process — the Quiche-hosted compiler wouldn't need to link against Rust
code for desugaring at all.

### Stage 0: quiche-host (Rust)

The Rust-written frontend. Parses `.q`, calls the bridge:

```
.q file → metaquiche-parser → Quiche AST → [bridge] → Elevate IR → [Elevate] → Rust
```

### Stage 1+: quiche-compiler (Quiche)

The compiler itself, written in `.q`. Parses `.q`, calls the same bridge:

```
.q file → metaquiche-parser → Quiche AST → [bridge] → Elevate IR → [Elevate] → Rust
```

When Stage 1 compiles itself and produces the same Elevate IR as Stage 2,
self-hosting is verified.

## Migration Steps

### Phase 1: Retarget (minimal changes)
1. Rename `metaquiche-host` → `quiche-host`
2. Wire `quiche-host` to use `compile_via_elevate()` path instead of direct Rust codegen
3. Verify demo.q, sudoku.q, and CLI tests still pass

### Phase 2: Simplify
1. Convert `main.qrs` → `main.q` (drop `ref()`/`clone()`, use auto-borrowing)
2. Delete `compiler/codegen.qrs` and associated legacy files
3. Delete `metaquiche-native/` entirely
4. Update Makefile: `stage0` builds `quiche-host`, `stage1`/`stage2` build `quiche-compiler`

### Phase 3: Self-host
1. Rewrite `desugar.rs` in Quiche (`.q`) within `quiche-compiler`
2. Bootstrap: Stage 0 compiles the Quiche desugar, Stage 1 compiles itself
3. Verify Stage 1 IR == Stage 2 IR
4. 🎉

## Rustdex Integration

Elevate has **built-in knowledge** of common trait implementations:
- `Vec`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet` — `FromIterator`, `IntoIterator`
- Numeric types — `Add`, `Sub`, `Mul`, `Div`, etc.

For custom types or less common traits, Elevate optionally shells out to the
`rustdex` CLI (`rustdex check <type> <trait>`). This is:
- **Not a crate dependency** — zero transitive dependency risk
- **Gracefully degraded** — if `rustdex` isn't installed, Elevate falls back to `Unknown`
- **Narrowly scoped** — only used for associated trait checks (`from_iter`, `from`, `default`)

## Design Principles

1. **Elevate is the backend.** Frontends produce IR. Elevate handles the hard parts.
2. **The compiler is a thin frontend.** Parse → desugar → Elevate IR. That's it.
3. **Self-hosting via shared IR.** Both frontends speak the same Elevate IR.
   Verification is comparing IR output, not final Rust code.
4. **Parser stays in Rust.** A stable, well-tested foundation. Can be
   self-hosted later if desired, but there's no urgency.
5. **Incremental migration.** Each phase is independently valuable and testable.
