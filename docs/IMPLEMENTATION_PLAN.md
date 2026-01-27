# Quiche Host Compiler vs Self-Hosted Compiler Comparison

## Summary of Missing Features

The self-hosted compiler (quiche-self) is missing several critical features that the host compiler (quiche-compiler) implements. This document provides a detailed comparison with specific code examples.

---

## 1. F-String Support (f"Hello {name}")

### Host Compiler Implementation
**Location:** `quiche/crates/compiler/src/expr.rs` (lines 487-519)

```rust
ast::Expr::FString(f) => {
    // f-string: f"Hello {name}" -> format!("Hello {}", name)
    self.output.push_str("format!(\"");
    let mut args: Vec<ast::Expr> = Vec::new();

    for part in &f.value {
        match part {
            ast::FStringPart::Literal(l) => {
                self.output.push_str(&l.value);
            }
            ast::FStringPart::FString(f) => {
                for element in &f.elements {
                    match element {
                        ast::InterpolatedStringElement::Literal(l) => {
                            self.output.push_str(&l.value)
                        }
                        ast::InterpolatedStringElement::Interpolation(i) => {
                            self.output.push_str("{}");
                            args.push(*i.expression.clone());
                        }
                    }
                }
            }
        }
    }

    self.output.push_str("\"");
    for arg in args {
        self.output.push_str(", ");
        self.generate_expr(arg);
    }
    self.output.push_str(")");
}
```

**Example transformation:**
```python
name = "Quiche"
msg = f"Hello {name}"
math = f"{1+1}"
```
Generates:
```rust
let name = std::string::String::from("Quiche");
let msg = format!("Hello {}", name);
let math = format!("{}", 1 + 1);
```

### Self-Hosted Compiler
**Status:** NOT IMPLEMENTED

The self-hosted compiler's `generate_expr` function in `compiler.qrs` has no case for `ast.Expr.FString`.

**Missing AST binding:**
```python
# Not present in quiche/crates/quiche-self/src/ast.qrs
@extern(path="ruff_python_ast::ExprFString", no_generic=True)
class ExprFString:
    value: List[FStringPart]
    # ...
```

---

## 2. Dict Literals Support

### Host Compiler Implementation
**Location:** `quiche/crates/compiler/src/expr.rs` (lines 382-400)

```rust
ast::Expr::Dict(d) => {
    self.output.push_str("std::collections::HashMap::from([");
    for (i, item) in d.items.iter().enumerate() {
        if let Some(key) = &item.key {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.output.push_str("(");
            self.generate_expr(key.clone());
            self.output.push_str(", ");
            self.generate_expr(item.value.clone());
            self.output.push_str(")");
        } else {
            // **kwargs
            self.output.push_str("/* **kwargs not supported */");
        }
    }
    self.output.push_str("])");
}
```

**Example transformation:**
```python
d: Dict[String, i32] = {"a": 1, "b": 2}
```
Generates:
```rust
let d: std::collections::HashMap<String, i32> = std::collections::HashMap::from([("a", 1), ("b", 2)]);
```

### Self-Hosted Compiler
**Status:** NOT IMPLEMENTED

The self-hosted compiler's `generate_expr` function has no case for `ast.Expr.Dict`.

However, type mapping handles `Dict` type names:
```python
case ast.Expr.Name(n):
    s = n.id.as_str().to_string()
    if s == "List": return "Vec"
    if s == "Dict": return "std::collections::HashMap"  # Type mapping exists
    return s
```

---

## 3. Try-Except Support

### Host Compiler Implementation
**Location:** `quiche/crates/compiler/src/stmt.rs` (lines 134-168)

```rust
ast::Stmt::Try(t) => {
    self.push_indent();
    self.output.push_str("let _quiche_try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {\n");
    self.indent_level += 1;
    for stmt in t.body {
        self.generate_stmt(stmt);
    }
    self.indent_level -= 1;
    self.push_indent();
    self.output.push_str("}));\n");

    self.push_indent();
    self.output.push_str("if let Err(_quiche_err) = _quiche_try_result {\n");
    self.indent_level += 1;

    for handler in t.handlers {
        match handler {
            ast::ExceptHandler::ExceptHandler(inner) => {
                if let Some(name) = &inner.name {
                    self.push_indent();
                    self.output.push_str(&format!("let {} = _quiche_err.downcast_ref::<String>().map(|s| s.clone()).or_else(|| _quiche_err.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| \"Unknown Error\".to_string());\n", name));
                }

                for stmt in &inner.body {
                    self.generate_stmt(stmt.clone());
                }
            }
        }
    }

    self.indent_level -= 1;
    self.push_indent();
    self.output.push_str("}\n");
}
```

