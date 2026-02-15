//! Quiche — Python-flavoured Rust
//!
//! Parses `.q` source files and produces `elevate::ast::Module`,
//! which feeds directly into the Elevate compiler pipeline.

pub mod lexer;
pub mod parser;

// Re-export Elevate options so the CLI can use them without depending on elevate directly
pub use elevate::{CompileOptions, CompilerOutput, ExperimentFlags};

/// Default options with core experiment flags enabled.
pub fn default_options() -> CompileOptions {
    CompileOptions {
        experiments: ExperimentFlags {
            move_mut_args: false,
            type_system: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Parse Quiche source into an Elevate AST Module.
pub fn parse(source: &str) -> Result<elevate::ast::Module, String> {
    parser::parse(source).map_err(|e| format!("{e}"))
}

/// Parse Quiche source, compile through Elevate, and emit Elevate source code.
/// This produces valid `.ers` syntax from the typed IR — useful for bug reports.
pub fn emit_elevate(source: &str, options: &CompileOptions) -> Result<String, String> {
    let module = parser::parse(source).map_err(|e| format!("{e}"))?;
    let output = elevate::compile_ast_with_options(&module, options).map_err(|e| format!("{e}"))?;
    Ok(elevate::emit_elevate::emit_typed_module(&output.typed))
}

/// Parse Quiche source and compile to Rust via Elevate.
pub fn compile(source: &str) -> Result<String, String> {
    compile_with_options(source, &CompileOptions::default())
}

/// Parse Quiche source and compile to Rust via Elevate with custom options.
pub fn compile_with_options(source: &str, options: &CompileOptions) -> Result<String, String> {
    let module = parser::parse(source).map_err(|e| format!("{e}"))?;
    let output = elevate::compile_ast_with_options(&module, options).map_err(|e| format!("{e}"))?;
    Ok(inject_auto_imports(&wrap_collections(
        &inject_display_impls(&output.rust_code),
    )))
}

/// Compile a .q file with source-mapped diagnostics.
/// Passes the filename and source text to Elevate so errors show file:line:col.
pub fn compile_file(
    source: &str,
    filename: &str,
    options: &CompileOptions,
) -> Result<String, String> {
    let module = parser::parse(source).map_err(|e| format!("{e}"))?;
    let mut opts = options.clone();
    opts.source_name = Some(filename.to_string());
    let output = elevate::compile_ast_with_options(&module, &opts).map_err(|e| {
        // CompileError Display already uses source_map::render_diagnostic,
        // but we need to also supply source_text for line:col resolution
        let mut err = e;
        if err.source_text.is_none() {
            err.source_text = Some(source.to_string());
        }
        format!("{err}")
    })?;
    Ok(inject_auto_imports(&wrap_collections(
        &inject_display_impls(&output.rust_code),
    )))
}

/// Post-process generated Rust: auto-generate `impl Display` for structs
/// that define a `to_string` method. This lets `print(x)` use the custom
/// format without the user writing any trait boilerplate.
fn inject_display_impls(rust_code: &str) -> String {
    let re_struct = regex::Regex::new(r"pub struct (\w+)").unwrap();
    let re_impl_to_string = regex::Regex::new(r"impl (\w+)\s*\{[^}]*fn to_string\s*\(").unwrap();

    // Collect struct names that have a to_string method
    let struct_names: std::collections::HashSet<String> = re_struct
        .captures_iter(rust_code)
        .map(|c| c[1].to_string())
        .collect();

    let impls_with_to_string: std::collections::HashSet<String> = re_impl_to_string
        .captures_iter(rust_code)
        .map(|c| c[1].to_string())
        .collect();

    let needs_display: Vec<&String> = struct_names
        .iter()
        .filter(|name| impls_with_to_string.contains(*name))
        .collect();

    if needs_display.is_empty() {
        return rust_code.to_string();
    }

    let mut result = rust_code.to_string();
    for name in &needs_display {
        result.push_str(&format!(
            "\nimpl std::fmt::Display for {name} {{\n    \
             fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        \
             write!(f, \"{{}}\", self.clone().to_string())\n    \
             }}\n}}\n"
        ));
    }
    result
}

/// Auto-inject `use std::collections::*` imports when the generated Rust
/// references collection types that aren't in the prelude.
fn inject_auto_imports(rust_code: &str) -> String {
    // HashMap::from(vec![…]) → HashMap::from([…]) already handled by wrap_collections

    let mut imports = Vec::new();

    if rust_code.contains("HashMap") {
        imports.push("HashMap");
    }
    if rust_code.contains("HashSet") {
        imports.push("HashSet");
    }
    if rust_code.contains("BTreeMap") {
        imports.push("BTreeMap");
    }
    if rust_code.contains("BTreeSet") {
        imports.push("BTreeSet");
    }

    if imports.is_empty() {
        return rust_code.to_string();
    }

    let import_line = format!("use std::collections::{{{}}};\n", imports.join(", "));

    // Insert after the last #![allow(...)] line
    if let Some(pos) = rust_code.rfind("#![allow(") {
        if let Some(end) = rust_code[pos..].find('\n') {
            let insert_at = pos + end + 1;
            let mut result = String::with_capacity(rust_code.len() + import_line.len());
            result.push_str(&rust_code[..insert_at]);
            result.push_str(&import_line);
            result.push_str(&rust_code[insert_at..]);
            return result;
        }
    }

    // Fallback: prepend
    format!("{import_line}{rust_code}")
}

/// Fix `HashMap::from(vec![...])` → `HashMap::from([...])`
/// AND wrap collection constructors in List/Dict newtypes.
///
/// Elevate sees `Vec` and `HashMap` during type-checking, but Quiche programs
/// should work with `List` and `Dict`.  We rewrite at the Rust string level so
/// Elevate can still resolve `.push()`, `.len()`, etc. against its Vec/HashMap
/// knowledge, and the final Rust code uses quiche-lib newtypes.
fn wrap_collections(rust_code: &str) -> String {
    // Step 1: fix HashMap::from(vec![…]) → HashMap::from([…])
    let code = rust_code.replace("HashMap::from(vec![", "HashMap::from([");

    // Step 2: wrap vec![…] → List::from(vec![…])
    //         and  Vec::new() → List::new()
    let code = wrap_vec_in_list(&code);

    // Step 3: wrap HashMap::from(…) → Dict(HashMap::from(…))
    //         and  HashMap::new()   → Dict::new()
    let code = wrap_hashmap_in_dict(&code);

    // Step 4: rewrite type annotations  Vec<T> → List<T>
    let code = rewrite_vec_type_annotations(&code);

    // Step 5: rewrite type annotations  HashMap<K,V> → Dict<K,V>
    rewrite_hashmap_type_annotations(&code)
}

/// Find each `vec![…]` (bracket-aware) and wrap in `List::from(vec![…])`.
/// Also replaces `Vec::new()` → `List::new()`.
fn wrap_vec_in_list(code: &str) -> String {
    // First, simple replacements
    let code = code.replace("Vec::new()", "List::new()");

    let mut result = String::with_capacity(code.len() + 128);
    let bytes = code.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Look for `vec![`
        if i + 4 < bytes.len() && &code[i..i + 4] == "vec!" && bytes.get(i + 4) == Some(&b'[') {
            // Find the matching `]`
            let open = i + 4; // position of `[`
            if let Some(close) = find_matching_bracket(&code, open) {
                result.push_str("List::from(");
                result.push_str(&code[i..=close]); // vec![…]
                result.push(')');
                i = close + 1;
            } else {
                // No matching bracket — emit as-is
                result.push(bytes[i] as char);
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// Wrap `HashMap::from(…)` in `Dict(…)` and replace `HashMap::new()` with `Dict::new()`.
fn wrap_hashmap_in_dict(code: &str) -> String {
    // HashMap::new() → Dict::new()
    let code = code.replace("HashMap::new()", "Dict::new()");

    // HashMap::from(…) / HashMap::from_iter(…) → Dict(HashMap::…(…))
    let mut result = String::with_capacity(code.len() + 128);
    let bytes = code.as_bytes();
    let mut i = 0;
    let needles = ["HashMap::from_iter(", "HashMap::from("];

    while i < bytes.len() {
        let mut matched = false;
        for needle in &needles {
            if i + needle.len() <= bytes.len() && &code[i..i + needle.len()] == *needle {
                let open = i + needle.len() - 1; // position of `(`
                if let Some(close) = find_matching_paren(&code, open) {
                    result.push_str("Dict(");
                    result.push_str(&code[i..=close]); // HashMap::from(…)
                    result.push(')');
                    i = close + 1;
                    matched = true;
                }
                break;
            }
        }
        if !matched {
            result.push(bytes[i] as char);
            i += 1;
        }
    }
    result
}

/// Rewrite `Vec<…>` → `List<…>` in type positions (bindings and return types).
fn rewrite_vec_type_annotations(code: &str) -> String {
    code.replace(": Vec<", ": List<")
        .replace("-> Vec<", "-> List<")
}

/// Rewrite `HashMap<…>` → `Dict<…>` in type positions (bindings and return types).
fn rewrite_hashmap_type_annotations(code: &str) -> String {
    code.replace(": HashMap<", ": Dict<")
        .replace("-> HashMap<", "-> Dict<")
}

/// Find the matching `]` for a `[` at position `open`, respecting nesting.
fn find_matching_bracket(code: &str, open: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    if bytes.get(open) != Some(&b'[') {
        return None;
    }
    let mut depth = 1;
    let mut i = open + 1;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' => {
                // Skip string literals
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Find the matching `)` for a `(` at position `open`, respecting nesting.
fn find_matching_paren(code: &str, open: usize) -> Option<usize> {
    let bytes = code.as_bytes();
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth = 1;
    let mut i = open + 1;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' => {
                // Skip string literals
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1; // skip escaped char
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::parser::parse;
    use elevate::ast::*;

    /// Number of Item::RustBlock items injected by the Quiche primitive type prelude.
    const PRELUDE_COUNT: usize = 2;

    /// Extract only user-defined items (skip the smart string prelude).
    fn user_items(module: &Module) -> &[Item] {
        &module.items[PRELUDE_COUNT..]
    }

    // ─── Functions ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_simple_expression() {
        let module = parse("x + 1").unwrap();
        assert_eq!(user_items(&module).len(), 0); // top-level expr is skipped as item
    }

    #[test]
    fn test_parse_function_def() {
        let source = "def foo(x: int) -> int:\n    return x + 1\n";
        let module = parse(source).unwrap();
        assert_eq!(user_items(&module).len(), 1);
        match &user_items(&module)[0] {
            Item::Function(f) => {
                assert_eq!(f.name, "foo");
                assert_eq!(f.params.len(), 1);
                assert_eq!(f.params[0].name, "x");
                assert!(f.return_type.is_some());
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── Structs ─────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_struct() {
        let source = "type Point:\n    x: int\n    y: int\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert_eq!(s.fields[1].name, "y");
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn test_struct_with_type_params() {
        let source = "type Point[T]:\n    x: T\n    y: int\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert_eq!(s.fields[0].ty.path, vec!["T"]);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    // ─── Control Flow ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_if() {
        let source = "def test():\n    if x:\n        y = 1\n    else:\n        y = 2\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => match &f.body.statements[0] {
                Stmt::If {
                    then_block,
                    else_block,
                    ..
                } => {
                    assert_eq!(then_block.statements.len(), 1);
                    assert!(else_block.is_some());
                }
                other => panic!("Expected If, got {:?}", other),
            },
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_if_elif_else() {
        let source = "def test():\n    if a > 0:\n        x = 1\n    elif b > 0:\n        x = 2\n    elif c > 0:\n        x = 3\n    else:\n        x = 4\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => {
                match &f.body.statements[0] {
                    Stmt::If {
                        then_block,
                        else_block,
                        ..
                    } => {
                        assert_eq!(then_block.statements.len(), 1);
                        // elif chain: else contains another If
                        let else_b = else_block.as_ref().unwrap();
                        assert_eq!(else_b.statements.len(), 1);
                        match &else_b.statements[0] {
                            Stmt::If {
                                else_block: elif2, ..
                            } => {
                                let elif2_b = elif2.as_ref().unwrap();
                                match &elif2_b.statements[0] {
                                    Stmt::If {
                                        else_block: final_else,
                                        ..
                                    } => {
                                        assert!(final_else.is_some());
                                    }
                                    other => panic!("Expected nested If (elif2), got {:?}", other),
                                }
                            }
                            other => panic!("Expected nested If (elif1), got {:?}", other),
                        }
                    }
                    other => panic!("Expected If, got {:?}", other),
                }
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_for() {
        let source = "def test():\n    for i in range(10):\n        print(i)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => match &f.body.statements[0] {
                Stmt::For { binding, .. } => match binding {
                    DestructurePattern::Name(n) => assert_eq!(n, "i"),
                    other => panic!("Expected Name binding, got {:?}", other),
                },
                other => panic!("Expected For, got {:?}", other),
            },
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── Imports ─────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_from_import() {
        let module = parse("from os import path").unwrap();
        match &user_items(&module)[0] {
            Item::RustUse(u) => {
                // UseTree::Path { segment: "os", next: UseTree::Name("path") }
                match &u.tree {
                    UseTree::Path { segment, next } => {
                        assert_eq!(segment, "os");
                        assert!(matches!(next.as_ref(), UseTree::Name(n) if n == "path"));
                    }
                    other => panic!("Expected UseTree::Path, got {:?}", other),
                }
            }
            other => panic!("Expected RustUse, got {:?}", other),
        }
    }

    // ─── Match ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_match() {
        let source = "def test():\n    match result:\n        case Ok(v):\n            return v\n        case Err(e):\n            print(e)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => match &f.body.statements[0] {
                Stmt::Expr(Expr::Match { arms, .. }) => {
                    assert_eq!(arms.len(), 2);
                }
                other => panic!("Expected Match expr, got {:?}", other),
            },
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── Multiline Calls ─────────────────────────────────────────────────────

    #[test]
    fn test_parse_multiline_function_call() {
        let source = "def test():\n    result = foo(\n        1,\n        2,\n        3\n    )\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => {
                // Should parse without error — multiline call inside brackets
                assert!(!f.body.statements.is_empty());
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── Docstrings ──────────────────────────────────────────────────────────

    #[test]
    fn test_parse_multiline_docstring_then_if() {
        let source = "def test(self):\n    \"\"\"Multi\n    line\n    doc\"\"\"\n    if x > 0:\n        pass\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => {
                // Should have docstring + if in body
                assert!(
                    f.body.statements.len() >= 2,
                    "Function body should have at least 2 statements, got {}",
                    f.body.statements.len()
                );
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── Operators ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_boolean_operators() {
        let source = "def test():\n    return a and b or not c\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => match &f.body.statements[0] {
                Stmt::Return(Some(Expr::Binary {
                    op: BinaryOp::Or, ..
                })) => {}
                other => panic!("Expected Return(Binary::Or), got {:?}", other),
            },
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── F-Strings ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_fstring() {
        let source = "def test():\n    name = \"World\"\n    greeting = f\"Hello, {name}!\"\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Function(f) => {
                // Should parse without errors — f-string becomes MacroCall
                assert!(f.body.statements.len() >= 2);
            }
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── End-to-End Compile ──────────────────────────────────────────────────

    #[test]
    fn test_compile_simple_function() {
        let source = "def add(a: i64, b: i64) -> i64:\n    return a + b\n";
        let rust = crate::compile(source).unwrap();
        assert!(rust.contains("fn add"));
        assert!(rust.contains("i64"));
    }

    #[test]
    fn test_compile_struct() {
        let source = "type Point:\n    x: i32\n    y: i32\n";
        let rust = crate::compile(source).unwrap();
        assert!(rust.contains("struct Point"));
        assert!(rust.contains("x: i32"));
    }

    // ─── Struct Instantiation ────────────────────────────────────────────────

    fn compile_ok(source: &str) -> String {
        crate::compile_with_options(source, &crate::default_options())
            .expect(&format!("Failed to compile:\n{source}"))
    }

    // --- Positional construction ---

    #[test]
    fn test_struct_positional_two_fields() {
        let rust = compile_ok(
            "type Point:\n    x: i32\n    y: i32\n\ndef main():\n    p: Point = Point(1, 2)\n",
        );
        assert!(
            rust.contains("Point {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_positional_single_field() {
        let rust = compile_ok(
            "type Wrapper:\n    val: i32\n\ndef main():\n    w: Wrapper = Wrapper(42)\n",
        );
        assert!(
            rust.contains("Wrapper {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_positional_three_fields() {
        let rust = compile_ok(
            "type Vec3:\n    x: i32\n    y: i32\n    z: i32\n\ndef main():\n    v: Vec3 = Vec3(1, 2, 3)\n",
        );
        assert!(
            rust.contains("Vec3 {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_positional_mixed_types() {
        let rust = compile_ok(
            "type Person:\n    name: String\n    age: i32\n\ndef main():\n    p: Person = Person(\"Alice\", 30)\n",
        );
        assert!(
            rust.contains("Person {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    // --- Keyword construction ---

    #[test]
    fn test_struct_keyword_two_fields() {
        let rust = compile_ok(
            "type Point:\n    x: i32\n    y: i32\n\ndef main():\n    p: Point = Point(x=5, y=10)\n",
        );
        assert!(
            rust.contains("Point {"),
            "Expected struct literal from kwargs, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_reversed_order() {
        // kwargs in reverse order should still assign correctly
        let rust = compile_ok(
            "type Point:\n    x: i32\n    y: i32\n\ndef main():\n    p: Point = Point(y=10, x=5)\n",
        );
        assert!(
            rust.contains("Point {"),
            "Expected struct literal from reversed kwargs, got:\n{rust}"
        );
        // Verify x comes before y in the struct literal (field order matches struct definition)
        let x_pos = rust.find("x:").unwrap();
        let y_pos = rust.find("y:").unwrap();
        assert!(
            x_pos < y_pos,
            "Expected x before y in struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_three_fields() {
        let rust = compile_ok(
            "type Color:\n    r: i32\n    g: i32\n    b: i32\n\ndef main():\n    c: Color = Color(r=255, g=128, b=0)\n",
        );
        assert!(
            rust.contains("Color {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_three_fields_shuffled() {
        // All kwargs in arbitrary order
        let rust = compile_ok(
            "type Color:\n    r: i32\n    g: i32\n    b: i32\n\ndef main():\n    c: Color = Color(b=0, r=255, g=128)\n",
        );
        assert!(
            rust.contains("Color {"),
            "Expected struct literal from shuffled kwargs, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_single_field() {
        let rust = compile_ok(
            "type Wrapper:\n    val: i32\n\ndef main():\n    w: Wrapper = Wrapper(val=42)\n",
        );
        assert!(
            rust.contains("Wrapper {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    // --- Top-level function kwargs ---

    #[test]
    fn test_toplevel_function_kwargs() {
        let source = "\
def add(a: i32, b: i32) -> i32:\n    return a + b\n\n\
def main():\n    x: i32 = add(b=10, a=5)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("add("),
            "Expected function call, got:\n{rust}"
        );
    }

    #[test]
    fn test_toplevel_function_kwargs_three_params() {
        let source = "\
def sum3(a: i32, b: i32, c: i32) -> i32:\n    return a + b + c\n\n\
def main():\n    x: i32 = sum3(c=30, a=10, b=20)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("sum3("),
            "Expected function call, got:\n{rust}"
        );
    }

    // --- Multiple structs ---

    #[test]
    fn test_multiple_structs_kwargs() {
        // Two different structs — kwargs should not cross-pollinate
        let source = "\
type Point:\n    x: i32\n    y: i32\n\n\
type Size:\n    width: i32\n    height: i32\n\n\
def main():\n    p: Point = Point(y=2, x=1)\n    s: Size = Size(height=200, width=100)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("Point {"),
            "Expected Point struct literal, got:\n{rust}"
        );
        assert!(
            rust.contains("Size {"),
            "Expected Size struct literal, got:\n{rust}"
        );
    }

    // ─── type keyword ────────────────────────────────────────────────────────

    #[test]
    fn test_type_struct() {
        let source = "type Point:\n    x: i32\n    y: i32\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert_eq!(s.fields[1].name, "y");
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    #[test]
    fn test_type_enum_with_payloads() {
        let source = "type Color = | Red | Green(i32) | Blue(i32, i32)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Color");
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].name, "Red");
                assert!(matches!(e.variants[0].fields, EnumVariantFields::Unit));
                assert_eq!(e.variants[1].name, "Green");
                assert!(
                    matches!(&e.variants[1].fields, EnumVariantFields::Tuple(t) if t.len() == 1)
                );
                assert_eq!(e.variants[2].name, "Blue");
                assert!(
                    matches!(&e.variants[2].fields, EnumVariantFields::Tuple(t) if t.len() == 2)
                );
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_enum_bare_variants() {
        let source = "type Direction = | North | South | East | West\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Direction");
                assert_eq!(e.variants.len(), 4);
                assert_eq!(e.variants[0].name, "North");
                assert!(matches!(e.variants[0].fields, EnumVariantFields::Unit));
                assert_eq!(e.variants[3].name, "West");
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_enum_multiline() {
        let source = "type Number =\n    | I64(i64)\n    | F64(f64)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Number");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name, "I64");
                assert!(
                    matches!(&e.variants[0].fields, EnumVariantFields::Tuple(t) if t[0].path == vec!["i64"])
                );
                assert_eq!(e.variants[1].name, "F64");
                assert!(
                    matches!(&e.variants[1].fields, EnumVariantFields::Tuple(t) if t[0].path == vec!["f64"])
                );
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_generic_enum() {
        let source = "type MyResult[T, E] = | Ok(T) | Err(E)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "MyResult");
                assert_eq!(e.type_params.len(), 2);
                assert_eq!(e.type_params[0].name, "T");
                assert_eq!(e.type_params[1].name, "E");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name, "Ok");
                assert_eq!(e.variants[1].name, "Err");
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_compile_struct() {
        let source = "type Point:\n    x: i32\n    y: i32\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("pub struct Point"),
            "Expected struct in output, got:\n{rust}"
        );
    }

    #[test]
    fn test_type_compile_enum() {
        let source = "type Number = | I64(i64) | F64(f64)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("pub enum Number"),
            "Expected enum in output, got:\n{rust}"
        );
        assert!(
            rust.contains("I64(i64)"),
            "Expected I64 variant, got:\n{rust}"
        );
        assert!(
            rust.contains("F64(f64)"),
            "Expected F64 variant, got:\n{rust}"
        );
    }

    #[test]
    fn test_type_enum_arity_disambiguation() {
        let source = "type Shape = | Point | Point(f64) | Point(f64, f64)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Shape");
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].name, "Point__a0");
                assert!(matches!(e.variants[0].fields, EnumVariantFields::Unit));
                assert_eq!(e.variants[1].name, "Point__a1");
                assert!(
                    matches!(&e.variants[1].fields, EnumVariantFields::Tuple(t) if t.len() == 1)
                );
                assert_eq!(e.variants[2].name, "Point__a2");
                assert!(
                    matches!(&e.variants[2].fields, EnumVariantFields::Tuple(t) if t.len() == 2)
                );
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_enum_mixed_unique_and_duplicate_names() {
        let source = "type Geo = | Circle(f64) | Circle(f64, f64) | Square(f64)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].name, "Circle__a1"); // disambiguated
                assert_eq!(e.variants[1].name, "Circle__a2"); // disambiguated
                assert_eq!(e.variants[2].name, "Square"); // unique, untouched
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_enum_named_fields() {
        let source = "type Shape = | Point | Rect(width: f64, height: f64)\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Shape");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name, "Point");
                assert!(matches!(e.variants[0].fields, EnumVariantFields::Unit));
                assert_eq!(e.variants[1].name, "Rect");
                match &e.variants[1].fields {
                    EnumVariantFields::Named(fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].name, "width");
                        assert_eq!(fields[1].name, "height");
                    }
                    other => panic!("Expected Named fields, got {:?}", other),
                }
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_inline_union() {
        let source = "type Number = i64 | f64\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Number");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name, "I64");
                assert!(
                    matches!(&e.variants[0].fields, EnumVariantFields::Tuple(t) if t[0].path == vec!["i64"])
                );
                assert_eq!(e.variants[1].name, "F64");
                assert!(
                    matches!(&e.variants[1].fields, EnumVariantFields::Tuple(t) if t[0].path == vec!["f64"])
                );
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_inline_union_three_types() {
        let source = "type Value = i64 | f64 | String\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Value");
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].name, "I64");
                assert_eq!(e.variants[1].name, "F64");
                assert_eq!(e.variants[2].name, "String");
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_enum_inline_no_leading_pipe() {
        let source = "type Dir = North | South | East | West\n";
        let module = parse(source).unwrap();
        match &user_items(&module)[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Dir");
                assert_eq!(e.variants.len(), 4);
                assert_eq!(e.variants[0].name, "North");
                assert_eq!(e.variants[3].name, "West");
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    #[test]
    fn test_type_compile_union() {
        let source = "type Number = i64 | f64\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("pub enum Number"),
            "Expected enum in output, got:\n{rust}"
        );
        assert!(
            rust.contains("I64(i64)"),
            "Expected I64 variant, got:\n{rust}"
        );
        assert!(
            rust.contains("F64(f64)"),
            "Expected F64 variant, got:\n{rust}"
        );
    }
}
