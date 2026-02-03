pub mod ast;
pub mod error_fmt;
pub mod fstring;
pub mod lexer;
pub mod parser;

pub use ast::QuicheModule;
pub use error_fmt::{byte_to_line_col, format_error_with_context};

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {

    use crate::ast::*;
    use crate::parser::parse; // Added this line to bring parse into scope

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
                assert_eq!(s.fields[0].ty, "T"); // using compat string extraction
            }
            _ => panic!("Expected StructDef"),
        }
    }

    #[test]
    fn test_expression_parsing() {
        let source = "x + 1";
        let module = parse(source).expect("Parse failed");
        match &module.body[0] {
            QuicheStmt::Expr(e) => match &**e {
                QuicheExpr::BinOp { left, op, right } => {
                    assert_eq!(*op, Operator::Add);
                    match &**left {
                        QuicheExpr::Name(n) => assert_eq!(n, "x"),
                        _ => panic!("Expected Name left"),
                    }
                    match &**right {
                        QuicheExpr::Constant(Constant::Int(1)) => {}
                        _ => panic!("Expected Int right"),
                    }
                }
                _ => panic!("Expected BinOp"),
            },
            _ => panic!("Expected Expr Stmt"),
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
        let source = r#"rust("print!(\"Top\")")"#;
        let module = parse(source).expect("Parse failed");
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
