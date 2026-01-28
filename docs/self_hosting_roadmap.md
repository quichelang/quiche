# Quiche Self-Hosting Roadmap

## Objective
Transition `quiche` from a Rust-hosted compiler to a self-hosted compiler (`quiche` written in `quiche`).

> [!NOTE]
> **Honesty Check**: We currently rely on `ruff_python_parser` (Rust) for parsing. We are creating a "Logic Self-Hosted" compiler, not a "Purist Self-Hosted" compiler (yet).

## Strategy: The Bootstrapping Levels

### Level 0: Logic Self-Hosting (Current Goal)
**Definition**: The compiler *logic* (AST -> Rust text) is written in Quiche, but the *parser* (Text -> AST) relies on the existing Rust crate `ruff_python_parser`.
- **Why this is useful**: It validates that Quiche is expressive enough to handle complex recursion, scope management, and type checking.
- **Value Proposition**: "Write compiler logic like Python, run it like Rust."
    - **Developer Experience**: Write complex tree traversals without fighting the borrow checker (handled by `Rc`/cloning in the transpiler layer).
    - **Safety**: Errors are automatically propagated via `check!`, reducing boilerplate `?` handling.
    - **Speed**: The output is still native Rust.

### Level 1: Parser Self-Hosting (Future Goal)
**Definition**: The parser itself is rewritten in Quiche.
- **Why**: Removes the dependency on `ruff` (except perhaps for the very first bootstrap stage).
- **Challenge**: Parsing is performance-critical and complex.
- **Status**: Out of scope for now. We explicitly accept the "cheat" of using a Rust parser to focus on proving the Compiler DX first.

## The Bootstrapping Cycle (Level 0)

### Phase 1: The "Dogfood" Test (Current State)
We currently verify `quiche-self` by compiling it with the *host* compiler (Rust implementation) and running unit tests against the generated Rust code.
- **Status**: Active. `cargo test -p quiche_self` verifies the transpiled logic works.

### Phase 2: Manual Bootstrapping
We will manually perform a full compilation cycle to prove capability.
1. **Stage 0 (Host)**: Use the existing Rust-based `quiche` compiler to transpile `crates/quiche-self/src/**/*.qrs` to `target/stage0/`.
2. **Compile Stage 0**: Compile the Rust code in `target/stage0/` to a native binary: `stage0-quiche`.
3. **Stage 1 (Self)**: Use `stage0-quiche` to transpile `crates/quiche-self/src/**/*.qrs` to `target/stage1/`.
4. **Compare**: `diff -r target/stage0 target/stage1`.
    - If they are identical, we have achieved deterministic self-reproduction.
    - If they differ, we debug the transpilation logic divergence.

### Phase 3: Automated Bootstrapping
Integrate the cycle into the build system.
- Create a `bootstrap.sh` script or `Makefile` target.
- CI should run the bootstrap cycle to prevent regressions.

### Phase 4: The Cutover
1. Move the original Rust implementation to `legacy/`.
2. Promote `quiche-self` to the canonical implementation.
3. The build process becomes:
    - Download/Use a "stable" snapshot of `quiche` binary (Stage 0).
    - Build current source (Stage 1).

## Detailed Plan

### 1. Verify Full Compiler Transpilation
Currently, we test isolated files. We need to verify that `quiche-self` can transpile *all* its own source files as a cohesive project.
- **Action**: Create a test harness that invokes `quiche-self` on its own directory.

### 2. Stdlib & Runtime Alignment
Ensure `quiche-self` generates code compatible with the same runtime `crates/runtime` that the host compiler uses.
- **Action**: Audit `crates/runtime` dependencies in generated code.

### 3. Build the Driver
`quiche-self` needs main entry point logic (CLI args parsing, file reading, writing output) that mirrors the host compiler.
- **Action**: Ensure `main.rs` (the Rust entry point for the generated code) is robust enough to act as a compiler CLI.

### 4. Comparison Tools
- **Action**: Write a script to diff generic Rust code, ignoring minor whitespace/comment differences if necessary (though identical byte output is the gold standard).

## Exception Handling Note
Regarding `catch_unwind`:
- It is a pragmatic "safety net" for the `except` block logic, essential for handling standard Rust panics (like `checked_add` overflow or assertions) that mimick Python's runtime exceptions.
- For "Business Logic" errors, we should indeed prefer `Result`. The `check!` macro bridges this by propagating `Result`s. `catch_unwind` catches what slips through (unexpected panics), ensuring valid Python semantics where "everything is catchable".
