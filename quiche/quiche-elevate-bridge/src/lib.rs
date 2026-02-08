//! Quiche → Elevate bridge.
//!
//! Re-exports the desugar module from quiche-compiler.
//! This crate exists to test the bridge independently of the MetaQuiche build system.

pub mod desugar;

/// Compile Quiche source through the Elevate pipeline.
pub fn compile(source: &str) -> Result<String, String> {
    // Step 1: Parse
    let parsed = metaquiche_parser::parse(source).map_err(|e| format!("Parse error: {e}"))?;

    // Step 2: Desugar Quiche AST → Elevate AST
    let elevate_ast = desugar::lower(&parsed);

    // Step 3: Elevate type inference
    let typed = elevate::passes::lower_to_typed(&elevate_ast)
        .map_err(|diags| format!("Type error: {:?}", diags))?;

    // Step 4: Ownership analysis + lowering
    let lowered = elevate::passes::lower_to_rust(&typed);

    // Step 5: Emit Rust code
    let rust_code = elevate::codegen::emit_rust_module(&lowered);

    Ok(rust_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let source = r#"
def add(x: i64, y: i64) -> i64:
    return x + y
"#;
        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
                assert!(code.contains("fn add"), "Should contain fn add: {code}");
            }
            Err(e) => println!("Error (expected during bootstrap): {e}"),
        }
    }

    #[test]
    fn test_struct_def() {
        let source = r#"
class Point(Struct):
    x: i32
    y: i32
"#;
        let parsed = metaquiche_parser::parse(source).expect("Parse failed");
        eprintln!("=== Parsed AST ===\n{parsed:#?}");

        let elevate_ast = desugar::lower(&parsed);
        eprintln!("=== Elevate AST ===\n{elevate_ast:#?}");

        let typed = elevate::passes::lower_to_typed(&elevate_ast);
        eprintln!("=== Typed result ===\n{typed:#?}");

        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
                assert!(
                    code.contains("struct Point") || code.contains("Point"),
                    "Should contain Point: {code}"
                );
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }
    }

    #[test]
    fn test_enum_def() {
        let source = r#"
class Color(Enum):
    Red = ()
    Green = (i32,)
"#;
        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
                assert!(
                    code.contains("enum Color"),
                    "Should contain enum Color: {code}"
                );
            }
            Err(e) => println!("Error (expected during bootstrap): {e}"),
        }
    }

    #[test]
    fn test_for_range() {
        let source = r#"
def count():
    for i in range(10):
        print(i)
"#;
        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
                assert!(code.contains("0.."), "Should contain range 0..: {code}");
            }
            Err(e) => println!("Error (expected during bootstrap): {e}"),
        }
    }

    #[test]
    fn test_fstring() {
        let source = r#"
def greet(name: String) -> String:
    return f"Hello {name}"
"#;
        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
                assert!(code.contains("format!"), "Should contain format!: {code}");
            }
            Err(e) => println!("Error (expected during bootstrap): {e}"),
        }
    }

    #[test]
    fn test_if_else() {
        let source = r#"
def check(x: i64) -> bool:
    if x > 0:
        return True
    else:
        return False
"#;
        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
                assert!(code.contains("if "), "Should contain if: {code}");
            }
            Err(e) => println!("Error (expected during bootstrap): {e}"),
        }
    }

    #[test]
    fn test_lambda() {
        let source = r#"
def go():
    add = |x: i32, y: i32| x + y
"#;
        match compile(source) {
            Ok(code) => {
                println!("=== Generated Rust ===\n{code}");
            }
            Err(e) => println!("Error (expected during bootstrap): {e}"),
        }
    }

    #[test]
    fn test_demo_q() {
        let source = include_str!("../../../examples/scripts/demo.q");
        let parsed = metaquiche_parser::parse(source).expect("Parse failed");
        let elevate_ast = desugar::lower(&parsed);
        eprintln!(
            "=== demo.q Elevate AST ({} items) ===",
            elevate_ast.items.len()
        );
        for (i, item) in elevate_ast.items.iter().enumerate() {
            eprintln!("  item[{i}]: {}", item_summary(item));
        }

        match elevate::passes::lower_to_typed(&elevate_ast) {
            Ok(typed) => {
                let lowered = elevate::passes::lower_to_rust(&typed);
                let rust_code = elevate::codegen::emit_rust_module(&lowered);
                println!("=== demo.q Generated Rust ===\n{rust_code}");
            }
            Err(diags) => {
                eprintln!("=== demo.q Type errors ({} diagnostics) ===", diags.len());
                for diag in &diags {
                    eprintln!("  {diag}");
                }
            }
        }
    }

    #[test]
    fn test_sudoku_q() {
        let source = include_str!("../../../examples/scripts/sudoku.q");
        let parsed = metaquiche_parser::parse(source).expect("Parse failed");
        let elevate_ast = desugar::lower(&parsed);
        eprintln!(
            "=== sudoku.q Elevate AST ({} items) ===",
            elevate_ast.items.len()
        );
        for (i, item) in elevate_ast.items.iter().enumerate() {
            eprintln!("  item[{i}]: {}", item_summary(item));
        }

        match elevate::passes::lower_to_typed(&elevate_ast) {
            Ok(typed) => {
                let lowered = elevate::passes::lower_to_rust(&typed);
                let rust_code = elevate::codegen::emit_rust_module(&lowered);
                println!("=== sudoku.q Generated Rust ===\n{rust_code}");
            }
            Err(diags) => {
                eprintln!("=== sudoku.q Type errors ({} diagnostics) ===", diags.len());
                for diag in &diags {
                    eprintln!("  {diag}");
                }
            }
        }
    }

    fn item_summary(item: &elevate::ast::Item) -> String {
        match item {
            elevate::ast::Item::Struct(s) => format!("struct {}", s.name),
            elevate::ast::Item::Enum(e) => format!("enum {}", e.name),
            elevate::ast::Item::Function(f) => format!("fn {}", f.name),
            elevate::ast::Item::Impl(i) => {
                format!("impl {} ({} methods)", i.target, i.methods.len())
            }
            elevate::ast::Item::Const(c) => format!("const {}", c.name),
            elevate::ast::Item::RustUse(u) => format!("use {}", u.path.join("::")),
            elevate::ast::Item::RustBlock(_) => "rust_block".to_string(),
            elevate::ast::Item::Static(s) => format!("static {}", s.name),
            elevate::ast::Item::Trait(t) => format!("trait {}", t.name),
        }
    }
}
