use rustpython_parser::ast;
use rustpython_parser::{parse, Mode};
use std::collections::{HashMap, HashSet};

pub mod dict;
pub mod expr;
pub mod list;
pub mod stmt;
pub mod types;

pub struct Codegen {
    pub(crate) output: String,
    pub(crate) indent_level: usize,
    pub(crate) scopes: Vec<HashMap<String, String>>,
    pub(crate) foreign_symbols: HashSet<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            scopes: vec![HashMap::new()],
            foreign_symbols: HashSet::new(),
        }
    }

    pub fn generate_module(&mut self, module: ast::Mod) -> String {
        match module {
            ast::Mod::Module(m) => {
                for stmt in m.body {
                    self.generate_stmt(stmt);
                }
            }
            _ => {
                self.output.push_str("// Only modules are supported\n");
            }
        }
        self.output.clone()
    }

    pub(crate) fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }

    pub(crate) fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn add_symbol(&mut self, name: String, ty: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    pub(crate) fn has_symbol(&self, name: &str) -> bool {
        self.get_symbol(name).is_some()
    }

    pub(crate) fn get_symbol(&self, name: &str) -> Option<&String> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}

pub fn compile(source: &str) -> Option<String> {
    match parse(source, Mode::Module, "input.py") {
        Ok(ast) => {
            let mut cg = Codegen::new();
            let rust_code = cg.generate_module(ast);
            // println!("Successfully generated Rust code:\n{}", rust_code);
            Some(rust_code)
        }
        Err(e) => {
            println!("Parse error: {:?}", e);
            None
        }
    }
}
