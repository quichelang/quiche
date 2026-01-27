use crate::Codegen;
use rustpython_parser::ast;

impl Codegen {
    pub(crate) fn generate_stmt(&mut self, stmt: ast::Stmt) {
        match stmt {
            ast::Stmt::FunctionDef(f) => {
                self.generate_function_def(f);
            }
            ast::Stmt::If(i) => {
                self.push_indent();
                self.output.push_str("if ");
                self.generate_expr(*i.test);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for stmt in i.body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
                if !i.orelse.is_empty() {
                    self.output.push_str(" else {\n");
                    self.indent_level += 1;
                    for stmt in i.orelse {
                        self.generate_stmt(stmt);
                    }
                    self.indent_level -= 1;
                    self.push_indent();
                    self.output.push_str("}");
                }
                self.output.push_str("\n");
            }
            ast::Stmt::While(w) => {
                self.push_indent();
                self.output.push_str("while ");
                self.generate_expr(*w.test);
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for stmt in w.body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            ast::Stmt::For(f) => {
                self.push_indent();
                self.output.push_str("for ");
                self.generate_expr(*f.target);
                self.output.push_str(" in (");
                self.generate_expr(*f.iter);
                self.output
                    .push_str(").into_iter().map(|__q| quiche::check!(__q)) {\n");
                self.indent_level += 1;
                for stmt in f.body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            ast::Stmt::Assign(a) => {
                self.push_indent();
                // Reassignment: x = y
                // If x is new, emit 'let mut x'
                for (i, target) in a.targets.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" = ");
                    }

                    if i == 0 && a.targets.len() == 1 {
                        if let ast::Expr::Name(n) = target {
                            if !self.has_symbol(&n.id) {
                                self.output.push_str("let mut ");
                                self.add_symbol(n.id.to_string(), "/* inferred */".to_string());
                            }
                        }
                    }

                    self.generate_expr(target.clone());
                }
                self.output.push_str(" = ");
                self.generate_expr(*a.value);
                self.output.push_str(";\n");
            }
            ast::Stmt::AnnAssign(a) => {
                self.push_indent();
                // Handle: target: annotation = value
                self.output.push_str("let mut ");
                let target_str = self.expr_to_string(&a.target);
                self.output.push_str(&target_str);
                self.output.push_str(": ");
                let type_ann = self.map_type(&a.annotation);
                self.output.push_str(&type_ann);

                // Register symbol
                self.add_symbol(target_str, type_ann);

                if let Some(value) = &a.value {
                    self.output.push_str(" = ");
                    self.generate_expr(*value.clone());
                }
                self.output.push_str(";\n");
            }
            ast::Stmt::Return(r) => {
                self.push_indent();
                self.output.push_str("return ");
                if let Some(expr) = r.value {
                    self.generate_expr(*expr);
                }
                self.output.push_str(";\n");
            }
            ast::Stmt::Expr(e) => {
                self.push_indent();
                self.generate_expr(*e.value);
                self.output.push_str(";\n");
            }
            ast::Stmt::Try(t) => {
                self.push_indent();
                self.output.push_str("let _quiche_try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {\n");
                self.indent_level += 1;
                for stmt in t.body {
                    self.generate_stmt(stmt);
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}));\n");

                self.push_indent();
                self.output
                    .push_str("if let Err(_quiche_err) = _quiche_try_result {\n");
                self.indent_level += 1;

