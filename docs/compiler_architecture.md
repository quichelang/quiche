# Quiche Compiler Architecture

## Overview

Quiche uses a **multi-stage bootstrap** architecture where the compiler compiles itself.

```mermaid
flowchart LR
    subgraph Stage0["Stage 0: Host Compiler"]
        Host[metaquiche-host<br/>Rust]
    end
    subgraph Stage1["Stage 1: Native Compiler"]
        Native1[metaquiche-native<br/>Compiled by Host]
    end
    subgraph Stage2["Stage 2: Self-Compiled"]
        Native2[metaquiche-native<br/>Compiled by Native]
    end
    
    Host -->|compiles| Native1
    Native1 -->|compiles| Native2
    Native1 -.->|must equal| Native2
```

**Verification**: Stage 1 and Stage 2 outputs must be identical to prove the compiler is self-consistent.

---

## Compiler Stages

| Stage | Compiler | Written In | Compiles |
|-------|----------|------------|----------|
| 0 | [metaquiche-host](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-host/src/lib.rs) | Rust | Quiche → Rust |
| 1 | metaquiche-native (via Stage 0) | Quiche | Quiche → Rust |
| 2 | metaquiche-native (via Stage 1) | Quiche | Quiche → Rust |

---

## Compilation Pipeline

Both compilers follow the same pipeline:

```mermaid
flowchart TD
    Source[".qrs Source"] --> Parser
    Parser --> AST["QuicheModule AST"]
    AST --> Codegen["Code Generation"]
    Codegen --> Rust[".rs Output"]
    Rust --> Cargo["cargo build"]
    Cargo --> Binary
```

### Parser

Uses [quiche-parser](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-parser/src/lib.rs) - a custom hand-written recursive-descent parser that produces the Quiche AST.

### Code Generation

The core of both compilers. Traverses AST and emits Rust code.

---

## Host Compiler (metaquiche-host)

Written in Rust. Entry point: [lib.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-host/src/lib.rs)

### Module Structure

| Module | Purpose |
|--------|---------|
| [lib.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-host/src/lib.rs) | `Codegen` struct, symbol table, main API |
| [stmt.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-host/src/stmt.rs) | Statement emission (if, for, class, match) |
| [expr.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-host/src/expr.rs) | Expression emission (calls, binops, literals) |
| [types.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-host/src/types.rs) | Type conversion (Quiche types → Rust types) |

### Key Methods

```
Codegen::generate()        → Main entry, processes module
Codegen::generate_stmt()   → Dispatches by statement type
Codegen::generate_expr()   → Dispatches by expression type
Codegen::type_to_string()  → Converts type annotations
```

---

## Native Compiler (metaquiche-native)

Written in Quiche. Entry point: [main.qrs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-native/src/main.qrs)

### Module Structure

| Module | Purpose |
|--------|---------|
| [main.qrs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-native/src/main.qrs) | CLI, file handling, module discovery |
| [codegen.qrs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-native/src/compiler/codegen.qrs) | All code generation logic |
| [type_utils.qrs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-native/src/compiler/type_utils.qrs) | Type inference utilities |

### Key Methods

```
Codegen.generate()           → Main entry
Codegen.generate_stmt()      → Statement dispatch
Codegen.generate_expr()      → Expression dispatch
Codegen.emit_function_def()  → Function emission
Codegen.emit_class_def()     → Struct/enum emission
Codegen.type_to_string()     → Type conversion
```

---

## AST Structure

Defined in [quiche-parser/src/ast.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-parser/src/ast.rs).

```mermaid
classDiagram
    QuicheModule --> QuicheStmt
    QuicheStmt <|-- FunctionDef
    QuicheStmt <|-- ClassDef
    QuicheStmt <|-- If
    QuicheStmt <|-- Match
    QuicheStmt <|-- For
    QuicheExpr <|-- Call
    QuicheExpr <|-- Attribute
    QuicheExpr <|-- BinOp
    QuicheExpr <|-- Name
    QuicheExpr <|-- Constant
```

