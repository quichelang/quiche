# Pathlib Implementation Grievances

## Issue: Runtime Module Codegen Path Mismatch

**Date:** 2026-02-02
**Component:** pathlib.qrs, quiche-runtime
**Severity:** Blocking

### Description

When compiling `pathlib.qrs` as part of `quiche-runtime`, the @extern helper functions are generated with the wrong crate path.

The codegen emits:
```rust
use crate::quiche::create_Path;
```

But in `quiche-runtime`, there is no `quiche` module. The `create_Path` function is defined directly in `lib.rs`.

### Root Cause

The host compiler's codegen assumes all .qrs files are compiled in the context of `metaquiche-native`, which has a `quiche` module. However, `quiche-runtime` is a separate crate with different module structure.

### Workaround

Either:
1. Define `create_Path` with an absolute path in the @extern decorator
2. Create a `quiche` module in `quiche-runtime` that re-exports the helpers
3. Make the codegen context-aware for which crate is being compiled

### Impact

The pathlib module cannot be compiled until this is resolved.

### Files Affected

- `quiche/quiche-runtime/src/pathlib.qrs`
- `quiche/quiche-runtime/src/lib.rs`
- `metaquiche/metaquiche-host/src/codegen.rs` (or similar)
