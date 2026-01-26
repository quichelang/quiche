use rustpython_parser::ast;

pub struct Codegen {
    output: String,
    indent_level: usize,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
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

    fn generate_stmt(&mut self, stmt: ast::Stmt) {
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
            ast::Stmt::Assign(a) => {
                self.push_indent();
                // For Stage 1, we just use 'let' for all assignments
                // This will fail if reassigning, but work for simple scripts
                self.output.push_str("let ");
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
            _ => {
                self.push_indent();
                self.output.push_str("// Unimplemented statement\n");
            }
        }
    }

    fn generate_function_def(&mut self, f: ast::StmtFunctionDef) {
        self.push_indent();
        self.output.push_str(&format!("fn {}(", f.name));

        // Generate arguments
        for (i, arg_with_default) in f.args.args.iter().enumerate() {
            if i > 0 {
                self.output.push_str(", ");
            }
            let arg = &arg_with_default.def;
            let type_ann = if let Some(annotation) = &arg.annotation {
                self.map_type(annotation)
            } else {
                "/* untyped */".to_string()
            };
            self.output.push_str(&format!("{}: {}", arg.arg, type_ann));
        }

        self.output.push_str(")");

        // Return type
        if let Some(ret_expr) = f.returns {
            self.output
                .push_str(&format!(" -> {}", self.map_type(&ret_expr)));
        }

        self.output.push_str(" {\n");
        self.indent_level += 1;
        for stmt in f.body {
            self.generate_stmt(stmt);
        }
        self.indent_level -= 1;
        self.push_indent();
        self.output.push_str("}\n\n");
    }

    fn generate_expr(&mut self, expr: ast::Expr) {
        match expr {
            ast::Expr::BinOp(b) => {
                self.generate_expr(*b.left);
                let op_str = match b.op {
                    ast::Operator::Add => "+",
                    ast::Operator::Sub => "-",
                    ast::Operator::Mult => "*",
                    ast::Operator::Div => "/",
                    _ => "?",
                };
                self.output.push_str(&format!(" {} ", op_str));
                self.generate_expr(*b.right);
            }
            ast::Expr::Compare(c) => {
                self.generate_expr(*c.left);
                for (op, right) in c.ops.iter().zip(c.comparators.iter()) {
                    let op_str = match op {
                        ast::CmpOp::Eq => "==",
                        ast::CmpOp::NotEq => "!=",
                        ast::CmpOp::Lt => "<",
                        ast::CmpOp::LtE => "<=",
                        ast::CmpOp::Gt => ">",
                        ast::CmpOp::GtE => ">=",
                        _ => "?",
                    };
                    self.output.push_str(&format!(" {} ", op_str));
                    self.generate_expr(right.clone());
                }
            }
            ast::Expr::Call(c) => {
                let func_name = if let ast::Expr::Name(n) = &*c.func {
                    n.id.as_str()
                } else {
                    ""
                };

                if func_name == "print" {
                    self.output.push_str("println!(\"{:?}\", ");
                } else {
                    self.generate_expr(*c.func);
                    self.output.push_str("(");
                }

                for (i, arg) in c.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg.clone());
                }
                self.output.push_str(")");
            }
            ast::Expr::Name(n) => {
                self.output.push_str(&n.id);
            }
            ast::Expr::Constant(c) => match c.value {
                ast::Constant::Int(i) => self.output.push_str(&i.to_string()),
                ast::Constant::Float(f) => self.output.push_str(&f.to_string()),
                ast::Constant::Str(s) => self.output.push_str(&format!("\"{}\"", s)),
                ast::Constant::Bool(b) => self.output.push_str(if b { "true" } else { "false" }),
                _ => self.output.push_str("/* unhandled constant */"),
            },
            _ => self.output.push_str("/* unhandled expression */"),
        }
    }

    fn map_type(&self, expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::Name(n) => match n.id.as_str() {
                "i8" => ""
                "i32" => "i32".to_string(),
                "i64" => "i64".to_string(),
                "u32" => "u32".to_string(),
                "u64" => "u64".to_string(),
                "f32" => "f32".to_string(),
                "f64" => "f64".to_string(),
                "bool" => "bool".to_string(),
                "str" => "String".to_string(),
                _ => n.id.to_string(),
            },
            _ => "/* complex type */".to_string(),
        }
    }

    fn push_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}
