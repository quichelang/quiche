use crate::Codegen;
use quiche_parser::ast;

impl Codegen {
    pub(crate) fn self_field_type(&self, attr: &str, value: &ast::QuicheExpr) -> Option<String> {
        if let ast::QuicheExpr::Name(n) = value {
            if n == "self" {
                return self.get_self_field_type(attr);
            }
        }
        None
    }

    pub(crate) fn generate_expr(&mut self, expr: ast::QuicheExpr) {
        if let ast::QuicheExpr::Call {
            func,
            args,
            keywords,
        } = &expr
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open("/tmp/quiche_debug.txt")
                .unwrap();
            writeln!(f, "GEN CAL: args={} kws={}", args.len(), keywords.len()).ok();
        }
        match expr {
            ast::QuicheExpr::BinOp { left, op, right } => {
                self.generate_expr(*left);
                let op_str = match op {
                    ast::Operator::Add => "+",
                    ast::Operator::Sub => "-",
                    ast::Operator::Mult => "*",
                    ast::Operator::Div => "/",
                    ast::Operator::Mod => "%",
                    ast::Operator::Pow => "/* pow */", // standard Rust doesn't use operator for pow
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
                    let mut f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("/tmp/quiche_debug.txt")
                        .unwrap();
                    writeln!(f, "Func: {:?}", func).ok();
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
                    let skip_check = [
                        "as_str",
                        "to_string",
                        "clone",
                        "into_syntax",
                        "into_iter",
                        "as_ref",
                        "enter_var_scope",
                        "exit_var_scope",
                        "get_root_name",
                        "is_type_or_mod",
                        "is_var_defined",
                        "mark_var_defined",
                        "define_var",
                        "expr_contains_name",
                        "generate_pattern",
                        "emit",
                        "push",
                    ]
                    .contains(&attr.as_str());
                    if !skip_check {
                        self.output.push_str("crate::quiche::check!(");
                    }
                    self.generate_expr(*value.clone());

                    let base_str = self.expr_to_string(&*value);
                    let sep = if self.is_type_or_mod(&base_str) {
                        "::"
                    } else {
                        "."
                    };
                    {
                        use std::io::Write;
                        let mut f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                            .unwrap();
                        writeln!(
                            f,
                            "Expr Call: base='{}' sep='{}' attr='{}'",
                            base_str, sep, attr
                        )
                        .ok();
                    }
                    if sep == "::" {
                        self.output.push_str("::");
                    } else {
                        self.output.push_str(".");
                    }

                    self.output.push_str(&attr);
                    self.output.push_str("(");
                    let args = args; // Ensure ownership
                    let args_len = args.len();
                    let args_empty = args.is_empty();

                    {
                        use std::io::Write;
                        let mut f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                            .unwrap();
                        writeln!(f, "Pre-Args Loop").ok();
                    }

                    for (i, arg) in args.into_iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg);
                    }

                    {
                        use std::io::Write;
                        let mut f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                            .unwrap();
                        writeln!(f, "Post-Args Loop").ok();
                    }

                    if !args_empty && !keywords.is_empty() {
                        self.output.push_str(", ");
                    }
                    {
                        use std::io::Write;
                        let mut f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                            .unwrap();
                        writeln!(f, "AttrCall KW: args={} kws={}", args_len, keywords.len()).ok();
                    }
                    for (i, kw) in keywords.into_iter().enumerate() {
                        {
                            use std::io::Write;
                            let mut f = std::fs::OpenOptions::new()
                                .append(true)
                                .create(true)
                                .open("/tmp/quiche_debug.txt")
                                .unwrap();
                            writeln!(f, "Loop KW {}", i).ok();
                        }
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(*kw.value);
                    }
                    self.output.push_str(")");

                    {
                        use std::io::Write;
                        let mut f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                            .unwrap();
                        writeln!(f, "AttrCall EMIT: {}", self.output).ok();
                    }

                    if !skip_check {
                        self.output.push_str(")");
                    }
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
                    self.output.push_str("println!(\"{}\", ");
                    // join args with comma
                    if let Some(first) = args.first() {
                        // logic to join? standard print in python takes multiple args and joins with space.
                        // Rust println! expected format string.
                        // Simplification: only support 1 arg or multiple printed consecutively?
                        // "println!(\"{}\", arg)" supports 1.
                        // For multiple: println!("{} {}", arg1, arg2)
                        // Let's implement multi-arg print support:
                        let fmt = std::iter::repeat("{}")
                            .take(args.len())
                            .collect::<Vec<_>>()
                            .join(" ");
                        // Override previous push
                        // Actually this is hard with current stream writing.
                        // Just loop.
                    }
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
                let is_helper = ["deref", "as_ref"].contains(&func_name.as_str());
                if !is_helper {
                    self.output.push_str("crate::quiche::check!(");
                }
                self.output.push_str(&func_name);
                if is_helper {
                    self.output.push_str("!");
                }
                self.output.push_str("(");
                let args_empty = args.is_empty();
                let args_len = args.len();
                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(arg);
                }
                if !args_empty && !keywords.is_empty() {
                    self.output.push_str(", ");
                }
                {
                    use std::io::Write;
                    let mut f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("/tmp/quiche_debug.txt")
                        .unwrap();
                    writeln!(f, "Call KW: args={} kws={}", args_len, keywords.len()).ok();
                }
                for (i, kw) in keywords.into_iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.generate_expr(*kw.value);
                }
                self.output.push_str(")");
                if !is_helper {
                    self.output.push_str(")");
                }
            }
            ast::QuicheExpr::Attribute { value, attr } => {
                // Determine separator
                let base_str = self.expr_to_string(&*value);
                let is_constr = if let Some(cls) = self.current_class.clone() {
                    {
                        use std::io::Write;
                        let mut f = std::fs::OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("/tmp/quiche_debug.txt")
                            .unwrap();
                        writeln!(f, "Checking Constr: cls={} constr={}", cls, attr).ok();
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
                ast::Constant::None => self.output.push_str("None"),
                ast::Constant::Bool(b) => self.output.push_str(&b.to_string()),
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
                _ => self.output.push_str("/* unknown const */"),
            },
            ast::QuicheExpr::List(l) => {
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
            _ => {
                self.output
                    .push_str(&format!("/* unhandled expr: {:?} */", expr));
            }
        }
    }
}