**Example transformation:**
```python
def test_manual_panic():
    caught = False
    try:
        assert(False, "Manual panic")
    except:
        caught = True
    assert(caught, "Should catch manual panic")
```
Generates:
```rust
fn test_manual_panic() {
    let mut caught = false;
    let _quiche_try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        assert!(false, "Manual panic");
    }));
    if let Err(_quiche_err) = _quiche_try_result {
        caught = true;
    }
    assert!(caught, "Should catch manual panic");
}
```

### Self-Hosted Compiler
**Status:** NOT IMPLEMENTED

No case for `ast.Stmt.Try` in `generate_stmt`.

**Missing AST binding:**
```python
# Not present in quiche/crates/quiche-self/src/ast.qrs
@extern(path="ruff_python_ast::StmtTry", no_generic=True)
class StmtTry:
    body: List[Stmt]
    handlers: List[ExceptHandler]
    # ...
```

---

## 4. Comprehensive Dict/List Method Mapping

### Host Compiler Implementation

**Dedicated modules for method mapping:**

**`quiche/crates/compiler/src/dict.rs`:**
```rust
pub fn map_dict_method(method: &str) -> Option<(&'static str, bool)> {
    match method {
        // Direct mappings (key needs &)
        "get" => Some(("get", true)),
        "remove" => Some(("remove", true)),
        "contains_key" => Some(("contains_key", true)),

        // Direct mappings (no & needed)
        "insert" => Some(("insert", false)),
        "clear" => Some(("clear", false)),
        "keys" => Some(("keys", false)),
        "values" => Some(("values", false)),
        "items" => Some(("iter", false)),
        "update" => Some(("extend", false)),
        "pop" => Some(("remove", true)),

        _ => None,
    }
}
```

**`quiche/crates/compiler/src/list.rs`:**
```rust
pub fn map_list_method(method: &str) -> Option<(&'static str, bool)> {
    match method {
        "append" => Some(("push", false)),
        "pop" => Some(("pop", false)),
        "clear" => Some(("clear", false)),
        "reverse" => Some(("reverse", false)),
        "sort" => Some(("sort", false)),
        "insert" => Some(("insert", false)),
        "extend" => Some(("extend", false)),
        _ => None,
    }
}
```

**Usage in expression generation:**
```rust
// 2. Check for Method Aliasing (List/Dict)
if let ast::Expr::Attribute(attr) => &*c.func {
    let method_name = attr.attr.as_str();

    // List
    if let Some((rust_method, _)) = crate::list::map_list_method(method_name) {
        self.output.push_str("crate::quiche::check!(");
        self.generate_expr(*attr.value.clone());
        self.output.push_str(".");
        self.output.push_str(rust_method);
        self.output.push_str("(");
        for (i, arg) in c.arguments.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            self.generate_expr(arg.clone());
        }
        self.output.push_str("))");
        return;
    }

    // Dict
    if let Some((rust_method, key_needs_ref)) =
        crate::dict::map_dict_method(method_name)
    {
        self.output.push_str("crate::quiche::check!(");
        self.generate_expr(*attr.value.clone());
        self.output.push_str(".");
        self.output.push_str(rust_method);
        self.output.push_str("(");
        for (i, arg) in c.arguments.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            if i == 0 && key_needs_ref {
                self.output.push_str("&");
            }
            self.generate_expr(arg.clone());
        }
        self.output.push_str(")");
        if method_name == "get" {
            self.output.push_str(".cloned()");
        }
        self.output.push_str(")");
        return;
    }
}
```

### Self-Hosted Compiler
**Status:** VERY LIMITED IMPLEMENTATION

Only handles 3 list methods:
```python
# Check for method call: v.append(...)
is_method = False
match as_ref(deref(c.func)):
    case ast.Expr.Attribute(at):
         attr_name_m = at.attr.as_str().to_string()
         if attr_name_m == "append": is_method = True
         if attr_name_m == "pop": is_method = True
         if attr_name_m == "push": is_method = True
    case _: pass

if is_method:
    match as_ref(deref(c.func)):
        case ast.Expr.Attribute(at2):
             self.generate_expr(ast.Expr.clone(as_ref(deref(at2.value))))
             attr_name_inner = at2.attr.as_str().to_string()
             if attr_name_inner == "append": self.emit(".push(")
             elif attr_name_inner == "pop": self.emit(".pop(")
             elif attr_name_inner == "push": self.emit(".push(")
             else: self.emit(".method(")
```

**Missing:**
- Dict methods (get, insert, remove, etc.)
- Extended list methods (clear, reverse, sort, insert, extend)
- Reference handling for dict keys

---

## 5. Attribute Access: Static vs Instance Detection

### Host Compiler Implementation
**Location:** `quiche/crates/compiler/src/expr.rs` (lines 321-348) and `quiche/crates/compiler/src/lib.rs` (lines 97-118)

