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
}
