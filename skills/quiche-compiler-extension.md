---
description: How to develop AST-manipulating macros and extend the Quiche compiler
---

# Quiche Macro & Compiler Extension Development

This skill explains how to extend the Quiche compiler with new AST transformations, syntax features, and macros.

## Compiler Architecture Overview

```
┌──────────────────┐     ┌────────────────────┐     ┌──────────────────┐
│  .qrs Source     │ ──► │  quiche_parser     │ ──► │  Codegen         │
│                  │     │  (lexer + parser)  │     │  (host/native)   │
└──────────────────┘     └────────────────────┘     └──────────────────┘
        ↓                        ↓                         ↓
   Token stream           Vec<QuicheStmt>            Rust .rs output
   (custom lexer)         (QuicheModule)
```

### Parser Architecture

| Component | Location | Purpose |
|-------|----------|---------|
| **lexer** | `metaquiche/metaquiche-parser/src/lexer.rs` | Tokenizes source code |
| **parser** | `metaquiche/metaquiche-parser/src/parser.rs` | Recursive descent parser |
| **ast** | `metaquiche/metaquiche-parser/src/ast.rs` | Quiche AST for codegen |

The parser directly produces the Quiche AST. No external dependencies.

## Adding New Syntax Features

### Step 1: Check Parser Support

The parser is in `metaquiche/metaquiche-parser/src/parser.rs`. Check if it handles the syntax.

Location: `metaquiche/metaquiche-parser/src/parser.rs`

```rust
// Example: Parsing type params with bounds
fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
    if !self.eat(&TokenKind::LBracket)? {
        return Ok(Vec::new());
    }
    let mut params = Vec::new();
    loop {
        let name = self.expect_ident()?;
        // Handle bounds: T: Display
        if self.eat(&TokenKind::Colon)? {
            let bound = self.expect_ident()?;
            params.push(format!("{}: {}", name, bound));
        } else {
            params.push(name);
        }
        if !self.eat(&TokenKind::Comma)? { break; }
    }
    self.expect(&TokenKind::RBracket)?;
    Ok(params)
}
```

Key pattern: **Parse into simple strings or AST nodes**

### Step 3: Update Quiche AST (ast.rs)

Location: `metaquiche/metaquiche-parser/src/ast.rs`

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

Location: `metaquiche/metaquiche-host/src/stmt.rs`

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

Location: `metaquiche/metaquiche-native/src/compiler/codegen.qrs`

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

Location: `metaquiche/metaquiche-host/tests/integration_main.rs`

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

**Important**: `q_ast` is `quiche_parser::ast` - the custom Quiche AST.

## Checklist for New Syntax

- [ ] Parser handles the syntax (check `parser.rs`)
- [ ] Add parsing logic in `parser.rs` if needed
- [ ] Add fields to `ast.rs` if needed
- [ ] Update host codegen in `stmt.rs` or `expr.rs`
- [ ] Update native codegen in `codegen.qrs`
- [ ] Add integration test
- [ ] Verify Stage 1 + Stage 2 build
- [ ] Update documentation in `docs/language_design/`
- [ ] Update documentation in `docs/language_design/`
