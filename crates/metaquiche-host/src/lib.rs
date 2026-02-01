// Legacy host compiler - will be deprecated in favor of metaquiche-native
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use metaquiche_shared::telemetry::{CompileContext, Diagnostic, Emitter};
use quiche_parser::ast;
use quiche_parser::parse;
use std::collections::{HashMap, HashSet};

pub mod expr;
pub mod stmt;
pub mod types;

/// Helper macro to get a codegen template or panic if missing
#[macro_export]
macro_rules! codegen_template {
    ($key:expr) => {
        metaquiche_shared::template::codegen_template($key)
            .expect(concat!("Template not found: ", $key))
    };
}

/// Shorthand for codegen_template! to reduce verbosity
macro_rules! T {
    ($key:expr) => {
        codegen_template!($key)
    };
}
pub(crate) use T;

pub struct Codegen {
    pub(crate) output: String,
    pub(crate) indent_level: usize,
    pub(crate) scopes: Vec<HashMap<String, String>>,
    pub(crate) defined_vars: Vec<HashSet<String>>,
    pub(crate) foreign_symbols: HashSet<String>,
    pub(crate) linked_modules: HashSet<String>,
    pub(crate) import_kinds: HashMap<String, String>,
    pub(crate) class_fields: HashMap<String, HashMap<String, String>>,
    pub(crate) current_class: Option<String>,
    pub(crate) source_stack: Vec<String>,
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
            import_kinds: HashMap::new(),
            class_fields: HashMap::new(),
            current_class: None,
            source_stack: Vec::new(),
        }
    }

    pub fn generate_module(&mut self, module: &ast::QuicheModule) -> String {
        {
            {
                let mut linked = HashSet::new();
                let mut filtered_body = Vec::new();

                for stmt in &module.body {
                    let mut is_hint = false;
                    /* Link Hint Logic removed/TODO: If we need linker hints, we need QuicheStmt support or dedicated field */
                    /* Re-implementing simplified hint check if QuicheStmt::Expr(StringLiteral) exists */
                    if let ast::QuicheStmt::Expr(e) = stmt {
                        if let ast::QuicheExpr::Constant(ast::Constant::Str(s)) = &**e {
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

    pub(crate) fn push_indent(&mut self) {}

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
        // Check import_kinds map first
        if let Some(kind) = self.import_kinds.get(base_str) {
            if kind == "mod" {
                return true;
            }
        }

        // Simple heuristic: if it looks like a type (Capitalized)
        if base_str.chars().next().map_or(false, |c| c.is_uppercase()) {
            return true;
        }

        // Also check if it's in linked_modules (which are crates/modules)
        if self.linked_modules.contains(base_str) {
            return true;
        }

        // Explicit list of known modules/types for legacy support
        if base_str == "ast"
            || base_str == "compiler"
            || base_str == "types"
            || base_str == "extern_defs"
            || base_str == "rustpython_parser"
            || base_str == "ruff_python_parser"
            || base_str == "ruff_python_ast"
            || base_str == "q_ast"
            || base_str.starts_with("std::")
            || base_str.starts_with("crate::")
            || base_str.contains("::")
        {
            return true;
        }

        false
    }

    /// Check if an expression appears to be a string (literal or involving strings)
    pub(crate) fn is_string_expr(&self, expr: &ast::QuicheExpr) -> bool {
        match expr {
            ast::QuicheExpr::Constant(ast::Constant::Str(_)) => true,
            ast::QuicheExpr::BinOp { left, op, right } => {
                if *op == ast::Operator::Add {
                    self.is_string_expr(left) || self.is_string_expr(right)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Flatten a chain of Add operations into a list of operands
    pub(crate) fn flatten_add_chain(
        &self,
        expr: &ast::QuicheExpr,
        parts: &mut Vec<ast::QuicheExpr>,
    ) {
        match expr {
            ast::QuicheExpr::BinOp { left, op, right } if *op == ast::Operator::Add => {
                self.flatten_add_chain(left, parts);
                self.flatten_add_chain(right, parts);
            }
            _ => {
                parts.push(expr.clone());
            }
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

/// Compile source code with proper error reporting using telemetry
pub fn compile(source: &str, filename: &str) -> Option<String> {
    let ctx = CompileContext::new(filename, source);

    match parse(source) {
        Ok(parsed) => {
            let mut cg = Codegen::new();
            let rust_code = cg.generate_module(&parsed);
            Some(dedup_shadowed_let_mut(&rust_code))
        }
        Err(e) => {
            // Use telemetry for proper error display
            Emitter::print_failed_header(filename);

            // Extract byte offset and clean error message
            let err_str = e.to_string();
            let byte_offset = extract_byte_offset(&err_str);

            // Clean the raw error message - remove "byte range X..Y" suffix
            let clean_msg = err_str
                .split(" at byte range")
                .next()
                .unwrap_or(&err_str)
                .to_string();

            let mut diag = Diagnostic::error(&clean_msg);
            if let Some(offset) = byte_offset {
                diag = diag.with_span(ctx.byte_to_span(offset));
            }

            eprintln!(
                "{}",
                metaquiche_shared::telemetry::format_diagnostic(&diag, Some(&ctx))
            );
            None
        }
    }
}

/// Extract byte offset from error message containing "byte range X..Y"
fn extract_byte_offset(message: &str) -> Option<usize> {
    let range_start = message.find("byte range ")?;
    let rest = &message[range_start + 11..];
    let (start_str, _) = rest.split_once("..")?;
    start_str.parse().ok()
}

fn dedup_shadowed_let_mut(code: &str) -> String {
    let mut out = String::new();
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];

    for line in code.lines() {
        let mut line_out = line.to_string();
        let mut in_string = false;
        let mut escape = false;

        // Pre-scan line for brace changes, respecting strings
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if !in_string {
                if ch == '}' && scopes.len() > 1 {
                    scopes.pop();
                }
            }
        }

        let mut search_start = 0;
        let mut in_str_detect = false;
        let mut esc_detect = false;

        loop {
            // Find "let mut " but verify it's not in a string
            // This naive search needs to respect string boundaries too
            // Reuse string tracking for replacement as well to be safe
            // But for simplicity, let's assume "let mut " doesn't appear in strings in a way that breaks this unique logic easily?
            // Actually, if we have print!("let mut "), we shouldn't replace it.

            // Simpler approach: Iterate chars and track state, buffer output
            break; // Breaking the loop to rewrite validly below
        }

        // Re-implement the whole logic with proper char iteration
        // This is getting complex to patch.
        // Let's stick to the previous loop structure but add string guard for BRACES mainly,
        // and string guard for 'let mut' search.
    }

    // Better implementation replacing the entire function content:
    let mut out = String::new();
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];

    for line in code.lines() {
        let mut line_out = line.to_string();

        // 1. Update scopes based on '}' at start of line (or before logic?)
        // Logic was: process '}' then process 'let mut' then process '{'
        // But scopes need to be right for the CURRENT line.
        // dedup logic seems to scan line for } to pop scope, then check let mut, then scan for { to push scope.

        let mut in_string = false;
        let mut escape = false;

        // Calculate scope POPS (closing braces)
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if !in_string && ch == '}' && scopes.len() > 1 {
                scopes.pop();
            }
        }

        // Search and replace "let mut " if shadowed
        let mut search_start = 0;
        loop {
            if let Some(idx) = line_out[search_start..].find("let mut ") {
                let abs_idx = search_start + idx;

                // Verify this instance is NOT in a string
                // We need to scan from 0 to abs_idx to check string state
                let prefix = &line_out[0..abs_idx];
                let mut p_in_string = false;
                let mut p_escape = false;
                for c in prefix.chars() {
                    if p_escape {
                        p_escape = false;
                        continue;
                    }
                    if c == '\\' {
                        p_escape = true;
                        continue;
                    }
                    if c == '"' {
                        p_in_string = !p_in_string;
                    }
                }

                if p_in_string {
                    search_start = abs_idx + 8; // skip
                    continue;
                }

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
                    // Adjustment: search_start is now abs_idx (string shrank by 8)
                    search_start = abs_idx;
                } else if let Some(cur) = scopes.last_mut() {
                    cur.insert(name);
                    search_start = name_end;
                } else {
                    search_start = name_end;
                }
            } else {
                break;
            }
        }

        // Calculate scope PUSHES (opening braces)
        in_string = false;
        escape = false;
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if !in_string && ch == '{' {
                scopes.push(HashSet::new());
            }
        }

        out.push_str(&line_out);
        out.push('\n');
    }

    out
}
