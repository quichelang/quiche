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
            move_mut_args: true,
            infer_local_bidi: true,
            effect_rows_internal: true,
            numeric_coercion: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Parse Quiche source into an Elevate AST Module.
pub fn parse(source: &str) -> Result<elevate::ast::Module, String> {
    parser::parse(source).map_err(|e| format!("{e}"))
}

/// Parse Quiche source and compile to Rust via Elevate.
pub fn compile(source: &str) -> Result<String, String> {
    compile_with_options(source, &CompileOptions::default())
}

/// Parse Quiche source and compile to Rust via Elevate with custom options.
pub fn compile_with_options(source: &str, options: &CompileOptions) -> Result<String, String> {
    let module = parser::parse(source).map_err(|e| format!("{e}"))?;
    let output = elevate::compile_ast_with_options(&module, options).map_err(|e| format!("{e}"))?;
    Ok(output.rust_code)
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
    Ok(output.rust_code)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use crate::parser::parse;
    use elevate::ast::*;

    // ─── Functions ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_simple_expression() {
        let module = parse("x + 1").unwrap();
        assert_eq!(module.items.len(), 0); // top-level expr is skipped as item
    }

    #[test]
    fn test_parse_function_def() {
        let source = "def foo(x: int) -> int:\n    return x + 1\n";
        let module = parse(source).unwrap();
        assert_eq!(module.items.len(), 1);
        match &module.items[0] {
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
        let source = "class Point(Struct):\n    x: int\n    y: int\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
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
        let source = "class Point[T](Struct):\n    x: T\n    y: int\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert_eq!(s.fields[0].ty.path, vec!["T"]);
            }
            other => panic!("Expected Struct, got {:?}", other),
        }
    }

    // ─── Enums ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_enum() {
        let source =
            "class Color(Enum):\n    Red = ()\n    Green = (int,)\n    Blue = (int, int)\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Color");
                assert_eq!(e.variants.len(), 3);
                assert_eq!(e.variants[0].name, "Red");
                assert!(e.variants[0].payload.is_empty());
                assert_eq!(e.variants[1].name, "Green");
                assert_eq!(e.variants[1].payload.len(), 1);
                assert_eq!(e.variants[2].name, "Blue");
                assert_eq!(e.variants[2].payload.len(), 2);
            }
            other => panic!("Expected Enum, got {:?}", other),
        }
    }

    // ─── Traits ──────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_trait() {
        let source = "class Drawable(Trait):\n    def draw(self):\n        pass\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
            Item::Trait(t) => {
                assert_eq!(t.name, "Drawable");
            }
            other => panic!("Expected Trait, got {:?}", other),
        }
    }

    // ─── Impl Blocks ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_impl() {
        let source = "@impl(Drawable)\nclass Point:\n    def draw(self): pass\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
            Item::Impl(i) => {
                assert_eq!(i.target, "Point");
            }
            other => panic!("Expected Impl, got {:?}", other),
        }
    }

    // ─── Control Flow ────────────────────────────────────────────────────────

    #[test]
    fn test_parse_if() {
        let source = "def test():\n    if x:\n        y = 1\n    else:\n        y = 2\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
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
        match &module.items[0] {
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
        match &module.items[0] {
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
        match &module.items[0] {
            Item::RustUse(u) => {
                assert_eq!(u.path, vec!["os", "path"]);
            }
            other => panic!("Expected RustUse, got {:?}", other),
        }
    }

    // ─── Match ───────────────────────────────────────────────────────────────

    #[test]
    fn test_parse_match() {
        let source = "def test():\n    match result:\n        case Ok(v):\n            return v\n        case Err(e):\n            print(e)\n";
        let module = parse(source).unwrap();
        match &module.items[0] {
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
        match &module.items[0] {
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
        match &module.items[0] {
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
        match &module.items[0] {
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
        match &module.items[0] {
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
        let source = "class Point(Struct):\n    x: i32\n    y: i32\n";
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
            "class Point(Struct):\n    x: i32\n    y: i32\n\ndef main():\n    p: Point = Point(1, 2)\n",
        );
        assert!(
            rust.contains("Point {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_positional_single_field() {
        let rust = compile_ok(
            "class Wrapper(Struct):\n    val: i32\n\ndef main():\n    w: Wrapper = Wrapper(42)\n",
        );
        assert!(
            rust.contains("Wrapper {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_positional_three_fields() {
        let rust = compile_ok(
            "class Vec3(Struct):\n    x: i32\n    y: i32\n    z: i32\n\ndef main():\n    v: Vec3 = Vec3(1, 2, 3)\n",
        );
        assert!(
            rust.contains("Vec3 {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_positional_mixed_types() {
        let rust = compile_ok(
            "class Person(Struct):\n    name: String\n    age: i32\n\ndef main():\n    p: Person = Person(\"Alice\".to_string(), 30)\n",
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
            "class Point(Struct):\n    x: i32\n    y: i32\n\ndef main():\n    p: Point = Point(x=5, y=10)\n",
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
            "class Point(Struct):\n    x: i32\n    y: i32\n\ndef main():\n    p: Point = Point(y=10, x=5)\n",
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
            "class Color(Struct):\n    r: i32\n    g: i32\n    b: i32\n\ndef main():\n    c: Color = Color(r=255, g=128, b=0)\n",
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
            "class Color(Struct):\n    r: i32\n    g: i32\n    b: i32\n\ndef main():\n    c: Color = Color(b=0, r=255, g=128)\n",
        );
        assert!(
            rust.contains("Color {"),
            "Expected struct literal from shuffled kwargs, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_single_field() {
        let rust = compile_ok(
            "class Wrapper(Struct):\n    val: i32\n\ndef main():\n    w: Wrapper = Wrapper(val=42)\n",
        );
        assert!(
            rust.contains("Wrapper {"),
            "Expected struct literal, got:\n{rust}"
        );
    }

    // --- @impl regression ---

    #[test]
    fn test_struct_construction_after_impl() {
        // Regression: @impl blocks must not overwrite struct field metadata
        let source = "\
class Point(Struct):\n    x: i32\n    y: i32\n\n\
class Printable(Trait):\n    def display(self) -> String: pass\n\n\
@impl(Printable)\nclass Point:\n    def display(self) -> String:\n        return \"Point\"\n\n\
def main():\n    p: Point = Point(5, 5)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("Point { x:"),
            "Expected struct literal after @impl block, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_after_impl() {
        let source = "\
class Point(Struct):\n    x: i32\n    y: i32\n\n\
class Printable(Trait):\n    def display(self) -> String: pass\n\n\
@impl(Printable)\nclass Point:\n    def display(self) -> String:\n        return \"Point\"\n\n\
def main():\n    p: Point = Point(x=5, y=5)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("Point { x:"),
            "Expected keyword struct literal after @impl, got:\n{rust}"
        );
    }

    #[test]
    fn test_struct_keyword_reversed_after_impl() {
        let source = "\
class Point(Struct):\n    x: i32\n    y: i32\n\n\
class Printable(Trait):\n    def display(self) -> String: pass\n\n\
@impl(Printable)\nclass Point:\n    def display(self) -> String:\n        return \"Point\"\n\n\
def main():\n    p: Point = Point(y=10, x=5)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("Point {"),
            "Expected struct literal after @impl, got:\n{rust}"
        );
    }

    // --- Method calls with kwargs ---

    #[test]
    fn test_method_kwargs_parse() {
        // Point.new(x=5, y=5) should parse without errors
        let source = "\
class Point(Struct):\n    x: i32\n    y: i32\n\n\
    def new(x: i32, y: i32) -> Point:\n        return Point(x, y)\n\n\
def main():\n    p: Point = Point.new(x=5, y=5)\n";
        let module = parse(source).unwrap();
        assert!(!module.items.is_empty());
    }

    #[test]
    fn test_method_kwargs_reordered() {
        // Point.new(y=6, x=5) — kwargs should be reordered to match param order
        let source = "\
class Point(Struct):\n    x: i32\n    y: i32\n\n\
    def new(x: i32, y: i32) -> Point:\n        return Point(x, y)\n\n\
def main():\n    p: Point = Point.new(y=6, x=5)\n";
        let rust = compile_ok(source);
        // The compiled output should call new(5, 6) — x first, then y
        assert!(rust.contains("new"), "Expected method call, got:\n{rust}");
    }

    #[test]
    fn test_method_kwargs_three_params() {
        let source = "\
class Vec3(Struct):\n    x: i32\n    y: i32\n    z: i32\n\n\
    def create(x: i32, y: i32, z: i32) -> Vec3:\n        return Vec3(x, y, z)\n\n\
def main():\n    v: Vec3 = Vec3.create(z=3, x=1, y=2)\n";
        let rust = compile_ok(source);
        assert!(
            rust.contains("create"),
            "Expected method call, got:\n{rust}"
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
class Point(Struct):\n    x: i32\n    y: i32\n\n\
class Size(Struct):\n    width: i32\n    height: i32\n\n\
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
}
