# Codegen Blockers & Known Limitations

This document tracks known code generation limitations that may block self-hosting complex logic.

## Resolved Issues

### Module-Level Constants (Resolved 2026-02-02)

**Issue**: Top-level `let` bindings were transpiled as `let mut` at module scope, which is invalid Rust.

**Solution**: Added `ConstDef` AST node. Parser detects ALL_UPPER_CASE identifiers or `Const[T]` annotations and emits `pub const` instead.

### Generic Type Syntax (No Fix Needed)

**Issue**: Initially reported that generic types used `[T]` instead of `<T>` in struct fields.

**Status**: Investigation confirmed `type_to_string` correctly uses angle brackets. This was a false report.

## Open Issues

### Crate Imports

**Issue**: `from rust import X` generates `use rust::X;` instead of `use crate::X;`.

**Status**: Pending implementation. Workaround: Use explicit crate path imports.

### Enum Variant Access

**Issue**: Potential issue where `EnumType.Variant` doesn't correctly translate to `EnumType::Variant`.

**Status**: Needs verification.

---

## Prevention Guidelines

1. **Run both stages** after any codegen changes: `make stage1 && make stage2 && make verify`
2. **Add test cases** for new constructs before implementing
3. **Update this document** when encountering new blockers