```rust
ast::Expr::Attribute(a) => {
    let base_str = self.expr_to_string(&a.value);
    self.generate_expr(*a.value.clone());

    let sep = if matches!(
        &*a.value,
        ast::Expr::StringLiteral(_)
            | ast::Expr::NumberLiteral(_)
            | ast::Expr::BooleanLiteral(_)
            | ast::Expr::NoneLiteral(_)
            | ast::Expr::List(_)
            | ast::Expr::Dict(_)
            | ast::Expr::Tuple(_)
            | ast::Expr::Lambda(_)
    ) {
        "."
    } else if self.is_type_or_mod(&base_str) {
        "::"
    } else {
        "."
    };
    let attr_name = if a.attr.as_str() == "def_" {
        "def"
    } else {
        a.attr.as_str()
    };
    self.output.push_str(&format!("{}{}", sep, attr_name));
}
```

**Helper function:**
```rust
pub(crate) fn is_type_or_mod(&self, base_str: &str) -> bool {
    if base_str == "self" {
        false
    } else if base_str == "ast"
        || base_str == "compiler"
        || base_str == "types"
        || base_str == "rustpython_parser"
        || base_str == "ruff_python_parser"
        || base_str == "ruff_python_ast"
        || base_str.starts_with("std::")
        || base_str.starts_with("crate::")
        || base_str.contains("::")
    {
        true
    } else {
        base_str
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }
}
```

### Self-Hosted Compiler
**Status:** SIMPLIFIED IMPLEMENTATION

```python
case ast.Expr.Attribute(a):
    target_attr = deref(a.value)
    is_static_attr = False
    match as_ref(target_attr):
        case ast.Expr.Name(n):
            name_attr = n.id.as_str().to_string()
            first_char = name_attr.chars().next()
            match first_char:
                case Some(c):
                    if c.is_uppercase():
                        is_static_attr = True
                case None: pass
        case _: pass
    if a.attr.as_str().to_string() == "new": is_static_attr = True

    self.generate_expr(ast.Expr.clone(as_ref(target_attr)))
    if is_static_attr: self.emit("::")
    else: self.emit(".")
    self.emit(a.attr.as_str().to_string())
```

**Differences:**
- Self-hosted doesn't check for known modules (ast, compiler, types, std, crate)
- Self-hosted doesn't handle `def_` -> `def` renaming
- Self-hosted doesn't check for `::` in base string

---

## 6. Special Built-in Functions

### Host Compiler
Handles several special cases without `check!` wrapper:
- `as_ref(x)` -> `&x`
- `deref(x)` -> `*x`
- `as_mut(x)` -> `&mut x`
- `parse_program(...)` -> `parse_program(...)`

```rust
// Handle as_ref and deref without check! wrapper to preserve ref/deref semantics
if func_name == "as_ref" {
    if let Some(arg) = c.arguments.args.first() {
        self.output.push_str("&");
        self.generate_expr(arg.clone());
    }
    return;
}

if func_name == "deref" {
    if let Some(arg) = c.arguments.args.first() {
        self.output.push_str("*");
        self.generate_expr(arg.clone());
    }
    return;
}

if func_name == "parse_program" {
    self.output.push_str("parse_program(");
    for (i, arg) in c.arguments.args.iter().enumerate() {
        if i > 0 {
            self.output.push_str(", ");
        }
        self.generate_expr(arg.clone());
    }
    self.output.push_str(")");
    return;
}
```

### Self-Hosted Compiler
**Status:** NOT IMPLEMENTED

These functions go through the generic `quiche_runtime::call!` path.

---

## 7. Type Mapping and Inference

### Host Compiler
**Location:** `quiche/crates/compiler/src/types.rs`

Comprehensive type mapping with turbo-fish syntax support:
```rust
fn map_type_internal(&self, expr: &ast::Expr, is_expr: bool) -> String {
    let sep = if is_expr { "::" } else { "" };
    match expr {
        ast::Expr::Name(n) => match n.id.as_str() {
            "Dict" | "HashMap" => "std::collections::HashMap".to_string(),
            "List" | "Vec" => "Vec".to_string(),
            "Option" => "Option".to_string(),
            "Result" => "Result".to_string(),
            "String" | "str" => "String".to_string(),
            "StrRef" => "&str".to_string(),
            _ => n.id.to_string(),
        },
        ast::Expr::Subscript(s) => {
            let base = self.map_type_internal(&s.value, false);
            let inner = self.map_type_internal(&s.slice, false);
            format!("{}{}<{}>", rust_base, sep, final_inner)
        }
        // ...
    }
}

pub(crate) fn get_expr_type(&self, expr: &ast::Expr) -> Option<String> {
    match expr {
        ast::Expr::Name(n) => self.get_symbol(&n.id).cloned(),
        ast::Expr::Subscript(s) => {
            let base_type = self.get_expr_type(&s.value)?;
            // Extract tuple element type by index
            if base_type.starts_with("(") {
                // Complex logic to parse tuple string and extract Nth element
            }
        }
        _ => None,
    }
}
```

