# Quiche Self-Hosting Roadmap

## Objective
Transition `quiche` from a Rust-hosted compiler to a self-hosted compiler (`quiche` written in `quiche`).

> [!NOTE]
> **Milestone Achieved**: We have implemented a custom recursive-descent parser (`quiche-parser`), removing the dependency on `ruff_python_parser`. The compiler is now fully self-contained.

## Status: Host Compiler Frozen
The Rust-based host compiler is now considered stable enough to bootstrap the self-hosted compiler with full fidelity (Stage 1 output == Stage 2 output and binary equivalence). We will freeze host compiler development and focus exclusively on the self-hosted compiler from this point forward.

## Strategy: The Bootstrapping Levels

### Level 0: Logic Self-Hosting (✅ Complete)
**Definition**: The compiler *logic* (AST -> Rust text) is written in Quiche, with a custom hand-written parser.
- **Why this is useful**: It validates that Quiche is expressive enough to handle complex recursion, scope management, and type checking.
- **Value Proposition**: "Write compiler logic like Python, run it like Rust."
    - **Developer Experience**: Write complex tree traversals without fighting the borrow checker (handled by `Rc`/cloning in the transpiler layer).
    - **Safety**: Errors are automatically propagated via `check!`, reducing boilerplate `?` handling.
    - **Speed**: The output is still native Rust.

### Level 1: Parser Self-Hosting (✅ Complete)
**Definition**: The parser itself is written in Rust with zero external parsing dependencies.
- **Achieved**: Custom recursive-descent parser in `quiche-parser` crate.
- **Dependencies**: Only `regex` for lexing, `thiserror` for errors.
- **Status**: Complete. Ruff dependency fully removed.

## The Bootstrapping Cycle (Level 0)

### Phase 1: The "Dogfood" Test (Current State)
We currently verify `quiche-self` by compiling it with the *host* compiler (Rust implementation) and running unit tests against the generated Rust code.
- **Status**: Active. `cargo test -p quiche_self` verifies the transpiled logic works.

### Phase 2: Manual Bootstrapping
We will manually perform a full compilation cycle to prove capability.
1. **Stage 1 Output (Host, implicit)**: Use the existing Rust-based `quiche` compiler to transpile `crates/quiche-self/src/**/*.qrs` to `target/stage1_out/`.
2. **Stage 1 Binary**: Compile the Rust code in `target/stage1_out/` to a native binary: `stage1-quiche`.
3. **Stage 2 Output (Self)**: Use `stage1-quiche` to transpile `crates/quiche-self/src/**/*.qrs` to `target/stage2_out/`.
4. **Compare**: `diff -r target/stage1_out target/stage2_out`.
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
    - Download/Use a "stable" snapshot of the host compiler (implicit stage).
    - Build current source (Stage 1 output + Stage 1 binary).

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
