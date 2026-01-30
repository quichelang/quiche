use ruff_python_ast as ast;
use ruff_python_parser::parse_module;
use std::collections::{HashMap, HashSet};

pub mod expr;
pub mod stmt;
pub mod types;

pub struct Codegen {
    pub(crate) output: String,
    pub(crate) indent_level: usize,
    pub(crate) scopes: Vec<HashMap<String, String>>,
    pub(crate) defined_vars: Vec<HashSet<String>>,
    pub(crate) foreign_symbols: HashSet<String>,
    pub(crate) linked_modules: HashSet<String>,
    pub(crate) class_fields: HashMap<String, HashMap<String, String>>,
    pub(crate) current_class: Option<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            scopes: vec![HashMap::new()],
            defined_vars: vec![HashSet::new()],
            foreign_symbols: HashSet::new(),
            linked_modules: HashSet::new(),
            class_fields: HashMap::new(),
            current_class: None,
        }
    }

    pub fn generate_module(&mut self, module: &ast::ModModule) -> String {
        {
            {
                let mut linked = HashSet::new();
                let mut filtered_body = Vec::new();

                for stmt in &module.body {
                    let mut is_hint = false;
                    if let ast::Stmt::Expr(e) = &stmt {
                        if let ast::Expr::StringLiteral(s) = &*e.value {
                            let s = s.value.to_str();
                            if s.starts_with("quiche:link=") {
                                let links = &s["quiche:link=".len()..];
                                for link in links.split(',') {
                                    linked.insert(link.trim().to_string());
                                }
                                is_hint = true;
                            }
                        }
                    }
                    if !is_hint {
                        filtered_body.push(stmt);
                    }
                }

                self.linked_modules = linked;
                for stmt in filtered_body {
                    self.generate_stmt(stmt.clone());
                }
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
        self.defined_vars.push(HashSet::new());
    }

    pub(crate) fn exit_scope(&mut self) {
        self.scopes.pop();
        self.defined_vars.pop();
    }

    pub(crate) fn mark_defined(&mut self, name: &str) {
        if let Some(scope) = self.defined_vars.last_mut() {
            scope.insert(name.to_string());
        }
    }

    pub(crate) fn is_defined(&self, name: &str) -> bool {
        self.defined_vars.iter().rev().any(|s| s.contains(name))
    }

    pub(crate) fn add_symbol(&mut self, name: String, ty: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    #[allow(dead_code)]
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
            || base_str == "ruff_python_parser"
            || base_str == "ruff_python_ast"
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

    pub(crate) fn register_class_field(&mut self, class: &str, field: &str, ty: String) {
        self.class_fields
            .entry(class.to_string())
            .or_default()
            .insert(field.to_string(), ty);
    }

    pub(crate) fn set_current_class(&mut self, class: &str) {
        self.current_class = Some(class.to_string());
    }

    pub(crate) fn clear_current_class(&mut self) {
        self.current_class = None;
    }

    pub(crate) fn get_self_field_type(&self, field: &str) -> Option<String> {
        let class = self.current_class.as_ref()?;
        self.class_fields
            .get(class)
            .and_then(|fields| fields.get(field).cloned())
    }
}

pub fn compile(source: &str) -> Option<String> {
    match parse_module(source) {
        Ok(parsed) => {
            let mut cg = Codegen::new();
            let rust_code = cg.generate_module(parsed.syntax());
            // println!("Successfully generated Rust code:\n{}", rust_code);
            Some(dedup_shadowed_let_mut(&rust_code))
        }
        Err(_e) => None,
    }
}

fn dedup_shadowed_let_mut(code: &str) -> String {
    let mut out = String::new();
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];

    for line in code.lines() {
        let mut line_out = line.to_string();

        for ch in line.chars() {
            if ch == '}' && scopes.len() > 1 {
                scopes.pop();
            }
        }

        let mut search_start = 0;
        loop {
            if let Some(idx) = line_out[search_start..].find("let mut ") {
                let abs_idx = search_start + idx;
                let name_start = abs_idx + "let mut ".len();
                let mut name_end = name_start;
                for (i, c) in line_out[name_start..].char_indices() {
                    if c.is_alphanumeric() || c == '_' {
                        name_end = name_start + i + c.len_utf8();
                    } else {
                        break;
                    }
                }
                if name_end == name_start {
                    search_start = name_start;
                    continue;
                }

                let name = line_out[name_start..name_end].to_string();
                let shadowed = scopes
                    .iter()
                    .take(scopes.len().saturating_sub(1))
                    .any(|s| s.contains(&name));
                if shadowed {
                    line_out.replace_range(abs_idx..name_start, "");
                } else if let Some(cur) = scopes.last_mut() {
                    cur.insert(name);
                }
                search_start = name_end;
            } else {
                break;
            }
        }

        for ch in line.chars() {
            if ch == '{' {
                scopes.push(HashSet::new());
            }
        }

        out.push_str(&line_out);
        out.push('\n');
    }

    out
}