### Two AST Layers

The compiler uses **two distinct AST layers**:

| Layer | Crate | Purpose |
|-------|-------|---------|
| **quiche_parser::lexer** | Internal | Tokenizes source into token stream |
| **quiche_parser::ast** | Internal | Quiche AST for codegen |

```mermaid
flowchart LR
    Source[".qrs Source"] --> Lexer["Lexer\n(tokens)"]
    Lexer --> Parser["Parser\n(recursive descent)"]
    Parser --> Quiche["quiche_parser::ast\n(QuicheModule)"]
    Quiche --> Codegen["Code Generation"]
```

### Parser Implementation

The parser directly produces the Quiche AST - no lowering step needed:

```rust
// Example: Parser creates AST node directly
fn parse_function_def() -> Result<FunctionDef>
    // Returns FunctionDef with type_params: Vec<String> already extracted
```

Key parser methods in [parser.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-parser/src/parser.rs):

| Function | Purpose |
|----------|----------|
| `parse()` | Entry point, parses source to `QuicheModule` |
| `parse_function_def()` | Parses function definitions |
| `parse_class_def()` | Parses class → `ClassDef` / `StructDef` / `EnumDef` |
| `parse_expression()` | Expression parsing with precedence |

### Native Compiler AST Access

The native compiler imports the **lowered** AST:

```python
import rust.quiche_parser.ast as q_ast

def generate_stmt(self, stmt: q_ast.Stmt):
    match stmt:
        case q_ast.Stmt.FunctionDef(f):
            # f.type_params is Vec[String], already lowered
            self.emit_type_params(f.type_params)
```

> [!IMPORTANT]
> `q_ast` is `quiche_parser::ast` - the custom Quiche AST.

---

## Runtime Support

[quiche-runtime](file:///Volumes/Dev/code/jagtesh/quiche/quiche/quiche-runtime/src/lib.rs) provides macros used by generated code:

| Macro | Purpose |
|-------|---------|
| `check!` | Wraps fallible operations, converts errors |
| `qref!` | Immutable borrow (`&x`) |
| `mutref!` | Mutable borrow (`&mut x`) |
| `deref!` | Dereference (`*x`) |

---

## Build System

See [Makefile](file:///Volumes/Dev/code/jagtesh/quiche/Makefile) for targets:

```bash
make stage1   # Host compiles Native
make stage2   # Native compiles Native  
make verify   # Compare Stage 1 = Stage 2
make test     # Run test suite
```

---

## Template System

The compiler uses a zero-dependency template system to share code between Host and Native implementations, ensuring byte-identical output.

- **Location**: `metaquiche/metaquiche-shared/templates/*.toml`
    - [codegen.toml](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-shared/templates/codegen.toml): Code generation templates.
    - [project.toml](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-shared/templates/project.toml): Project scaffolding (Cargo.toml, main.rs, etc).
    - [runtime.toml](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-shared/templates/runtime.toml): Runtime wrappers and macros.
    - [messages.toml](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-shared/templates/messages.toml): i18n strings.

- **Access**: `metaquiche_shared::template::templates()` loads these at runtime (embedded).

## Bootstrapping & Runner

The native compiler (`metaquiche-native`) is written in Quiche (`src/main.qrs`), but requires a Rust entry point to be compiled by Cargo during the bootstrap process.

- **[metaquiche-native/src/main.rs](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-native/src/main.rs)**:
    - This file is a **thin bootstrap wrapper**.
    - It acts as the bridge between the generated Rust code (from `main.qrs`) and the Rust standard library / Cargo.
    - Logic for executing Rust code (`run_rust_code`) and Cargo commands (`run_cargo_command`) is delegated to **[metaquiche_shared::runner](file:///Volumes/Dev/code/jagtesh/quiche/metaquiche/metaquiche-shared/src/runner.rs)**.
    - **Do not modify logic in `main.rs` directly.** Instead, modify the shared runner or `main.qrs`.

