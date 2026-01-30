use crate::Codegen;
use ruff_python_ast as ast;

impl Codegen {
    fn emit_attribute_assign(&mut self, attr: &ast::ExprAttribute, value: &ast::Expr) {
        let target = self.expr_to_string(&ast::Expr::Attribute(attr.clone()));
        self.output.push_str(&target);
        self.output.push_str(" = ");
        self.generate_expr(value.clone());
        self.output.push_str(";\n");
    }

    fn emit_subscript_assign(&mut self, sub: &ast::ExprSubscript, value: &ast::Expr) {
        // Strict 1:1 mapping: d[k] = v -> val[idx] = v.
        // If val is HashMap, this fails in Rust (use .insert).
        // User must write d.insert(k, v) in Quiche.

        self.generate_expr(*sub.value.clone());
        self.output.push_str("[");
        self.generate_expr(*sub.slice.clone());
        self.output.push_str("] = ");
        self.generate_expr(value.clone());
        self.output.push_str(";\n");
    }
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
                self.enter_scope();
                for stmt in i.body {
                    self.generate_stmt(stmt);
                }
                self.exit_scope();
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}");
                for clause in i.elif_else_clauses {
                    if let Some(test) = clause.test {
                        self.output.push_str(" else if ");
                        self.generate_expr(test.clone());
                        self.output.push_str(" {\n");
                    } else {
                        self.output.push_str(" else {\n");
                    }
                    self.indent_level += 1;
                    self.enter_scope();
                    for stmt in clause.body {
                        self.generate_stmt(stmt);
                    }
                    self.exit_scope();
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
                self.enter_scope();
                for stmt in w.body {
                    self.generate_stmt(stmt);
                }
                self.exit_scope();
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            ast::Stmt::For(f) => {
                self.push_indent();
                self.output.push_str("for __q in (");
                self.generate_expr(*f.iter);
                // In Rust, for loop iterates using IntoIterator.
                // quiche code: for x in vec -> for x in vec
                // quiche code: for k in dict -> for k in dict.keys() in Python, but Rust HashMap iterates (k, v) or keys depending on usage?
                // Actually Rust HashMap iterates (k, v) by default. Python iterates keys.
                // Strict 1:1 -> user writes `for k in d.keys()` if they want keys.
                // If user writes `for x in d`, they get `for x in d`.
                // Rust hashmap `for (k, v) in d` works.
                // So providing raw `f.iter` is correct 1:1 behavior, relying on Rust's behavior.
                self.output.push_str(") {\n");
                self.indent_level += 1;
                self.enter_scope();
                self.push_indent();
                self.output.push_str("let ");
                self.generate_expr(*f.target.clone());
                self.output.push_str(" = __q;\n");
                for stmt in f.body {
                    self.generate_stmt(stmt);
                }
                self.exit_scope();
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            ast::Stmt::Assign(a) => {
                self.push_indent();
                if a.targets.len() == 1 {
                    if let ast::Expr::Subscript(s) = &a.targets[0] {
                        self.emit_subscript_assign(s, &a.value);
                        return;
                    }
                    if let ast::Expr::Attribute(a_attr) = &a.targets[0] {
                        self.emit_attribute_assign(a_attr, &a.value);
                        return;
                    }
                }
                // Reassignment: x = y
                // If x is new, emit 'let mut x'
                for (i, target) in a.targets.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" = ");
                    }

                    if i == 0 && a.targets.len() == 1 {
                        if let ast::Expr::Name(n) = target {
                            let already_defined =
                                self.is_defined(&n.id) || self.get_symbol(&n.id).is_some();
                            if !already_defined {
                                self.output.push_str("let mut ");
                                let mut inferred = "/* inferred */".to_string();
                                match &*a.value {
                                    ast::Expr::List(_) => inferred = "List".to_string(),
                                    ast::Expr::Dict(_) => inferred = "Dict".to_string(),
                                    ast::Expr::Call(c) => match &*c.func {
                                        ast::Expr::Name(fn_name) => {
                                            inferred = fn_name.id.to_string();
                                        }
                                        ast::Expr::Attribute(attr) => {
                                            if attr.attr.as_str() == "new" {
                                                inferred = self.expr_to_string(&attr.value);
                                            }
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                }
                                self.add_symbol(n.id.to_string(), inferred);
                                self.mark_defined(&n.id);
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
                self.add_symbol(target_str.clone(), type_ann);
                self.mark_defined(&target_str);

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
                // Try/Except removed from MetaQuiche (Moved to quiche-to-metaquiche)            self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n");
            }
            ast::Stmt::ClassDef(c) => {
                self.push_indent();

                // Check for @extern(path="...", no_generic=true)
                let mut extern_path = None;
                let mut no_generic = false;

                for decorator in &c.decorator_list {
                    if let ast::Expr::Call(call) = &decorator.expression {
                        if let ast::Expr::Name(n) = &*call.func {
                            if n.id.as_str() == "extern" {
                                for keyword in &call.arguments.keywords {
                                    if let Some(arg) = &keyword.arg {
                                        if arg == "path" {
                                            if let ast::Expr::StringLiteral(s) = &keyword.value {
                                                extern_path = Some(s.value.to_string());
                                            }
                                        } else if arg == "no_generic" {
                                            match &keyword.value {
                                                ast::Expr::BooleanLiteral(b) => {
                                                    no_generic = b.value;
                                                }
                                                ast::Expr::Name(n) => {
                                                    if n.id.as_str() == "true" {
                                                        no_generic = true;
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(path) = extern_path {
                    if no_generic {
                        self.output
                            .push_str(&format!("pub type {} = {};\n", c.name, path));
                    } else {
                        // type Vector<T> = std::vec::Vec<T>;
                        self.output
                            .push_str(&format!("pub type {}<T> = {}<T>;\n", c.name, path));
                    }
                    return;
                }

                // Check for @enum decorator
                let is_enum = c.decorator_list.iter().any(|d| {
                    if let ast::Expr::Name(n) = &d.expression {
                        n.id.as_str() == "enum"
                    } else {
                        false
                    }
                });

                if is_enum {
                    self.output.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                    self.push_indent();
                    self.output.push_str(&format!("pub enum {} {{\n", c.name));
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
                    self.output.push_str("#[derive(Clone, Debug, Default)]\n");
                    self.push_indent();
                    self.output.push_str(&format!("pub struct {} {{\n", c.name));

                    let mut methods = Vec::new();
                    let class_name = c.name.to_string();

                    self.indent_level += 1;
                    for stmt in &c.body {
                        match stmt {
                            ast::Stmt::AnnAssign(a) => {
                                let field_name = self.expr_to_string(&a.target);
                                let field_type = self.map_type(&a.annotation);
                                self.register_class_field(
                                    &class_name,
                                    &field_name,
                                    field_type.clone(),
                                );
                                self.push_indent();
                                self.output.push_str("pub ");
                                self.output.push_str(&field_name);
                                self.output.push_str(": ");
                                self.output.push_str(&field_type);
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
                        self.set_current_class(&class_name);

                        for f in methods {
                            self.generate_function_def(f.clone());
                        }

                        self.clear_current_class();
                        self.indent_level -= 1;
                        self.push_indent();
                        self.output.push_str("}\n\n");
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

                        let is_likely_local = !is_rust_interop
                            && !mod_name.contains("std::")
                            && !mod_name.contains("quiche_runtime");
                        let final_mod_path = if is_likely_local && i.level == 0 {
                            format!("crate::{}", mod_name)
                        } else {
                            mod_name.clone()
                        };

                        if mod_name.is_empty() {
                            // Import: "from rust import crate" or "import module"
                            // If it's a simple name and not strict "rust" or "std", assume crate-local? (Or simple crate dependency)
                            // Ideally, we'd check against a list of local modules.
                            // For now, let's assume if it's NOT a known 3rd party or std, it's local if no dots.

                            // NOTE: This logic is heuristic-based for the prototype.
                            let is_likely_local = !name.contains("::")
                                && name != "std"
                                && name != "rust"
                                && name != "rustpython_parser";

                            let use_path = if is_likely_local {
                                format!("crate::{}", name)
                            } else {
                                name.to_string()
                            };

                            if let Some(asname) = alias.asname {
                                self.output
                                    .push_str(&format!("use {} as {};\n", use_path, asname));
                            } else {
                                self.output.push_str(&format!("use {};\n", use_path));
                            }
                        } else if let Some(asname) = alias.asname {
                            self.output.push_str(&format!(
                                "use {}::{} as {};\n",
                                final_mod_path, name, asname
                            ));
                        } else {
                            self.output
                                .push_str(&format!("use {}::{};\n", final_mod_path, name));
                        }
                    }
                }
            }
            ast::Stmt::Import(i) => {
                self.push_indent();
                for alias in i.names {
                    let name = alias.name.as_str();
                    // Heuristic for local vs crate import
                    let is_likely_local = !name.contains(".")
                        && name != "std"
                        && name != "rust"
                        && name != "rustpython_parser";

                    let use_path = if is_likely_local {
                        if self.linked_modules.contains(name) {
                            continue;
                        }
                        format!("crate::{}", name.replace(".", "::"))
                    } else {
                        name.replace(".", "::")
                    };

                    if let Some(asname) = alias.asname {
                        self.output.push_str(&format!(
                            "use {} as {};\n",
                            use_path,
                            asname.as_str()
                        ));
                    } else {
                        self.output.push_str(&format!("use {};\n", use_path));
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
                    self.enter_scope();
                    for stmt in case.body {
                        self.generate_stmt(stmt);
                    }
                    self.exit_scope();
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
                self.output.push_str("// Unimplemented stmt\n");
            }
        }
    }

    pub(crate) fn generate_pattern(&mut self, pat: &ast::Pattern) {
        match pat {
            ast::Pattern::MatchValue(v) => {
                // For enum variant patterns like ast.CmpOp.Eq, use emit_pattern_path
                fn emit_pattern_path(codegen: &mut Codegen, expr: &ast::Expr) {
                    match expr {
                        ast::Expr::Name(n) => {
                            codegen.output.push_str(&n.id);
                        }
                        ast::Expr::Attribute(a) => {
                            emit_pattern_path(codegen, &a.value);
                            codegen.output.push_str("::");
                            codegen.output.push_str(&a.attr);
                        }
                        _ => codegen.generate_expr(expr.clone()),
                    }
                }
                emit_pattern_path(self, &v.value);
            }
            ast::Pattern::MatchAs(a) => {
                if let Some(name) = &a.name {
                    self.output.push_str(&name.id);
                    // Add symbol to scope for inference? Pattern bindings imply new variables.
                    self.add_symbol(
                        name.id.to_string(),
                        "/* inferred pattern bind */".to_string(),
                    );
                } else {
                    self.output.push_str("_"); // Anonymous bind? or wildcard
                }
                if let Some(pattern) = &a.pattern {
                    self.output.push_str(" @ ");
                    self.generate_pattern(pattern);
                }
            }
            ast::Pattern::MatchClass(c) => {
                // Class name: ast.Stmt.FunctionDef -> ast::Stmt::FunctionDef
                // Recursively emit path with :: for all segments
                fn emit_pattern_path(codegen: &mut Codegen, expr: &ast::Expr) {
                    match expr {
                        ast::Expr::Name(n) => {
                            codegen.output.push_str(&n.id);
                        }
                        ast::Expr::Attribute(a) => {
                            emit_pattern_path(codegen, &a.value);
                            codegen.output.push_str("::");
                            codegen.output.push_str(&a.attr);
                        }
                        _ => codegen.output.push_str("/* unknown pattern path */"),
                    }
                }
                emit_pattern_path(self, &c.cls);

                if !c.arguments.patterns.is_empty() {
                    // Tuple style: (p1, p2)
                    self.output.push_str("(");
                    for (i, p) in c.arguments.patterns.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_pattern(p);
                    }
                    self.output.push_str(")");
                } else if !c.arguments.keywords.is_empty() {
                    // Struct style: { k: p, .. }
                    self.output.push_str(" { ");
                    for (i, kw) in c.arguments.keywords.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(&kw.attr);
                        self.output.push_str(": ");
                        self.generate_pattern(&kw.pattern);
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

        if let Some((path, _no_generic)) = self.extract_extern_path(&f.decorator_list) {
            self.output
                .push_str(&format!("pub use {} as {};\n", path, f.name));
            return;
        }

        let is_main_with_args = f.name.as_str() == "main" && !f.parameters.args.is_empty();

        if is_main_with_args {
            self.output.push_str("fn main() {\n");
        } else {
            // Auto-detect test functions
            if f.name.starts_with("test_") {
                self.output.push_str("#[test]\n");
                self.push_indent();
            }
            self.output.push_str(&format!("pub fn {}(", f.name));

            // self.output.push_str(&format!("// method_mutates_self: {}\n", needs_mut_self)); // Removed to match native output

            // Generate arguments
            // Ruff parameters.args returns Vec<ParameterWithDefault> ? No, check AST
            // ParameterWithDefault { parameter: Parameter, default: Option<Box<Expr>>, .. }
            for (i, param_with_default) in f.parameters.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                let arg = &param_with_default.parameter; // Ruff renaming from def to parameter
                if arg.name.as_str() == "self" {
                    self.output.push_str("&mut self");
                } else {
                    let type_ann = if let Some(annotation) = &arg.annotation {
                        self.map_type(annotation)
                    } else {
                        // Emit a compile-time error for untyped parameters
                        // This is better than silently generating invalid Rust
                        format!(
                            "compile_error!(\"Parameter '{}' requires a type annotation\")",
                            arg.name
                        )
                    };
                    self.output.push_str(&format!("{}: {}", arg.name, type_ann));
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

        // Register arguments in new scope (MUST register ALL params, typed or not)
        if !is_main_with_args {
            for param_with_default in f.parameters.args.iter() {
                let arg = &param_with_default.parameter;
                if arg.name.as_str() != "self" {
                    let type_ann = if let Some(annotation) = &arg.annotation {
                        self.map_type(annotation)
                    } else {
                        // Untyped parameters get registered with "unknown" type
                        // to prevent "not found" errors in match arms
                        "unknown".to_string()
                    };
                    self.add_symbol(arg.name.to_string(), type_ann);
                    self.mark_defined(arg.name.as_str());
                }
            }
        }

        // Inject args extraction for main
        if is_main_with_args {
            self.push_indent();
            // Assuming first arg is 'args'
            if let Some(param) = f.parameters.args.first() {
                self.output.push_str(&format!(
                    "let {}: Vec<String> = std::env::args().collect();\n",
                    param.parameter.name
                ));
                self.add_symbol(param.parameter.name.to_string(), "Vec<String>".to_string());
                self.mark_defined(param.parameter.name.as_str());
            }
        }

        for (i, stmt) in f.body.clone().into_iter().enumerate() {
            if i == 0 {
                // Check for docstring
                if let ast::Stmt::Expr(e) = &stmt {
                    if let ast::Expr::StringLiteral(s) = &*e.value {
                        self.push_indent();
                        self.output.push_str("#![doc = \"");
                        self.output
                            .push_str(&s.value.to_str().replace("\"", "\\\""));
                        self.output.push_str("\"]\n");
                        continue;
                    }
                }
            }
            self.generate_stmt(stmt);
        }

        self.exit_scope(); // End function scope
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}\n\n");
    }

    fn extract_extern_path(&self, decorators: &[ast::Decorator]) -> Option<(String, bool)> {
        for d in decorators {
            if let ast::Expr::Call(c) = &d.expression {
                if let ast::Expr::Name(n) = &*c.func {
                    if n.id.as_str() == "extern" {
                        let mut path = None;
                        let mut no_generic = false;
                        for kw in &c.arguments.keywords {
                            if let Some(arg) = &kw.arg {
                                if arg == "path" {
                                    if let ast::Expr::StringLiteral(s) = &kw.value {
                                        path = Some(s.value.to_string());
                                    }
                                } else if arg == "no_generic" {
                                    match &kw.value {
                                        ast::Expr::BooleanLiteral(b) => {
                                            no_generic = b.value;
                                        }
                                        ast::Expr::Name(n) => {
                                            if n.id.as_str() == "true" {
                                                no_generic = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        if let Some(p) = path {
                            return Some((p, no_generic));
                        }
                    }
                }
            }
        }
        None
    }
}
