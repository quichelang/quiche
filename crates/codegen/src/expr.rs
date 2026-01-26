use crate::Codegen;
use rustpython_parser::ast;

impl Codegen {
    pub(crate) fn generate_expr(&mut self, expr: ast::Expr) {
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
            ast::Expr::IfExp(i) => {
                self.output.push_str("if ");
                self.generate_expr(*i.test);
                self.output.push_str(" { ");
                self.generate_expr(*i.body);
                self.output.push_str(" } else { ");
                self.generate_expr(*i.orelse);
                self.output.push_str(" }");
            }
            ast::Expr::Call(c) => {
                let func_name = if let ast::Expr::Name(n) = &*c.func {
                    n.id.as_str()
                } else {
                    ""
                };

                if func_name == "print" {
                    self.output.push_str("println!(\"{:?}\", ");
                    for (i, arg) in c.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                } else if !c.keywords.is_empty() {
                    // Assume Struct Init: Name(key=val) -> Name { key: val }
                    self.generate_expr(*c.func);
                    self.output.push_str(" { ");
                    for (i, kw) in c.keywords.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        if let Some(arg) = &kw.arg {
                            self.output.push_str(arg);
                            self.output.push_str(": ");
                            self.generate_expr(kw.value.clone());
                        }
                    }
                    self.output.push_str(" }");
                } else {
                    // Regular Function Call
                    self.generate_expr(*c.func);
                    self.output.push_str("(");
                    for (i, arg) in c.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                }
            }
            ast::Expr::Name(n) => {
                self.output.push_str(&n.id);
            }
            ast::Expr::Constant(c) => match c.value {
                ast::Constant::Int(i) => self.output.push_str(&i.to_string()),
                ast::Constant::Float(f) => {
                    let s = f.to_string();
                    if s.contains('.') {
                        self.output.push_str(&s);
                    } else {
                        self.output.push_str(&format!("{}.0", s));
                    }
                }
                ast::Constant::Str(s) => self.output.push_str(&format!("\"{}\"", s)),
                ast::Constant::Bool(b) => self.output.push_str(if b { "true" } else { "false" }),
                _ => self.output.push_str("/* unhandled constant */"),
            },
            ast::Expr::List(l) => {
                self.output.push_str("vec![");
                for (i, elt) in l.elts.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(elt.clone());
                }
                self.output.push_str("]");
            }
            ast::Expr::Subscript(s) => {
                // Check for tuple access via symbol table
                let is_tuple_access = if let Some(ty) = self.get_expr_type(&s.value) {
                    ty.starts_with("(")
                } else {
                    false
                };

                if is_tuple_access {
                    if let ast::Expr::Constant(c) = &*s.slice {
                        if let ast::Constant::Int(idx) = &c.value {
                            self.generate_expr(*s.value.clone());
                            self.output.push_str(&format!(".{}", idx));
                            return;
                        }
                    }
                }

                // Fallback to Vec/Index access: val[slice]
                self.generate_expr(*s.value.clone());
                self.output.push_str("[");
                self.generate_expr(*s.slice.clone());
                self.output.push_str("]");
            }
            ast::Expr::Tuple(t) => {
                self.output.push_str("(");
                for (i, elt) in t.elts.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(elt.clone());
                }
                self.output.push_str(")");
            }
            _ => self.output.push_str("/* unhandled expression */"),
        }
    }
}
