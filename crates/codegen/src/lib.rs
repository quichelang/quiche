use rustpython_parser::ast;
use std::collections::HashMap;

pub mod expr;
pub mod stmt;
pub mod types;

pub struct Codegen {
    pub(crate) output: String,
    pub(crate) indent_level: usize,
    pub(crate) scopes: Vec<HashMap<String, String>>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            scopes: vec![HashMap::new()],
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

    pub(crate) fn get_symbol(&self, name: &str) -> Option<&String> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }
}