### Self-Hosted Compiler
**Status:** SIMPLIFIED

```python
def type_to_string(self, expr: ast.Expr) -> String:
    match expr:
        case ast.Expr.Name(n):
            s = n.id.as_str().to_string()
            if s == "List": return "Vec"
            if s == "Dict": return "std::collections::HashMap"
            return s
        case ast.Expr.Subscript(s):
            base = self.type_to_string(deref(s.value))
            inner = self.type_to_string(deref(s.slice))
            res = RustString.new()
            res.push_str(as_ref(base))
            res.push_str(as_ref("<"))
            res.push_str(as_ref(inner))
            res.push_str(as_ref(">"))
            return res
        case _: return "Any"

def infer_expr_type(self, expr: ast.Expr) -> String:
    match expr:
        case ast.Expr.NumberLiteral(_): return "i32"
        case ast.Expr.StringLiteral(_): return "String"
        case ast.Expr.BooleanLiteral(_): return "bool"
        case ast.Expr.Name(n):
            match self.symbols.lookup(n.id.as_str().to_string()):
                case Some(s): return s.type_name
                case None: return "unknown"
        case _: return "unknown"
```

**Differences:**
- No tuple element type extraction
- No turbo-fish syntax (always uses `<` not `::<`)
- Limited type name mappings

---

## 8. Decorator Processing

### Host Compiler
Handles `@extern` and `@enum` decorators for classes:

```rust
// Check for @extern(path="...", no_generic=true)
let mut extern_path = None;
let mut no_generic = false;

for decorator in &c.decorator_list {
    if let ast::Expr::Call(call) = &decorator.expression {
        if let ast::Expr::Name(n) = &*call.func {
            if n.id.as_str() == "extern" {
                for keyword in &call.arguments.keywords {
                    if let Some(arg) = &keyword.arg {
                        if arg == "path" {
                            if let ast::Expr::StringLiteral(s) = &keyword.value {
                                extern_path = Some(s.value.to_string());
                            }
                        } else if arg == "no_generic" {
                            match &keyword.value {
                                ast::Expr::BooleanLiteral(b) => {
                                    no_generic = b.value;
                                }
                                ast::Expr::Name(n) => {
                                    if n.id.as_str() == "true" {
                                        no_generic = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}

if let Some(path) = extern_path {
    if no_generic {
        self.output
            .push_str(&format!("pub type {} = {};\n", c.name, path));
    } else {
        self.output
            .push_str(&format!("pub type {}<T> = {}<T>;\n", c.name, path));
    }
    return;
}

// Check for @enum decorator
let is_enum = c.decorator_list.iter().any(|d| {
    if let ast::Expr::Name(n) = &d.expression {
        n.id.as_str() == "enum"
    } else {
        false
    }
});
```

### Self-Hosted Compiler
Handles basic decorator checking but doesn't implement all keyword parsing details.

---

## Summary Table

| Feature | Host Compiler | Self-Hosted Compiler | Status |
|---------|--------------|---------------------|--------|
| F-Strings | ✓ | ✗ | Missing AST & generation |
| Dict Literals | ✓ | ✗ | Missing AST & generation |
| Try-Except | ✓ | ✗ | Missing AST & generation |
| List Methods (all) | ✓ (9 methods) | Partial (3 methods) | Incomplete |
| Dict Methods | ✓ (9 methods) | ✗ | Missing |
| Static Attribute Detection | ✓ (comprehensive) | Partial (simplified) | Incomplete |
| def_ -> def renaming | ✓ | ✗ | Missing |
| Special Built-ins (as_ref, etc.) | ✓ | ✗ | Missing |
| Tuple Type Extraction | ✓ | ✗ | Missing |
| Turbo-fish Syntax | ✓ | ✗ | Missing |
| Symbol Table Lookup | ✓ | Partial | Incomplete |

---

## Implementation Priority

### Critical (Test Failures)
1. **F-Strings** - Used in `test_types_suite.qrs`
2. **Dict Literals** - Used in `test_types_suite.qrs`
3. **Dict Methods** - `get`, `insert`, `remove` used in tests

### High Priority
4. **Try-Except** - Used in `test_exceptions.qrs`
5. **Complete List Methods** - Missing `clear`, `reverse`, `sort`, etc.

### Medium Priority
6. **Static Attribute Detection** - Better handling of `::` vs `.`
7. **Special Built-ins** - `as_ref`, `deref` for better interop

### Low Priority
8. **Tuple Type Extraction** - Advanced feature
9. **Turbo-fish Syntax** - Can work with regular `<>` for now
