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
    pub(crate) linked_modules: HashSet<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            scopes: vec![HashMap::new()],
            foreign_symbols: HashSet::new(),
            linked_modules: HashSet::new(),
        }
    }

    pub fn generate_module(&mut self, module: ast::Mod) -> String {
        match module {
            ast::Mod::Module(m) => {
                let mut linked = HashSet::new();
                let mut filtered_body = Vec::new();

                for stmt in m.body {
                    let mut is_hint = false;
                    if let ast::Stmt::Expr(e) = &stmt {
                        if let ast::Expr::Constant(c) = &*e.value {
                            if let ast::Constant::Str(s) = &c.value {
                                if s.starts_with("quiche:link=") {
                                    let links = &s["quiche:link=".len()..];
                                    for link in links.split(',') {
                                        linked.insert(link.trim().to_string());
                                    }
                                    is_hint = true;
                                }
                            }
                        }
                    }
                    if !is_hint {
                        filtered_body.push(stmt);
                    }
                }

                self.linked_modules = linked;
                for stmt in filtered_body {
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

    pub(crate) fn is_type_or_mod(&self, base_str: &str) -> bool {
        if base_str == "self" {
            false
        } else if base_str == "ast"
            || base_str == "compiler"
            || base_str == "types"
            || base_str == "rustpython_parser"
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
