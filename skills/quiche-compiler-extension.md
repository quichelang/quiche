---
description: How to develop AST-manipulating macros and extend the Quiche compiler
---

# Quiche Macro & Compiler Extension Development

This skill explains how to extend the Quiche compiler with new AST transformations, syntax features, and macros.

## Compiler Architecture Overview

```
┌──────────────────┐     ┌────────────────────┐     ┌──────────────────┐
│  .qrs Source     │ ──► │  quiche_parser     │ ──► │  Codegen         │
│                  │     │  (Lowered AST)     │     │  (host/native)   │
└──────────────────┘     └────────────────────┘     └──────────────────┘
        ↓                        ↓                         ↓
   ruff_python_ast         Vec<QuicheStmt>            Rust .rs output
   (raw Python AST)        (simplified)
```

### Two AST Layers

| Layer | Location | Purpose |
|-------|----------|---------|
| **ruff_python_ast** | External crate | Raw Python 3.12 syntax |
| **quiche_parser::ast** | `crates/quiche-parser/src/ast.rs` | Simplified AST for codegen |

The parser **lowers** ruff AST → quiche AST. Codegen only sees the simplified form.

## Adding New Syntax Features

### Step 1: Check if Parsing Already Works

Python syntax is parsed by ruff. Check `ruff_python_ast` docs for supported nodes.

### Step 2: Update the Lowering (parser.rs)

Location: `crates/quiche-parser/src/parser.rs`

```rust
// Example: Extracting type params with bounds
fn extract_type_params_def(params: &Option<Box<ast::TypeParams>>) -> Vec<String> {
    if let Some(p) = params {
        p.type_params.iter().map(|tp| match tp {
            ast::TypeParam::TypeVar(t) => {
                let name = t.name.to_string();
                if let Some(bound) = &t.bound {
                    format!("{}: {}", name, expr_to_string_compat(bound))
                } else { name }
            }
            _ => "?".to_string(),
        }).collect()
    } else { vec![] }
}
```

Key pattern: **Lower complex types to simple strings or primitives**

### Step 3: Update Quiche AST (ast.rs)

Location: `crates/quiche-parser/src/ast.rs`

Add new fields to structs:
```rust
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Vec<QuicheStmt>,
    pub type_params: Vec<String>,  // ← Add new fields
    // ...
}
```

### Step 4: Update Host Compiler Codegen (stmt.rs)

Location: `crates/metaquiche-host/src/stmt.rs`

Emit Rust code for the new syntax:
```rust
// Emit generic type parameters
if !f.type_params.is_empty() {
    self.output.push_str("<");
    self.output.push_str(&f.type_params.join(", "));
    self.output.push_str(">");
}
```

### Step 5: Update Native Compiler Codegen (codegen.qrs)

Location: `crates/metaquiche-native/src/compiler/codegen.qrs`

Mirror the host logic in Quiche:
```python
def emit_type_params(self, type_params: Vec[String]):
    if type_params.len() > 0:
        self.emit("<")
        first = True
        for tp in type_params:
            if not first:
                self.emit(", ")
            first = False
            self.emit(tp)
        self.emit(">")
```

### Step 6: Add Tests

Location: `crates/metaquiche-host/tests/integration_main.rs`

```rust
#[test]
fn test_compile_generic_function() {
    let source = r#"
def identity[T](x: T) -> T:
    return x
"#;
    let rust_code = compile(source).expect("Compilation failed");
    assert!(rust_code.contains("pub fn identity<T>(x: T) -> T"));
}
```

### Step 7: Build & Verify

```bash
# Build both stages
make stage1 && make stage2

# Run tests
cargo test -p metaquiche-host
```

## Native Compiler AST Access

The native compiler uses `rust.quiche_parser.ast as q_ast`:

```python
import rust.quiche_parser.ast as q_ast

def generate_stmt(self, stmt: q_ast.Stmt):
    match stmt:
        case q_ast.Stmt.FunctionDef(f):
            # f.type_params is Vec[String]
            self.emit_type_params(f.type_params)
```

**Important**: `q_ast` is `quiche_parser::ast` (lowered), NOT `ruff_python_ast`.

## Checklist for New Syntax

- [ ] Ruff parses the syntax (check `ruff_python_ast`)
- [ ] Add lowering in `parser.rs`
- [ ] Add fields to `ast.rs` if needed
- [ ] Update host codegen in `stmt.rs` or `expr.rs`
- [ ] Update native codegen in `codegen.qrs`
- [ ] Add integration test
- [ ] Verify Stage 1 + Stage 2 build
- [ ] Update documentation in `docs/language_design/`