                for handler in t.handlers {
                    match handler {
                        ast::ExceptHandler::ExceptHandler(inner) => {
                            if let Some(name) = &inner.name {
                                self.push_indent();
                                self.output.push_str(&format!("let {} = _quiche_err.downcast_ref::<String>().map(|s| s.clone()).or_else(|| _quiche_err.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| \"Unknown Error\".to_string());\n", name));
                            }

                            for stmt in &inner.body {
                                self.generate_stmt(stmt.clone());
                            }
                        }
                    }
                }

                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            ast::Stmt::ClassDef(c) => {
                self.push_indent();

                // Check for @extern(path="...")
                // We need to implement extract_extern_path helper or inline it here
                let mut extern_path = None;
                for decorator in &c.decorator_list {
                    if let ast::Expr::Call(call) = decorator {
                        if let ast::Expr::Name(n) = &*call.func {
                            if n.id.as_str() == "extern" {
                                for keyword in &call.keywords {
                                    if let Some(arg) = &keyword.arg {
                                        if arg == "path" {
                                            if let ast::Expr::Constant(c) = &keyword.value {
                                                if let ast::Constant::Str(s) = &c.value {
                                                    extern_path = Some(s.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(path) = extern_path {
                    // type Vector<T> = std::vec::Vec<T>;
                    // Minimal generic support for now
                    self.output
                        .push_str(&format!("type {}<T> = {}<T>;\n", c.name, path));
                    return;
                }

                // Check for @enum decorator
                let is_enum = c.decorator_list.iter().any(|d| {
                    if let ast::Expr::Name(n) = d {
                        n.id.as_str() == "enum"
                    } else {
                        false
                    }
                });

                if is_enum {
                    self.output.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                    self.push_indent();
                    self.output.push_str(&format!("enum {} {{\n", c.name));
                    self.indent_level += 1;

                    for stmt in &c.body {
                        if let ast::Stmt::AnnAssign(a) = stmt {
                            self.push_indent();
                            let variant_name = self.expr_to_string(&a.target);
                            self.output.push_str(&variant_name);

                            // Parse types from annotation List: [T1, T2]
                            if let ast::Expr::List(l) = &*a.annotation {
                                if !l.elts.is_empty() {
                                    self.output.push_str("(");
                                    for (i, t) in l.elts.iter().enumerate() {
                                        if i > 0 {
                                            self.output.push_str(", ");
                                        }
                                        self.output.push_str(&self.map_type(t));
                                    }
                                    self.output.push_str(")");
                                }
                            }
                            self.output.push_str(",\n");
                        }
                    }

                    self.indent_level -= 1;
                    self.push_indent();
                    self.output.push_str("}\n\n");
                } else {
                    // Generate struct definition
                    self.output.push_str("#[derive(Clone, Debug)]\n");
                    self.push_indent();
                    self.output.push_str(&format!("struct {} {{\n", c.name));

                    let mut methods = Vec::new();

                    self.indent_level += 1;
                    for stmt in &c.body {
                        match stmt {
                            ast::Stmt::AnnAssign(a) => {
                                self.push_indent();
                                self.output.push_str(&self.expr_to_string(&a.target));
                                self.output.push_str(": ");
                                self.output.push_str(&self.map_type(&a.annotation));
                                self.output.push_str(",\n");
                            }
                            ast::Stmt::FunctionDef(f) => {
                                methods.push(f);
                            }
                            _ => {
                                self.push_indent();
                                self.output.push_str(
                                    "// Only fields (annotated) and methods supported in class\n",
                                );
                            }
                        }
                    }
                    self.indent_level -= 1;
                    self.push_indent();
                    self.output.push_str("}\n\n");

                    // Generate impl block
                    if !methods.is_empty() {
                        self.push_indent();
                        self.output.push_str(&format!("impl {} {{\n", c.name));
                        self.indent_level += 1;

                        for f in methods {
                            self.generate_function_def(f.clone());
                        }

                        self.indent_level -= 1;
                        self.push_indent();
                        self.output.push_str("}\n\n");
                    }
                }
            }
            ast::Stmt::Import(i) => {
                self.push_indent();
                for alias in i.names {
                    let name = alias.name.as_str().replace(".", "::");
                    if let Some(asname) = alias.asname {
                        self.output
                            .push_str(&format!("use {} as {};\n", name, asname));
                    } else {
                        self.output.push_str(&format!("use {};\n", name));
                    }
                }
            }
            ast::Stmt::ImportFrom(i) => {
                self.push_indent();
                if let Some(module) = &i.module {
                    if module.as_str() == "lib.test" {
                        self.output
                            .push_str("// skipped lib.test import (using native macros)\n");
                        return;
                    }

                    let mut mod_name = module.as_str().replace(".", "::");
                    let mut is_rust_interop = false;

                    if module.as_str() == "rust" {
                        is_rust_interop = true;
                        mod_name = String::new();
                    } else if module.as_str().starts_with("rust.") {
                        is_rust_interop = true;
                        mod_name = module
                            .as_str()
                            .strip_prefix("rust.")
                            .unwrap()
                            .replace(".", "::");
                    }

                    for alias in i.names {
                        let name = alias.name.as_str();
                        let target_name = if let Some(asname) = &alias.asname {
                            asname.as_str()
                        } else {
                            name
                        };

                        if is_rust_interop {
                            self.foreign_symbols.insert(target_name.to_string());
                        }

                        if mod_name.is_empty() {
                            // from rust import crate -> use crate;
                            if let Some(asname) = alias.asname {
                                self.output
                                    .push_str(&format!("use {} as {};\n", name, asname));
                            } else {
                                self.output.push_str(&format!("use {};\n", name));
                            }
                        } else if let Some(asname) = alias.asname {
                            self.output
                                .push_str(&format!("use {}::{} as {};\n", mod_name, name, asname));
                        } else {
                            self.output
                                .push_str(&format!("use {}::{};\n", mod_name, name));
                        }
                    }
                }
            }
            ast::Stmt::Match(m) => {
                self.push_indent();
                self.output.push_str("match ");
                self.generate_expr(*m.subject);
                self.output.push_str(" {\n");
                self.indent_level += 1;

                for case in m.cases {
                    self.push_indent();
                    self.generate_pattern(&case.pattern);
                    if let Some(guard) = case.guard {
                        self.output.push_str(" if ");
                        self.generate_expr(*guard);
                    }
                    self.output.push_str(" => {\n");
                    self.indent_level += 1;
                    for stmt in case.body {
                        self.generate_stmt(stmt);
                    }
                    self.indent_level -= 1;
                    self.push_indent();
                    self.output.push_str("}\n");
                }

                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            _ => {
                self.push_indent();
                self.output.push_str("// Unimplemented statement\n");
            }
        }
    }

    pub(crate) fn generate_pattern(&mut self, pat: &ast::Pattern) {
        match pat {
            ast::Pattern::MatchValue(v) => {
                self.generate_expr(*v.value.clone());
            }
            ast::Pattern::MatchAs(a) => {
                if let Some(name) = &a.name {
                    self.output.push_str(name);
                    // Add symbol to scope for inference? Pattern bindings imply new variables.
                    self.add_symbol(name.to_string(), "/* inferred pattern bind */".to_string());
                } else {
                    self.output.push_str("_"); // Anonymous bind? or wildcard
                }
                if let Some(pattern) = &a.pattern {
                    self.output.push_str(" @ ");
                    self.generate_pattern(pattern);
                }
            }
            ast::Pattern::MatchClass(c) => {
                // Class name: Shape.Circle -> Shape::Circle
                // Self-contained logic to print class path
                match &*c.cls {
                    ast::Expr::Attribute(a) => {
                        self.output.push_str(&self.expr_to_string(&a.value));
                        self.output.push_str("::");
                        self.output.push_str(&a.attr);
                    }
                    ast::Expr::Name(n) => {
                        self.output.push_str(&n.id);
                    }
                    _ => self.output.push_str("/* unknown match class */"),
                }

                if !c.patterns.is_empty() {
                    // Tuple style: (p1, p2)
                    self.output.push_str("(");
                    for (i, p) in c.patterns.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_pattern(p);
                    }
                    self.output.push_str(")");
                } else if !c.kwd_attrs.is_empty() {
                    // Struct style: { k: p, .. }
                    self.output.push_str(" { ");
                    for (i, (attr, p)) in c.kwd_attrs.iter().zip(c.kwd_patterns.iter()).enumerate()
                    {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(attr);
                        self.output.push_str(": ");
                        self.generate_pattern(p);
                    }
                    self.output.push_str(", .. }");
                }
            }
            ast::Pattern::MatchStar(_) => {
                self.output.push_str(".."); // Wildcard match in list/slice
            }
            ast::Pattern::MatchOr(o) => {
                for (i, p) in o.patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.generate_pattern(p);
                }
            }
            _ => {
                // Fallback (e.g. wildcards often Parse as MatchAs without name if Python < 3.10? No, Python 3.10 wildcard is MatchAs(None))
                // Actually MatchAs name is Option. If None -> `_`.
                self.output.push_str("_");
            }
        }
    }

    pub(crate) fn generate_function_def(&mut self, f: ast::StmtFunctionDef) {
        self.push_indent();

        if let Some(path) = self.extract_extern_path(&f.decorator_list) {
            self.output
                .push_str(&format!("pub use {} as {};\n", path, f.name));
            return;
        }

        let is_main_with_args = f.name.as_str() == "main" && !f.args.args.is_empty();

        if is_main_with_args {
            self.output.push_str("fn main() {\n");
        } else {
            // Auto-detect test functions
            if f.name.starts_with("test_") {
                self.output.push_str("#[test]\n");
                self.push_indent();
            }
            self.output.push_str(&format!("pub fn {}(", f.name));

            // Generate arguments
            for (i, arg_with_default) in f.args.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                let arg = &arg_with_default.def;
                if arg.arg.as_str() == "self" {
                    self.output.push_str("&self");
                } else {
                    let type_ann = if let Some(annotation) = &arg.annotation {
                        self.map_type(annotation)
                    } else {
                        "/* untyped */".to_string()
                    };
                    self.output.push_str(&format!("{}: {}", arg.arg, type_ann));
                }
            }

            self.output.push_str(")");

            // Return type
            if let Some(ret_expr) = f.returns {
                self.output
                    .push_str(&format!(" -> {}", self.map_type(&ret_expr)));
            }

            self.output.push_str(" {\n");
        }

        self.indent_level += 1;
        self.enter_scope(); // Start function scope

        // Register arguments in new scope
        if !is_main_with_args {
            for arg_with_default in f.args.args.iter() {
                let arg = &arg_with_default.def;
                if arg.arg.as_str() != "self" {
                    if let Some(annotation) = &arg.annotation {
                        let type_ann = self.map_type(annotation);
                        self.add_symbol(arg.arg.to_string(), type_ann);
                    }
                }
            }
        }

        // Inject args extraction for main
        if is_main_with_args {
            self.push_indent();
            // Assuming first arg is 'args'
            if let Some(arg) = f.args.args.first() {
                self.output.push_str(&format!(
                    "let {}: Vec<String> = std::env::args().collect();\n",
                    arg.def.arg
                ));
                self.add_symbol(arg.def.arg.to_string(), "Vec<String>".to_string());
            }
        }

        for stmt in f.body {
            self.generate_stmt(stmt);
        }

        self.exit_scope(); // End function scope
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}\n\n");
    }

    fn extract_extern_path(&self, decorators: &[ast::Expr]) -> Option<String> {
        for d in decorators {
            if let ast::Expr::Call(c) = d {
                if let ast::Expr::Name(n) = &*c.func {
                    if n.id.as_str() == "extern" {
                        for kw in &c.keywords {
                            if let Some(arg) = &kw.arg {
                                if arg == "path" {
                                    if let ast::Expr::Constant(const_val) = &kw.value {
                                        if let ast::Constant::Str(s) = &const_val.value {
                                            return Some(s.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
