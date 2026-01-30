# Handoff: Fixing Codegen AST Mismatches (Stage 1 Build)

**To the next agent:**

*   I significantly refactored `crates/metaquiche-native/src/compiler/codegen.qrs` to align with `quiche_parser::ast` structs. Important renames: `StmtIf` -> `IfStmt`, `StmtFor` -> `ForStmt`, `StmtReturn` -> `Option[Box[ast.Expr]]` (tuple variant handling), etc.
*   I discovered and fixed a critical bug where `q_ast.Expr.Call` patterns were transpiring to invalid Rust (using `.` instead of `::`). The root cause was `is_type_or_mod` being a stub. I implemented it fully in `codegen.qrs` by adding an `import_kinds` field to the `Codegen` class and checking against it.
*   To resolve a Rust namespace collision (E0252) between `mod.rs` importing `ast` and `codegen.qrs` importing `ast`, I renamed the import in `codegen.qrs` and `type_utils.qrs` to `import rust.quiche_parser.ast as q_ast`.
*   I fixed `main.qrs` to remove invalid `.id` field access on `alias.name` and incorrect `deref()` usage comparisons.
*   I updated `emit_if` logic in `codegen.qrs` to correctly handle the recursive `orelse` structure of `IfStmt`, replacing the unsupported `elif_else_clauses`.
*   Refactored `emit_function_def` to hoist `func_name` initialization, fixing an "unused variable" / scope error.

**Action for you:** 
Run `make clean && make stage1`. I just applied the logic fixes for path emission (`is_type_or_mod`), so the build should now succeed. If it passes, proceed to `make stage2`.

**Relevant Files:**
```json
[
  "quiche/crates/metaquiche-native/src/compiler/codegen.qrs",
  "quiche/crates/metaquiche-native/src/main.qrs",
  "quiche/crates/metaquiche-native/src/compiler/type_utils.qrs",
  "quiche/crates/quiche-parser/src/ast.rs",
  "quiche/crates/metaquiche-native/src/compiler/extern_defs.qrs",
  "quiche/crates/metaquiche-native/src/compiler/mod.qrs",
  "quiche/Makefile"
]
```
