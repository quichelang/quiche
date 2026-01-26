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
                // Check for method call: obj.method(args)
                if let ast::Expr::Attribute(attr) = &*c.func {
                    let method_name = attr.attr.as_str();

                    // Check for list method aliasing
                    if let Some((rust_method, _)) = crate::list::map_list_method(method_name) {
                        self.generate_expr(*attr.value.clone());
                        self.output.push_str(".");
                        self.output.push_str(rust_method);
                        self.output.push_str("(");
                        for (i, arg) in c.args.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            self.generate_expr(arg.clone());
                        }
                        self.output.push_str(")");
                        return;
                    }

                    // Check for dict method aliasing
                    if let Some((rust_method, key_needs_ref)) =
                        crate::dict::map_dict_method(method_name)
                    {
                        self.generate_expr(*attr.value.clone());
                        self.output.push_str(".");
                        self.output.push_str(rust_method);
                        self.output.push_str("(");
                        for (i, arg) in c.args.iter().enumerate() {
                            if i > 0 {
                                self.output.push_str(", ");
                            }
                            if i == 0 && key_needs_ref {
                                self.output.push_str("&");
                            }
                            self.generate_expr(arg.clone());
                        }
                        self.output.push_str(")");
                        if method_name == "get" {
                            self.output.push_str(".cloned()");
                        }
                        return;
                    }
                }

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
                } else if func_name == "len" {
                    // len(x) -> x.len()
                    if let Some(arg) = c.args.first() {
                        self.generate_expr(arg.clone());
                        self.output.push_str(".len()");
                    }
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
            ast::Expr::Attribute(a) => {
                let base_str = self.expr_to_string(&a.value);
                self.generate_expr(*a.value.clone());
                // Heuristic: Capitalized base -> Type/Enum static access (::)
                // Lowercase base -> Instance access (.)
                let sep = if base_str
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    "::"
                } else {
                    "."
                };
                self.output.push_str(sep);
                self.output.push_str(&a.attr);
            }
            ast::Expr::Name(n) => {
                self.output.push_str(&n.id);
            }
            ast::Expr::Constant(c) => match c.value {
                ast::Constant::None => self.output.push_str("None"),
                ast::Constant::Int(i) => self.output.push_str(&i.to_string()),
                ast::Constant::Float(f) => {
                    let s = f.to_string();
                    if s.contains('.') {
                        self.output.push_str(&s);
                    } else {
                        self.output.push_str(&format!("{}.0", s));
                    }
                }
                ast::Constant::Str(s) => self.output.push_str(&format!("String::from(\"{}\")", s)),
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
            ast::Expr::Dict(d) => {
                self.output.push_str("std::collections::HashMap::from([");
                for (i, (k, v)) in d.keys.iter().zip(d.values.iter()).enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str("(");
                    if let Some(key) = k {
                        self.generate_expr(key.clone());
                    } else {
                        self.output.push_str("/* **kwargs not supported */");
                    }
                    self.output.push_str(", ");
                    self.generate_expr(v.clone());
                    self.output.push_str(")");
                }
                self.output.push_str("])");
            }
            ast::Expr::Subscript(s) => {
                // Check for tuple/map access via symbol table
                let expr_type = self.get_expr_type(&s.value);
                let is_tuple = expr_type
                    .as_ref()
                    .map(|t| t.starts_with("("))
                    .unwrap_or(false);
                let is_map = expr_type
                    .as_ref()
                    .map(|t| t.contains("HashMap") || t.contains("Dict"))
                    .unwrap_or(false);

                if is_tuple {
                    if let ast::Expr::Constant(c) = &*s.slice {
                        if let ast::Constant::Int(idx) = &c.value {
                            self.generate_expr(*s.value.clone());
                            self.output.push_str(&format!(".{}", idx));
                            return;
                        }
                    }
                }

                if is_map {
                    // Map access: d[&key]
                    self.generate_expr(*s.value.clone());
                    self.output.push_str("[&");
                    self.generate_expr(*s.slice.clone());
                    self.output.push_str("]");
                    return;
                }

                // Check for negative indexing first
                if let ast::Expr::UnaryOp(u) = &*s.slice {
                    if matches!(u.op, ast::UnaryOp::USub) {
                        // x[-n] -> x[x.len() - n]
                        let value_str = self.expr_to_string(&s.value);
                        self.generate_expr(*s.value.clone());
                        self.output.push_str("[");
                        self.output.push_str(&value_str);
                        self.output.push_str(".len() - ");
                        self.generate_expr(*u.operand.clone());
                        self.output.push_str("]");
                        return;
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
            ast::Expr::Lambda(l) => {
                self.output.push_str("(|");
                for (i, arg) in l.args.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(&arg.def.arg);
                }
                self.output.push_str("| ");
                self.generate_expr(*l.body);
                self.output.push_str(")");
            }
            ast::Expr::JoinedStr(j) => {
                // f-string: f"Hello {name}" -> format!("Hello {}", name)
                self.output.push_str("format!(\"");
                let mut args: Vec<ast::Expr> = Vec::new();

                for value in &j.values {
                    match value {
                        ast::Expr::Constant(c) => {
                            if let ast::Constant::Str(s) = &c.value {
                                self.output.push_str(s);
                            }
                        }
                        ast::Expr::FormattedValue(f) => {
                            self.output.push_str("{}");
                            args.push(*f.value.clone());
                        }
                        _ => {}
                    }
                }

                self.output.push_str("\"");
                for arg in args {
                    self.output.push_str(", ");
                    self.generate_expr(arg);
                }
                self.output.push_str(")");
            }
            _ => self.output.push_str("/* unhandled expression */"),
        }
    }
}
