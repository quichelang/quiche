# Handoff: Unifying Compiler Templates for Stage Parity

## Status: ✅ COMPLETED

## Summary

Successfully implemented a unified template system in `metaquiche-shared` that both the host compiler (stage0) and native compiler (stage1) now use, ensuring identical code output for stage parity.

## What Was Implemented

### 1. Created `crates/metaquiche-shared/templates.toml`
A TOML file containing all templates:
- `cargo_toml` - Generated Cargo.toml for Quiche projects
- `cargo_toml_lib_section` / `cargo_toml_bin_section` - Library/binary sections
- `build_rs` - Generated build.rs
- `lib_qrs` / `main_qrs` - Starter templates
- `quiche_toml` - Project config
- `quiche_module` - Core runtime module with traits and macros
- `quiche_module_run` - Extended module for run mode with test helpers
- `lib_rs_wrapper` / `main_rs_wrapper` - Wrappers for transpiled code
- `run_wrapper` - Wrapper for direct rustc compilation

### 2. Created `crates/metaquiche-shared/src/templates.rs`
A Rust module providing:
- `Templates::load()` - Parses and loads all templates
- `templates()` - Global template registry
- `render(template, vars)` - Simple `{{key}}` substitution
- `format_rust_code(code)` - Format using rustfmt
- `get_and_render(name, vars)` - Convenience function

### 3. Updated Host Compiler (`metaquiche-host/src/templates.rs`)
Refactored to use shared templates instead of inline strings.

### 4. Updated Native Compiler (`metaquiche-native/src/main.rs`)
Replaced inline `quiche_module` string in `run_rust_code()` with shared template usage.

## Key Changes

| File | Change |
|------|--------|
| `crates/metaquiche-shared/templates.toml` | NEW - All templates in TOML format |
| `crates/metaquiche-shared/src/templates.rs` | NEW - Template parser and renderer |
| `crates/metaquiche-shared/src/lib.rs` | Added `pub mod templates` export |
| `crates/metaquiche-host/src/templates.rs` | Rewritten to use shared templates |
| `crates/metaquiche-native/src/main.rs` | `run_rust_code()` uses shared templates |

## Unified Template Content

The key unification was in the `quiche_module` - both compilers now use the exact same:
- `QuicheResult` / `QuicheGeneric` traits
- `check!` macro with `use crate::quiche::{QuicheResult, QuicheGeneric}`
- `check` aliased as `call`
- `qref!`, `mutref!`, `deref!`, `strcat!` macros

## Verification

- ✅ `make stage1` - Compiles successfully
- ✅ `make stage2` - Compiles successfully  
- ✅ `cargo test -p metaquiche-shared templates` - All 4 tests pass
- ✅ Native compiler's `run` command works with shared templates

## Related Files (for reference)
- `crates/metaquiche-shared/templates.toml`
- `crates/metaquiche-shared/src/templates.rs`
- `crates/metaquiche-shared/src/lib.rs`
- `crates/metaquiche-host/src/templates.rs`
- `crates/metaquiche-native/src/main.rs`
