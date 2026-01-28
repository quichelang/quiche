# Quiche Self-Hosting Handoff

## Goal: Level 0 Logic Self-Hosting

The primary objective is to make Quiche "self-hosting." This means the Quiche transpiler (written in Quiche) should be able to transpile its own source code into valid Rust, and the resulting binary should produce identical output to the original.

### The Bootstrap Process
The project uses a staged bootstrapping approach orchestrated by `bootstrap.py`:
1.  **Stage 0**: Build a Rust binary from handwritten Rust code that embeds the Quiche-in-Quiche source (`compiler.qrs`).
2.  **Stage 1 Output**: Run the Stage 0 binary on the Quiche source to generate Rust source code.
3.  **Stage 1 Binary**: Compile the Stage 1 Output into a binary.
4.  **Verification**: Run the Stage 1 Binary on the same Quiche source to produce **Stage 2 Output**. If `Stage 1 Output == Stage 2 Output`, self-hosting is achieved.

## Current Progress

- [x] **Stage 0 Build**: Successfully building `quiche_self`. This required significant workarounds for Rust's `String` handling and borrow checker limitations when dealing with transpiled code.
- [x] **Stage 1 Transpilation**: The Stage 0 binary successfully runs and generates `target/stage1_out/*.rs`.
- [/] **Stage 1 Compilation**: Currently blocked by syntax and logic errors in the *generated* code. This indicates bugs in the logic of `compiler.qrs`.

## Key Challenges & Approaches

### 1. String Concatenation and Borrowing
Transpiled code often produces sequences like `res.push_str(other.as_str())`. In Quiche, these are often built from temporary `String` objects, leading to "dropped while borrowed" or "mismatched types" (`&str` vs `String`) errors in Rust.

**Solution**: The `q_push` helper.
We introduced a functional approach in `compiler.qrs`:
```python
# compiler.qrs
res = q_push(res, "suffix")
```
This is backed by a wrapper in `main.rs` that takes ownership and returns it, bypassing the need for complex lifetime management in the transpiler's early stages.

### 2. Workspace & Build Isolation
Running `cargo build` inside the `target/` directory for bootstrap stages caused conflicts with the root `Cargo.toml`.
**Solution**: `bootstrap.py` now injects an empty `[workspace]` table into the generated `Cargo.toml` files to isolate each stage.

### 3. Meta-Escaping (The Backslash Problem)
The self-hosted compiler needs to emit code that itself contains string literals with escaped quotes. 
Example from `compiler.qrs`:
```python
self.emit(s.value.to_string().replace("\"", "\\\""))
```
**Current Issue**: This is generating invalid Rust tokens in the `compiler.rs` output, such as `replace("\"", "\\\"")` which Rust might interpret as having a trailing backslash escaping the quote incorrectly. See [compiler.rs:L844](file:///Users/jagtesh/.gemini/antigravity/playground/exo-kuiper/quiche/target/debug/build/quiche_self-3264368fc6f366e4/out/compiler.rs#L844).

## Next Steps

1.  **Fix String Escaping**: Refine how `compiler.qrs` handles quote escaping to ensure the generated Rust is valid.
2.  **Import Logic**: Ensure `compiler.qrs` correctly prefixes local modules with `crate::` and emits `use crate::` instead of `crate::use`.
3.  **Dict/Map Support**: Ensure the `Dict` type is correctly mapped in the self-hosted environment so `Dict.new()` works as intended.
4.  **Iterative Debugging**: 
    - Run `python3 bootstrap.py`.
    - Inspect failures in `target/stage1_out/`.
    - Apply fixes to `crates/quiche-self/src/compiler.qrs`.
    - Repeat until Stage 1 compiles.

## Relevant Files & Snippets

### [compiler.qrs](file:///Users/jagtesh/.gemini/antigravity/playground/exo-kuiper/quiche/crates/quiche-self/src/compiler.qrs)
The heart of the self-hosting effort. It's a port of the handwritten Rust compiler back into Quiche.
*   **Location**: `crates/quiche-self/src/compiler.qrs`
*   **Key Logic**: `Codegen` class, `emit` methods, and `q_push` usage.

### [bootstrap.py](file:///Users/jagtesh/.gemini/antigravity/playground/exo-kuiper/quiche/bootstrap.py)
The script that runs the whole show.
```python
def main():
    # ... Step 0: Host -> Stage 0 Bin
    # ... Step 1: Stage 0 Bin -> Stage 1 Out
    # ... Step 2: Compile Stage 1 Out -> Stage 1 Bin
    # ... Step 3: Stage 1 Bin -> Stage 2 Out
```

### [main.rs (Shim)](file:///Users/jagtesh/.gemini/antigravity/playground/exo-kuiper/quiche/crates/quiche-self/src/main.rs)
Provides the host environment for the self-hosted code.
```rust
pub fn push_str_wrapper(mut s: String, val: String) -> String {
    s.push_str(&val);
    s
}
```

## Possible Approaches to Remaining Issues

- **Heuristic Import Resolution**: Improving `is_type_or_mod` in `compiler.qrs` to better distinguish between local modules and external types for `::` vs `.` emission.
- **Escape Helpers**: Adding a dedicated `escape_rust_string` function to the runtime/shim to handle the complex backslash escaping needed for the compiler-to-compiler transpilation.
