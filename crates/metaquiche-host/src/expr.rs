use crate::codegen_template;
use crate::Codegen;
use quiche_parser::ast;

impl Codegen {
    pub(crate) fn generate_expr(&mut self, expr: ast::QuicheExpr) {
        match expr {
            ast::QuicheExpr::BinOp { left, op, right } => {
                // Check if this is string concatenation
                if op == ast::Operator::Add
                    && (self.is_string_expr(&left) || self.is_string_expr(&right))
                {
                    // Flatten the chain and emit strcat!
                    let mut parts = Vec::new();
                    self.flatten_add_chain(&left, &mut parts);
                    self.flatten_add_chain(&right, &mut parts);

                    self.output.push_str("crate::quiche::strcat!(");
                    for (i, part) in parts.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(part);
                    }
                    self.output.push_str(")");
                } else {
                    self.generate_expr(*left);
                    let op_str = match op {
                        ast::Operator::Add => "+",
                        ast::Operator::Sub => "-",
                        ast::Operator::Mult => "*",
                        ast::Operator::Div => "/",
                        ast::Operator::Mod => "%",
                        ast::Operator::Pow => ".pow", // handled specially below
                        ast::Operator::BitAnd => "&",
                        ast::Operator::BitOr => "|",
                        ast::Operator::BitXor => "^",
                        ast::Operator::LShift => "<<",
                        ast::Operator::RShift => ">>",
                        _ => "?",
                    };
                    if op == ast::Operator::Pow {
                        // TODO: Implement pow via method call or specialized logic if needed
                        self.output.push_str(".pow(");
                        self.generate_expr(*right);
                        self.output.push_str(")");
                    } else {
                        self.output.push_str(&format!(" {} ", op_str));
                        self.generate_expr(*right);
                    }
                }
            }
            ast::QuicheExpr::BoolOp { op, values } => {
                let op_str = match op {
                    ast::BoolOperator::And => "&&",
                    ast::BoolOperator::Or => "||",
                };
                for (i, value) in values.into_iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(&format!(" {} ", op_str));
                    }
                    self.output.push_str("(");
                    self.generate_expr(value);
                    self.output.push_str(")");
                }
            }
            ast::QuicheExpr::UnaryOp { op, operand } => {
                let op_str = match op {
                    ast::UnaryOperator::Invert => "!",
                    ast::UnaryOperator::Not => "!",
                    ast::UnaryOperator::UAdd => "+",
                    ast::UnaryOperator::USub => "-",
                };
                self.output.push_str(op_str);
                self.generate_expr(*operand);
            }
            ast::QuicheExpr::Compare {
                left,
                ops,
                comparators,
            } => {
                self.generate_expr(*left);
                for (op, right) in ops.iter().zip(comparators.into_iter()) {
                    let op_str = match op {
                        ast::CmpOperator::Eq => "==",
                        ast::CmpOperator::NotEq => "!=",
                        ast::CmpOperator::Lt => "<",
                        ast::CmpOperator::LtE => "<=",
                        ast::CmpOperator::Gt => ">",
                        ast::CmpOperator::GtE => ">=",
                        // Is/In not directly map to operator in Rust usually, need check! or method
                        _ => "?",
                    };
                    self.output.push_str(&format!(" {} ", op_str));
                    self.generate_expr(right);
                }
            }
            ast::QuicheExpr::IfExp { test, body, orelse } => {
                self.output.push_str("if ");
                self.generate_expr(*test);
                self.output.push_str(" { ");
                self.generate_expr(*body);
                self.output.push_str(" } else { ");
                self.generate_expr(*orelse);
                self.output.push_str(" }");
            }
            ast::QuicheExpr::Call {
                func,
                args,
                keywords,
            } => {
                {
                    use std::io::Write;
                    if let Ok(mut f) = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("/tmp/quiche_debug.txt")
                    {
                        writeln!(f, "Func: {:?}", func).ok();
                    }
                }
                // 0. Special Case: Direct Lambda Calls
                if let ast::QuicheExpr::Lambda { .. } = *func {
                    self.generate_expr(*func);
                    self.output.push_str("(");
                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg);
                    }
                    self.output.push_str(")");
                    return;
                }

                // 0b. Constructors for List/Dict
                if let ast::QuicheExpr::Attribute { value, attr } = &*func {
                    if attr == "new" {
                        if let ast::QuicheExpr::Name(n) = &**value {
                            match n.as_str() {
                                "List" | "Vec" => {
                                    self.output.push_str("Vec::new()");
                                    return;
                                }
                                "Dict" | "HashMap" => {
                                    self.output.push_str("std::collections::HashMap::new()");
                                    return;
                                }
                                _ => {}
                            }
                        }
                        // Handle Subscript: List[int].new() -> Vec::<int>::new()
                        if let ast::QuicheExpr::Subscript {
                            value: sub_val,
                            slice,
                        } = &**value
                        {
                            let base = self.expr_to_string(sub_val);
                            if base == "List" || base == "Vec" {
                                let inner = self.map_type_expr(slice); // Need map_type_expr
                                self.output.push_str(&format!("Vec::<{}>::new()", inner));
                                return;
                            }
                            if base == "Dict" || base == "HashMap" {
                                // Simplified tuple handling for now
                                self.output.push_str("std::collections::HashMap::new()");
                                return;
                            }
                        }
                    }
                }

                // 2. Strict 1:1 Method Calling
                if let ast::QuicheExpr::Attribute { value, attr } = *func {
                    self.generate_expr(*value.clone());

                    let base_str = self.expr_to_string(&*value);
                    let sep = if self.is_type_or_mod(&base_str) {
                        "::"
                    } else {
                        "."
                    };
                    self.output.push_str(sep);
                    self.output.push_str(&attr);
                    self.output.push_str("(");
                    let args_empty = args.is_empty();

                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg);
                    }

                    if !args_empty && !keywords.is_empty() {
                        self.output.push_str(", ");
                    }
                    for (i, kw) in keywords.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(*kw.value);
                    }
                    self.output.push_str(")");
                    return;
                }

                // 3. Builtins & Generic Calls
                let func_name = if let ast::QuicheExpr::Name(n) = &*func {
                    n.clone()
                } else {
                    "".to_string()
                };

                // Intrinsic/Macro Handling
                if func_name == "print" || func_name == "println" {
                    // Generate format string with correct number of {:?} placeholders (Debug fmt)
                    let fmt = std::iter::repeat("{:?}")
                        .take(args.len())
                        .collect::<Vec<_>>()
                        .join(" ");
                    self.output.push_str(&format!("println!(\"{}\", ", fmt));
                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg);
                    }
                    self.output.push_str(")");
                    return;
                }

                // Handle exit() specially to emit std::process::exit() directly
                if func_name == "exit" {
                    self.output.push_str("std::process::exit(");
                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg);
                    }
                    self.output.push_str(")");
                    return;
                }

                // Default Function Call
                let is_helper = ["deref", "as_ref", "ref", "mutref", "as_mut", "strcat"]
                    .contains(&func_name.as_str());
                // Translate to actual macro names
                let macro_name = match func_name.as_str() {
                    "ref" | "as_ref" => "qref",
                    "mutref" | "as_mut" => "mutref",
                    _ => &func_name,
                };
                self.output.push_str(macro_name);
                if is_helper {
                    self.output.push_str("!");
                }
                self.output.push_str("(");
                let args_empty = args.is_empty();
                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg);
                }
                if !args_empty && !keywords.is_empty() {
                    self.output.push_str(", ");
                }
                for (i, kw) in keywords.into_iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(*kw.value);
                }
                self.output.push_str(")");
            }
            ast::QuicheExpr::Attribute { value, attr } => {
                // Determine separator
                let base_str = self.expr_to_string(&*value);
                let _is_constr = if let Some(cls) = self.current_class.clone() {
                    {
                        use std::io::Write;
                        if let Ok(mut f) = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                        {
                            writeln!(f, "Checking Constr: cls={} constr={}", cls, attr).ok();
                        }
                    }
                    cls == attr.to_string()
                } else {
                    false
                };
                let sep = if self.is_type_or_mod(&base_str) {
                    "::"
                } else {
                    "."
                };

                // Access: val.attr -> val.attr OR val::attr
                self.generate_expr(*value);
                self.output.push_str(sep);
                self.output.push_str(&attr);
            }
            ast::QuicheExpr::Name(n) => {
                self.output.push_str(&n);
            }
            ast::QuicheExpr::Constant(c) => match c {
                ast::Constant::NoneVal => self.output.push_str(codegen_template!("none_literal")),
                ast::Constant::Bool(b) => {
                    if b {
                        self.output.push_str(codegen_template!("true_literal"));
                    } else {
                        self.output.push_str(codegen_template!("false_literal"));
                    }
                }
                ast::Constant::Str(s) => self.output.push_str(&format!("String::from({:?})", s)),
                ast::Constant::Int(i) => self.output.push_str(&i.to_string()),
                ast::Constant::Float(f) => {
                    let s = f.to_string();
                    if s.contains('.') {
                        self.output.push_str(&s);
                    } else {
                        self.output.push_str(&format!("{}.0", s));
                    }
                }
                _ => self.output.push_str("()"), // Ellipsis -> unit type ()
            },
            ast::QuicheExpr::List(l) => {
                // Emit compiler warning about Vec vs Python list semantics
                eprintln!("Warning: List literal [] creates a Rust Vec, which has different semantics than Python's list");
                self.output.push_str("vec![");
                for (i, e) in l.into_iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(e);
                }
                self.output.push_str("]");
            }
            ast::QuicheExpr::Subscript { value, slice } => {
                self.generate_expr(*value);
                self.output.push_str("[");
                self.generate_expr(*slice);
                self.output.push_str("].clone()");
            }
            ast::QuicheExpr::Lambda { args, body } => {
                self.output.push_str("(|");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(arg);
                }
                self.output.push_str("| ");
                self.generate_expr(*body);
                self.output.push_str(")");
            }
            _ => {
                self.output
                    .push_str(&format!("/* unhandled expr: {:?} */", expr));
            }
        }
    }
}
