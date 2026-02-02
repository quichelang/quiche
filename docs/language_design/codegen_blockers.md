# Codegen Blockers & Known Limitations

This document tracks known code generation limitations that may block self-hosting complex logic.

## Resolved Issues

### Module-Level Constants (Resolved 2026-02-02)

**Issue**: Top-level `let` bindings were transpiled as `let mut` at module scope, which is invalid Rust.

**Solution**: Added `ConstDef` AST node. Parser detects ALL_UPPER_CASE identifiers or `Const[T]` annotations and emits `pub const` instead.

### Generic Type Syntax in Struct Fields (Resolved 2026-02-02)

**Issue**: `Vec[String]` in struct field definitions was emitted literally instead of `Vec<String>`.

**Solution**: Fixed `expr_to_type_string` in parser to use `<>` instead of `[]` for generic types.

## Open Issues

### Nested Enum Types Not In Scope

**Issue**: Enum variants like `Constant::Bool` aren't automatically imported when used in patterns.

**Status**: Needs verification.

---

## Prevention Guidelines

1. **Run both stages** after any codegen changes: `make stage1 && make stage2 && make verify`
2. **Add test cases** for new constructs before implementing
3. **Update this document** when encountering new blockers
