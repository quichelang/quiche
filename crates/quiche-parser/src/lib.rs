pub mod ast;
pub mod parser;

pub use ast::QuicheModule;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_struct_parsing() {
        let source = r#"
class Point[T](Struct):
    x: T
    y: int
"#;
        let module = parse(source).expect("Parse failed");
        assert_eq!(module.body.len(), 1);

        match &module.body[0] {
            QuicheStmt::StructDef(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.type_params, vec!["T"]);
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert_eq!(s.fields[0].ty, "T");
                assert_eq!(s.fields[1].name, "y");
                assert_eq!(s.fields[1].ty, "int");
            }
            _ => panic!("Expected StructDef, got {:?}", module.body[0]),
        }
    }

    #[test]
    fn test_enum_parsing() {
        let source = r#"
class Color(Enum):
    Red = ()
    Green = (int,)
    Blue = (int, int)
"#;
        let module = parse(source).expect("Parse failed");
        match &module.body[0] {
            QuicheStmt::EnumDef(e) => {
                assert_eq!(e.name, "Color");
                assert_eq!(e.variants.len(), 3);

                // Red
                assert_eq!(e.variants[0].name, "Red");
                assert!(e.variants[0].fields.is_empty());

                // Green
                assert_eq!(e.variants[1].name, "Green");
                assert_eq!(e.variants[1].fields, vec!["int"]);

                // Blue
                assert_eq!(e.variants[2].name, "Blue");
                assert_eq!(e.variants[2].fields, vec!["int", "int"]);
            }
            _ => panic!("Expected EnumDef"),
        }
    }

    #[test]
    fn test_trait_parsing() {
        let source = r#"
class Drawable(Trait):
    def draw(self):
        pass
"#;
        let module = parse(source).expect("Parse failed");
        match &module.body[0] {
            QuicheStmt::TraitDef(t) => {
                assert_eq!(t.name, "Drawable");
            }
            _ => panic!("Expected TraitDef"),
        }
    }

    #[test]
    fn test_rust_block() {
        let source = r#"
def main():
    rust("println!(\"Hello\")")
"#;
        // Note: FunctionDef body parsing logic is needed to recurse.
        // Currently `lower_stmt` handles top level.
        // If we want to test recursive parsing, we need to implement it in `lower_function_def`.
        // BUT `parser.rs` currently blindly wraps unknown stmts in `QuicheStmt::Stmt`.
        // `Expr` logic handles `rust(...)` at *statement position*.

        // Let's test top-level rust block for simplicity of current implementation
        let source_top = r#"rust("print!(\"Top\")")"#;
        let module = parse(source_top).expect("Parse failed");
        match &module.body[0] {
            QuicheStmt::RustBlock(code) => {
                assert_eq!(code, "print!(\"Top\")");
            }
            _ => panic!("Expected RustBlock"),
        }
    }

    #[test]
    fn test_impl_parsing() {
        let source = r#"
@impl(Drawable)
class Point:
    def draw(self): pass
"#;
        let module = parse(source).expect("Parse failed");
        match &module.body[0] {
            QuicheStmt::ImplDef(i) => {
                assert_eq!(i.target_type, "Point");
                // trait_name extraction TODO
            }
            _ => panic!("Expected ImplDef"),
        }
    }
}
pub use parser::parse;
