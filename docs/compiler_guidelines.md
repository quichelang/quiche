# Metaquiche Compiler Guidelines

This document provides guidelines for maintaining and extending the Quiche compiler infrastructure.

## Core Principle: Compiler Stability

> **Do not make non-bug-fix changes to the base compilers.**

Both `metaquiche-host` (Rust) and `metaquiche-native` (Quiche) are considered the **base compiler**. They should remain stable and minimal.

### Why This Matters

The native compiler (`metaquiche-native`) exists primarily as a **dogfooding test** that establishes the stability and capability of Quiche. Changes to the base compilers affect the entire bootstrap chain.

### Where to Add Features

| Feature Type | Where to Implement |
|--------------|-------------------|
| Sugar/syntax extensions | Runtime AST transforms |
| Utility functions | Standard library |
| Advanced features | Extended library |
| Code generation helpers | Runtime macros |

The runtime has the ability to modify the AST before it is parsed by `metaquiche-native`, making it the ideal place for experimental features.

### Bug Fixes and Modifications

Any bug fix, addition, or deletion **MUST be made to BOTH compilers**:
- `metaquiche/metaquiche-host/src/` (Rust)
- `metaquiche/metaquiche-native/src/` (Quiche)

Run `make stage1 && make stage2 && make verify` after any change.

---

## Current Stage Parity Status (2026-01-31)

### Diff Summary

| File | Lines Different |
|------|-----------------|
| main.rs | ~121 |
| codegen.rs | ~125 |
| type_utils.rs | ~21 |
| mod.rs | ~17 |

### Useful Commands

```bash
# Full rebuild and verify
make clean && make stage1 && make stage2 && make verify

# Check diff counts for each generated file
for f in main.rs compiler/type_utils.rs compiler/mod.rs compiler/codegen.rs; do
  echo "$f: $(diff target/stage1/debug/build/metaquiche-native-*/out/$f \
                   target/stage2/debug/build/metaquiche-native-*/out/$f | wc -l)"
done

# View specific file diff
diff target/stage1/debug/build/metaquiche-native-*/out/main.rs \
     target/stage2/debug/build/metaquiche-native-*/out/main.rs | head -50
```

### Fixes Applied (74% reduction in main.rs)

1. **MatchSingleton pattern** - Added `None =>` matching instead of `_ =>`
2. **Pass/Break/Continue** - Added statement handlers
3. **Skip-check list** - `is_skip_check_method()` to avoid wrapping infallible calls
4. **Struct double-space** - `pub struct Name  {` to match host
5. **Simplified elif** - No special chain detection, matches host
6. **HashMap types** - Use inferred `HashMap::new()` without explicit type params
7. **Blank lines** - Added `}\n\n` after struct/impl blocks

### Remaining Differences

| Type | Lines | Notes |
|------|-------|-------|
| Import grouping | ~30+ | Host uses `{A, B}`, native uses individual |
| Default init | ~10 | Native uses `= Default::default()` (safer) |
| Minor spacing | ~5 | Trailing newlines in some places |

---

## Future Consideration: Template Strings

To further reduce stage mismatches and improve maintainability, consider:

### Shared Template File

Create a dedicated file (e.g., `compiler_templates.toml` or `compiler_templates.yaml`) containing:
- All emit strings as key-value pairs
- Hierarchical organization by area (imports, structs, functions, etc.)
- Support for whitespace and newlines in embedded strings

Example structure:
```toml
[emit.struct]
open = "pub struct "
derives = "#[derive(Clone, Debug, Default)]\n"
close = "}\n\n"

[emit.function]
open = "pub fn "
return_arrow = " -> "
close = "}\n"

[emit.import]
use_prefix = "use "
group_open = "::{" 
group_close = "};\n"
```

Both compilers would reference these shared keys, ensuring:
1. Identical output formatting
2. Single source of truth for code generation templates
3. Easier maintenance and auditing

### Import Style Decision

Pick **one** grouping style and settle on it for both compilers:
- **Grouped**: `use module::{A, B, C};` (current host style)
- **Individual**: `use module::A; use module::B;` (current native style)

Recommendation: Adopt grouped imports as the standard since it matches Rust conventions.
