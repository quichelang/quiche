use crate::codegen_template;
use crate::Codegen;
use metaquiche_parser::ast;

impl Codegen {
    pub(crate) fn generate_stmt(&mut self, stmt: ast::QuicheStmt) {
        match stmt {
            ast::QuicheStmt::FunctionDef(f) => {
                self.generate_function_def(f);
            }
            ast::QuicheStmt::If(i) => {
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
                self.output.push_str("}\n");

                if !i.orelse.is_empty() {
                    // Check if it's an else-if chain or a block
                    self.push_indent();
                    self.output.push_str("else {\n");
                    self.indent_level += 1;
                    self.enter_scope();
                    for stmt in i.orelse {
                        self.generate_stmt(stmt);
                    }
                    self.exit_scope();
                    self.indent_level -= 1;
                    self.push_indent();
                    self.output.push_str("}\n");
                }
            }
            ast::QuicheStmt::While(w) => {
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
            ast::QuicheStmt::For(f) => {
                self.push_indent();
                self.output.push_str("for __q in (");
                self.generate_expr(*f.iter);
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
            ast::QuicheStmt::Assign(a) => {
                self.push_indent();
                if a.targets.len() == 1 {
                    if let ast::QuicheExpr::Subscript { value, slice } = &a.targets[0] {
                        // emit_subscript_assign logic needs update for QuicheExpr
                        self.generate_expr(*value.clone());
                        self.output.push_str("[");
                        self.generate_expr(*slice.clone());
                        self.output.push_str("] = ");
                        self.generate_expr(*a.value.clone());
                        self.output.push_str(";\n");
                        return;
                    }
                    if let ast::QuicheExpr::Attribute { value, attr } = &a.targets[0] {
                        // emit_attribute_assign logic
                        self.generate_expr(*value.clone());
                        self.output.push_str(".");
                        self.output.push_str(attr);
                        self.output.push_str(" = ");
                        self.generate_expr(*a.value.clone());
                        self.output.push_str(";\n");
                        return;
                    }
                }

                for (i, target) in a.targets.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" = ");
                    }
                    if i == 0 && a.targets.len() == 1 {
                        if let ast::QuicheExpr::Name(n) = target {
                            let already_defined =
                                self.is_defined(n) || self.get_symbol(n).is_some();
                            if !already_defined {
                                self.output.push_str("let mut ");
                                self.add_symbol(n.to_string(), "inferred".to_string());
                                self.mark_defined(n);
                            }
                        }
                    }
                    self.generate_expr(target.clone());
                }
                self.output.push_str(" = ");
                self.generate_expr(*a.value);
                self.output.push_str(";\n");
            }
            ast::QuicheStmt::AnnAssign(a) => {
                self.push_indent();
                self.output.push_str("let mut ");
                self.generate_expr(*a.target.clone());
                self.output.push_str(": ");
                // Type handling: Annotation is QuicheExpr. We need map_type to work on QuicheExpr.
                // Assuming map_type is updated or we use generate_expr to string.
                // For now, let's assume we implement map_type for QuicheExpr or convert.
                // Actually `expr_to_string` needs to handle type mapping semantics.
                let type_ann = self.map_type(&a.annotation);
                self.output.push_str(&type_ann);

                if let ast::QuicheExpr::Name(n) = &*a.target {
                    self.add_symbol(n.to_string(), type_ann);
                    self.mark_defined(n);
                }

                if let Some(value) = a.value {
                    self.output.push_str(" = ");
                    self.generate_expr(*value);
                } else {
                    self.output.push_str(" = Default::default()");
                }
                self.output.push_str(";\n");
            }
            ast::QuicheStmt::Return(r) => {
                self.push_indent();
                self.output.push_str("return ");
                if let Some(expr) = r {
                    self.generate_expr(*expr);
                }
                self.output.push_str(";\n");
            }
            ast::QuicheStmt::Expr(e) => {
                self.push_indent();
                self.generate_expr(*e);
                self.output.push_str(";\n");
            }
            ast::QuicheStmt::ConstDef(c) => {
                // Module-level constant: pub const NAME: TYPE = VALUE;
                self.push_indent();
                self.output.push_str("pub const ");
                self.output.push_str(&c.name);
                self.output.push_str(": ");
                self.output.push_str(&self.map_type(&c.ty));
                self.output.push_str(" = ");
                self.generate_expr(*c.value);
                self.output.push_str(";\n");
            }
            ast::QuicheStmt::StructDef(s) => {
                self.output.push_str("#[derive(Clone, Debug, Default)]\n");
                self.push_indent();
                // Private structs start with underscore
                let visibility = if s.name.starts_with('_') { "" } else { "pub " };
                self.output
                    .push_str(&format!("{}struct {} ", visibility, s.name));
                if !s.type_params.is_empty() {
                    self.output.push_str("<");
                    self.output.push_str(&s.type_params.join(", "));
                    self.output.push_str(">");
                }
                self.output.push_str(" {\n");
                self.indent_level += 1;

                for field in &s.fields {
                    self.push_indent();
                    // Private fields start with underscore
                    let field_visibility = if field.name.starts_with('_') {
                        ""
                    } else {
                        "pub "
                    };
                    self.output.push_str(&format!(
                        "{}{}: {},\n",
                        field_visibility, field.name, field.ty
                    ));
                }

                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n\n");
            }
            ast::QuicheStmt::EnumDef(e) => {
                self.output.push_str("#[derive(Clone, Debug, PartialEq)]\n");
                self.push_indent();
                self.output.push_str(&format!("pub enum {} ", e.name));
                if !e.type_params.is_empty() {
                    self.output.push_str("<");
                    self.output.push_str(&e.type_params.join(", "));
                    self.output.push_str(">");
                }
                self.output.push_str(" {\n");
                self.indent_level += 1;

                for variant in &e.variants {
                    self.push_indent();
                    self.output.push_str(&variant.name);
                    if !variant.fields.is_empty() {
                        self.output.push_str("(");
                        self.output.push_str(&variant.fields.join(", "));
                        self.output.push_str(")");
                    }
                    self.output.push_str(",\n");
                }

                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n\n");
            }
            ast::QuicheStmt::RustBlock(code) => {
                // Indent logic? RustBlock usually contains lines.
                for line in code.lines() {
                    self.push_indent();
                    self.output.push_str(line);
                    self.output.push_str("\n");
                }
            }
            ast::QuicheStmt::ImplDef(i) => {
                self.push_indent();
                self.output.push_str("impl ");
                if let Some(trait_name) = &i.trait_name {
                    self.output.push_str(trait_name);
                    self.output.push_str(" for ");
                }
                self.output.push_str(&i.target_type);
                self.output.push_str(" {\n");
                self.indent_level += 1;

                self.in_trait_or_impl = true;
                for stmt in i.body {
                    self.generate_stmt(stmt);
                }
                self.in_trait_or_impl = false;

                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n\n");
            }
            ast::QuicheStmt::TraitDef(t) => {
                self.push_indent();
                self.output.push_str(&format!("pub trait {} {{\n", t.name));
                self.indent_level += 1;
                self.in_trait_or_impl = true;
                for stmt in t.body {
                    self.generate_stmt(stmt);
                }
                self.in_trait_or_impl = false;
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n\n");
            }
            ast::QuicheStmt::Break => {
                self.push_indent();
                self.output.push_str(codegen_template!("break_stmt"));
                self.output.push_str("\n");
            }
            ast::QuicheStmt::Continue => {
                self.push_indent();
                self.output.push_str(codegen_template!("continue_stmt"));
                self.output.push_str("\n");
            }
            ast::QuicheStmt::Pass => {}
            ast::QuicheStmt::Match(m) => {
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
                    // Bind pattern variables if needed (generate_pattern does bind?)
                    // Actually generate_pattern emits Rust pattern syntax which binds natively.
                    // But we might need to register symbols in our Codegen scope for checking?
                    // My `generate_pattern` calls `add_symbol` for MatchAs!
                    // So yes.
                    for s in case.body {
                        self.generate_stmt(s);
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
            ast::QuicheStmt::Import(i) => {
                for alias in i.names {
                    self.push_indent();

                    let mod_path_original = &alias.name;
                    let mod_path = mod_path_original.replace(".", "::");
                    let is_external = mod_path.starts_with("std") // usually std is top level
                        || mod_path.starts_with("parsley")
                        || mod_path.starts_with("metaquiche_parser")
                        || mod_path == "glob"
                        || mod_path == "anyhow";

                    self.output.push_str("use ");

                    if mod_path.starts_with("rust::") {
                        // Strip "rust::" prefix for external crates
                        self.output.push_str(&mod_path[6..]);
                    } else if is_external {
                        self.output.push_str(&mod_path);
                    } else if !mod_path.starts_with("crate::") {
                        self.output.push_str("crate::");
                        self.output.push_str(&mod_path);
                    } else {
                        self.output.push_str(&mod_path);
                    }

                    if let Some(asname) = alias.asname {
                        self.output.push_str(&format!(" as {};\n", asname));
                    } else {
                        self.output.push_str(";\n");
                    }
                }
            }
            ast::QuicheStmt::ImportFrom(i) => {
                if let Some(module) = i.module {
                    // Handle relative imports logic (level > 0)?
                    // For now assume absolute or crate::
                    let mod_path = module.replace(".", "::");
                    let is_external = mod_path.starts_with("std::")
                        || mod_path == "std"
                        || mod_path.starts_with("core::")
                        || mod_path == "core"
                        || mod_path.starts_with("parsley")
                        || mod_path.starts_with("metaquiche_parser"); // Add other external crates here

                    if mod_path == "extern_defs"
                        || mod_path == "compiler"
                        || mod_path == "compiler.extern_defs"
                    {
                        self.import_kinds
                            .insert(mod_path.to_string(), "mod".to_string());
                    }

                    // Emit each import on a separate line (matches native compiler output)
                    for alias in i.names.iter() {
                        self.push_indent();
                        self.output.push_str("use ");

                        if mod_path.starts_with("rust::") {
                            self.output.push_str(&mod_path[6..]);
                        } else if is_external {
                            self.output.push_str(&mod_path);
                        } else if !mod_path.starts_with("crate::") {
                            self.output.push_str("crate::");
                            self.output.push_str(&mod_path);
                        } else {
                            self.output.push_str(&mod_path);
                        }
                        self.output.push_str("::");
                        self.output.push_str(&alias.name);
                        if let Some(asname) = &alias.asname {
                            self.output.push_str(" as ");
                            self.output.push_str(asname);
                        }
                        self.output.push_str(";\n");
                    }
                }
            }
            ast::QuicheStmt::Assert(a) => {
                self.push_indent();
                self.output.push_str("assert!(");
                self.generate_expr(*a.test);
                if let Some(msg) = a.msg {
                    self.output.push_str(", ");
                    self.generate_expr(*msg);
                }
                self.output.push_str(");\n");
            }
            ast::QuicheStmt::ClassDef(c) => {
                // Check for @extern
                if let Some((path, _no_generic)) = self.extract_extern_path(&c.decorator_list) {
                    self.output
                        .push_str(&format!("pub type {} = {};\n", c.name, path));
                    return;
                }

                // Default Class: Split into Struct (fields) and Impl (methods)
                let mut fields = Vec::new();
                let mut methods = Vec::new();

                for stmt in &c.body {
                    match stmt {
                        ast::QuicheStmt::AnnAssign(a) => {
                            if let ast::QuicheExpr::Name(n) = &*a.target {
                                // Field
                                // Use expr_to_string for type (simple compat)
                                let ty_str = self.map_type(&a.annotation);
                                fields.push((n.to_string(), ty_str));
                            }
                        }
                        ast::QuicheStmt::FunctionDef(f) => {
                            methods.push(f.clone());
                        }
                        ast::QuicheStmt::Pass => {}
                        _ => {
                            // Constants or other statements in class body?
                            // For now ignore or comment
                            self.output
                                .push_str(&format!("// Ignored in class body: {:?}\n", stmt));
                        }
                    }
                }

                // Emit Struct
                self.output.push_str("#[derive(Clone, Debug, Default)]\n"); // Auto-derive for convenience
                self.push_indent();
                // Private classes start with underscore
                let class_visibility = if c.name.starts_with('_') { "" } else { "pub " };
                self.output
                    .push_str(&format!("{}struct {} ", class_visibility, c.name));
                let params = if !c.type_params.is_empty() {
                    Some(c.type_params.join(", "))
                } else {
                    None
                };

                if let Some(ref p) = params {
                    self.output.push_str("<");
                    self.output.push_str(p);
                    self.output.push_str(">");
                }
                self.output.push_str(" {\n");
                self.indent_level += 1;
                for (name, ty) in &fields {
                    self.push_indent();
                    // Private fields start with underscore
                    let field_visibility = if name.starts_with('_') { "" } else { "pub " };
                    self.output
                        .push_str(&format!("{}{}: {},\n", field_visibility, name, ty));
                }
                self.indent_level -= 1;
                self.push_indent();
                self.output.push_str("}\n\n");

                // Emit Impl
                if !methods.is_empty() {
                    self.push_indent();
                    self.output.push_str("impl");
                    if let Some(ref p) = params {
                        self.output.push_str("<");
                        self.output.push_str(p);
                        self.output.push_str(">");
                    }
                    self.output.push_str(" ");
                    self.output.push_str(&c.name);
                    if let Some(ref p) = params {
                        self.output.push_str("<");
                        self.output.push_str(p);
                        self.output.push_str(">");
                    }
                    self.output.push_str(" {\n");
                    self.indent_level += 1;
                    for method in methods {
                        self.generate_function_def(method);
                    }
                    self.indent_level -= 1;
                    self.push_indent();
                    self.output.push_str("}\n\n");
                }
            }
        }
    }

    pub(crate) fn generate_pattern(&mut self, pat: &ast::Pattern) {
        match pat {
            ast::Pattern::MatchValue(v) => {
                self.generate_expr(*v.clone());
            }
            ast::Pattern::MatchSingleton(c) => {
                match c {
                    ast::Constant::NoneVal => self.output.push_str("None"),
                    ast::Constant::Bool(b) => self.output.push_str(&b.to_string()),
                    ast::Constant::Str(s) => self.output.push_str(&format!("\"{}\"", s)), // Match patterns need literals usually
                    ast::Constant::Int(i) => self.output.push_str(&i.to_string()),
                    _ => self.output.push_str("_"),
                }
            }
            ast::Pattern::MatchAs { pattern, name } => {
                if let Some(n) = name {
                    self.output.push_str(n);
                    self.add_symbol(n.to_string(), "/* inferred pattern bind */".to_string());
                } else {
                    self.output.push_str("_");
                }
                if let Some(p) = pattern {
                    self.output.push_str(" @ ");
                    self.generate_pattern(p);
                }
            }
            ast::Pattern::MatchClass(c) => {
                // Class path
                // Class path
                self.output.push_str(&self.expr_to_string(&c.cls));

                if !c.patterns.is_empty() {
                    self.output.push_str("(");
                    for (i, p) in c.patterns.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_pattern(p);
                    }
                    self.output.push_str(")");
                } else if !c.kwd_attrs.is_empty() {
                    self.output.push_str(" { ");
                    for (i, (attr, pat)) in
                        c.kwd_attrs.iter().zip(c.kwd_patterns.iter()).enumerate()
                    {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(attr);
                        self.output.push_str(": ");
                        self.generate_pattern(pat);
                    }
                    self.output.push_str(", .. }");
                }
            }
            ast::Pattern::MatchSequence(pats) => {
                // Tuple matching?
                self.output.push_str("(");
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_pattern(p);
                }
                self.output.push_str(")");
            }
            ast::Pattern::MatchMapping {
                keys: _,
                patterns: _,
                rest: _,
            } => {
                // Rust doesn't support easy map matching in syntax pattern.
                self.output.push_str("/* Map matching unsupported */ _");
            }
            ast::Pattern::MatchStar(_) => {
                self.output.push_str("..");
            }
            ast::Pattern::MatchOr(pats) => {
                for (i, p) in pats.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.generate_pattern(p);
                }
            }
        }
    }

    pub(crate) fn generate_function_def(&mut self, f: ast::FunctionDef) {
        self.push_indent();

        if let Some((path, _no_generic)) = self.extract_extern_path(&f.decorator_list) {
            self.output
                .push_str(&format!("pub use {} as {};\n", path, f.name));
            return;
        }

        let is_main_with_args = f.name.as_str() == "main" && !f.args.is_empty();

        if is_main_with_args || f.name.as_str() == "main" {
            self.output.push_str("pub fn main() {\n"); // Entry point must differ
        } else {
            // Auto-detect test functions
            if f.name.starts_with("test_") {
                self.output.push_str("#[test]\n");
                self.push_indent();
            }
            // Use 'fn' instead of 'pub fn' when inside trait/impl or when private (underscore prefix)
            let is_private = f.name.starts_with('_');
            if self.in_trait_or_impl || is_private {
                self.output.push_str(&format!("fn {}", f.name));
            } else {
                self.output.push_str(&format!("pub fn {}", f.name));
            }

            // Emit generic type parameters if present
            if !f.type_params.is_empty() {
                self.output.push_str("<");
                self.output.push_str(&f.type_params.join(", "));
                self.output.push_str(">");
            }

            self.output.push_str("(");

            for (i, arg) in f.args.iter().enumerate() {
                if i > 0 {
                    self.output.push_str(", ");
                }
                if arg.arg.as_str() == "self" {
                    // Use self_kind from AST (determined at parse time)
                    match f.self_kind {
                        ast::SelfKind::Ref(ast::Mutability::Mut) => {
                            self.output.push_str("&mut self")
                        }
                        ast::SelfKind::Ref(ast::Mutability::Not) => self.output.push_str("&self"),
                        ast::SelfKind::Value(ast::Mutability::Mut) => {
                            self.output.push_str("mut self")
                        }
                        ast::SelfKind::Value(ast::Mutability::Not) | ast::SelfKind::NoSelf => {
                            self.output.push_str("self")
                        }
                    }
                } else {
                    let type_ann = if let Some(annotation) = &arg.annotation {
                        self.map_type(annotation) // assuming map_type handles simple types
                    } else {
                        format!(
                            "compile_error!(\"Parameter '{}' requires a type annotation\")",
                            arg.arg
                        )
                    };
                    self.output.push_str(&format!("{}: {}", arg.arg, type_ann));
                }
            }

            self.output.push_str(")");

            // Return type
            if let Some(ref ret_expr) = f.returns {
                self.output
                    .push_str(&format!(" -> {}", self.map_type(ret_expr)));
            }

            // Check if this is a trait method signature (empty or pass-only body)
            let is_signature_only = self.in_trait_or_impl && {
                f.body.is_empty()
                    || (f.body.len() == 1 && matches!(f.body[0], ast::QuicheStmt::Pass))
            };

            if is_signature_only {
                self.output.push_str(";\n");
                return;
            }

            self.output.push_str(" {\n");
        }

        self.indent_level += 1;
        self.enter_scope(); // Start function scope

        // Register arguments
        if !is_main_with_args {
            for arg in f.args.iter() {
                if arg.arg.as_str() != "self" {
                    let type_ann = if let Some(annotation) = &arg.annotation {
                        self.map_type(annotation)
                    } else {
                        "unknown".to_string()
                    };
                    self.add_symbol(arg.arg.clone(), type_ann);
                    self.mark_defined(&arg.arg); // Assuming mark_defined takes &str
                }
            }
        }

        // Inject args extraction for main
        if is_main_with_args {
            self.push_indent();
            if let Some(param) = f.args.first() {
                self.output.push_str(&format!(
                    "let {}: Vec<String> = std::env::args().collect();\n",
                    param.arg
                ));
                self.add_symbol(param.arg.clone(), "Vec<String>".to_string());
                self.mark_defined(&param.arg);
            }
        }

        for (i, stmt) in f.body.clone().into_iter().enumerate() {
            if i == 0 {
                // Check for docstring
                if let ast::QuicheStmt::Expr(e) = &stmt {
                    if let ast::QuicheExpr::Constant(ast::Constant::Str(s)) = &**e {
                        self.push_indent();
                        self.output.push_str(codegen_template!("docstring_start"));
                        self.output.push_str(&s.replace("\"", "\\\""));
                        self.output.push_str(codegen_template!("docstring_end"));
                        continue;
                    }
                }
            }
            self.generate_stmt(stmt);
        }

        self.exit_scope(); // End function scope
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str(codegen_template!("function_def_end"));
    }

    fn extract_extern_path(&self, decorators: &[ast::QuicheExpr]) -> Option<(String, bool)> {
        for d in decorators {
            if let ast::QuicheExpr::Call {
                func,
                args: _,
                keywords,
            } = d
            {
                if let ast::QuicheExpr::Name(n) = &**func {
                    if n == "extern" {
                        let mut path = None;
                        let mut no_generic = false;
                        for kw in keywords {
                            if let Some(arg) = &kw.arg {
                                if arg == "path" {
                                    if let ast::QuicheExpr::Constant(ast::Constant::Str(s)) =
                                        &*kw.value
                                    {
                                        path = Some(s.clone());
                                    }
                                } else if arg == "no_generic" {
                                    // Check boolean literal
                                    if let ast::QuicheExpr::Constant(ast::Constant::Bool(b)) =
                                        &*kw.value
                                    {
                                        no_generic = *b;
                                    } else if let ast::QuicheExpr::Name(n) = &*kw.value {
                                        if n == "true" {
                                            no_generic = true;
                                        }
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
