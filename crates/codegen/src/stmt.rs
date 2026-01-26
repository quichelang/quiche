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
                self.output.push_str(" in ");
                self.generate_expr(*f.iter);
                self.output.push_str(" {\n");
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
                // We assume explicit 'AnnAssign' for declarations
                for (i, target) in a.targets.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" = ");
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
            ast::Stmt::ClassDef(c) => {
                self.push_indent();
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
            _ => {
                self.push_indent();
                self.output.push_str("// Unimplemented statement\n");
            }
        }
    }

    pub(crate) fn generate_function_def(&mut self, f: ast::StmtFunctionDef) {
        self.push_indent();

        let is_main_with_args = f.name.as_str() == "main" && !f.args.args.is_empty();

        if is_main_with_args {
            self.output.push_str("fn main() {\n");
        } else {
            self.output.push_str(&format!("fn {}(", f.name));

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
}
