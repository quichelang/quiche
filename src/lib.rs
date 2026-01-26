use codegen::Codegen;
pub use quiche_codegen as codegen;
use rustpython_parser::{parse, Mode};

pub fn compile(source: &str) -> Option<String> {
    match parse(source, Mode::Module, "input.py") {
        Ok(ast) => {
            let mut cg = Codegen::new();
            let rust_code = cg.generate_module(ast);
            println!("Successfully generated Rust code:\n{}", rust_code);
            Some(rust_code)
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let code = "def foo(x): return x + 1";
        compile(code);
    }

    #[test]
    fn test_parse_class() {
        let code = r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y
"#;
        compile(code);
    }
}
