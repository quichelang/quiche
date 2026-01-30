use crate::Codegen;
use ruff_python_ast as ast;

impl Codegen {
    pub(crate) fn self_field_type(&self, attr: &ast::ExprAttribute) -> Option<String> {
        if let ast::Expr::Name(n) = &*attr.value {
            if n.id.as_str() == "self" {
                return self.get_self_field_type(attr.attr.as_str());
            }
        }
        None
    }

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
            ast::Expr::BoolOp(b) => {
                let op_str = match b.op {
                    ast::BoolOp::And => "&&",
                    ast::BoolOp::Or => "||",
                };
                for (i, value) in b.values.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(&format!(" {} ", op_str));
                    }
                    self.output.push_str("(");
                    self.generate_expr(value.clone());
                    self.output.push_str(")");
                }
            }
            ast::Expr::UnaryOp(u) => {
                let op_str = match u.op {
                    ast::UnaryOp::Invert => "!",
                    ast::UnaryOp::Not => "!",
                    ast::UnaryOp::UAdd => "+",
                    ast::UnaryOp::USub => "-",
                };
                self.output.push_str(op_str);
                self.generate_expr(*u.operand);
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
            ast::Expr::If(i) => {
                self.output.push_str("if ");
                self.generate_expr(*i.test);
                self.output.push_str(" { ");
                self.generate_expr(*i.body);
                self.output.push_str(" } else { ");
                self.generate_expr(*i.orelse);
                self.output.push_str(" }");
            }
            ast::Expr::Call(c) => {
                // 0. Special Case: Direct Lambda Calls
                if let ast::Expr::Lambda(_) = &*c.func {
                    self.generate_expr(*c.func);
                    self.output.push_str("(");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                    return;
                }

                // 0b. Constructors for List/Dict (Rc<RefCell<...>>)
                if let ast::Expr::Attribute(attr) = &*c.func {
                    if attr.attr.as_str() == "new" {
                        match &*attr.value {
                            ast::Expr::Name(n) => match n.id.as_str() {
                                "List" | "Vec" => {
                                    self.output.push_str("Vec::new()");
                                    return;
                                }
                                "Dict" | "HashMap" => {
                                    self.output.push_str("std::collections::HashMap::new()");
                                    return;
                                }
                                _ => {}
                            },
                            ast::Expr::Subscript(s) => {
                                let base = self.expr_to_string(&s.value);
                                if base == "List" || base == "Vec" {
                                    let inner = self.map_type(&s.slice);
                                    self.output.push_str("Vec::<");
                                    self.output.push_str(&inner);
                                    self.output.push_str(">::new()");
                                    return;
                                }
                                if base == "Dict" || base == "HashMap" {
                                    if let ast::Expr::Tuple(t) = &*s.slice {
                                        if t.elts.len() == 2 {
                                            let k = self.map_type(&t.elts[0]);
                                            let v = self.map_type(&t.elts[1]);
                                            self.output.push_str("std::collections::HashMap::<");
                                            self.output.push_str(&k);
                                            self.output.push_str(", ");
                                            self.output.push_str(&v);
                                            self.output.push_str(">::new()");
                                            return;
                                        }
                                    }
                                    self.output.push_str("std::collections::HashMap::new()");
                                    return;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                // 1. Check foreign symbols (rust.* imports)
                let foreign_name = if let ast::Expr::Name(n) = &*c.func {
                    if self.foreign_symbols.contains(n.id.as_str()) {
                        Some(n.id.to_string())
                    } else {
                        None
                    }
                } else if let ast::Expr::Attribute(attr) = &*c.func {
                    let (base_str, base_is_type) = if let ast::Expr::Subscript(s) = &*attr.value {
                        let base_name = self.expr_to_string(&s.value);
                        if self.is_type_or_mod(&base_name) {
                            (self.map_type_expr(&attr.value), true)
                        } else {
                            (self.expr_to_string(&attr.value), false)
                        }
                    } else {
                        let base = self.expr_to_string(&attr.value);
                        let is_type = self.is_type_or_mod(&base);
                        (base, is_type)
                    };
                    if self.foreign_symbols.contains(&base_str) || base_is_type {
                        Some(format!("{}::{}", base_str, attr.attr))
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(path) = foreign_name {
                    self.output.push_str("crate::quiche::check!(");
                    self.output.push_str(&path);
                    self.output.push_str("(");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str("))");
                    return;
                }

                // 2. Strict 1:1 Method Calling (No heuristics, no renaming)
                if let ast::Expr::Attribute(attr) = &*c.func {
                    let method_name = attr.attr.as_str();

                    self.output.push_str("crate::quiche::check!(");

                    // Check if it's a mutating method that needs make_mut (we might still need this for Rc<Vec>)
                    // BUT for strict 1:1, we ideally want the USER to handle mutation or we provide a smarter check!
                    // For now, let's keep the make_mut logic if we know the method name implies mutation on known types,
                    // OR we can strip that too and require explicit Rc::make_mut handling in source.
                    // Given the goal is "dumb compiler", let's strip the make_mut magic too?
                    // User said "use vectors ... as if we were using it in Rust".
                    // In Rust, if you have Rc<Vec>, you can't call push directly. You need &mut.
                    // So we probably SHOULD keep the make_mut injection if we want "Python-like" ease for Rc types,
                    // OR we force the user to do `Rc::make_mut(&mut x).push(y)`.
                    // The user said: "Delegate all methods to the underlying struct object... as much as possible, we will try to keep a 1:1 mapping."
                    // This implies we should propagate the method call directly.
                    // However, we are wrapping in `check!`.

                    self.generate_expr(*attr.value.clone());
                    self.output.push_str(".");
                    self.output.push_str(method_name); // 1:1 Name Mapping
                    self.output.push_str("(");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str("))");
                    return;
                }

                // 3. Builtins & Generic Calls (Wrap ALL, except specific builtins)
                let func_name = if let ast::Expr::Name(n) = &*c.func {
                    n.id.as_str()
                } else {
                    ""
                };

                // Handle as_ref and deref without check! wrapper
                if func_name == "as_ref" {
                    if let Some(arg) = c.arguments.args.first() {
                        self.output.push_str("&");
                        self.generate_expr(arg.clone());
                    }
                    return;
                }

                if func_name == "deref" {
                    if let Some(arg) = c.arguments.args.first() {
                        self.output.push_str("*");
                        self.generate_expr(arg.clone());
                    }
                    return;
                }

                if func_name == "parse_program" {
                    self.output.push_str("parse_program(");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                    return;
                }

                let is_intrinsic = func_name == "as_ref"
                    || func_name == "as_mut"
                    || func_name == "deref"
                    || func_name == "print"
                    || func_name == "print_str"
                    || func_name == "assert"
                    || func_name == "assert_eq"
                    || func_name == "assert_str_eq"
                    || func_name == "assert_true"
                    || func_name == "range"
                    || func_name == "exit";

                let is_struct_init = !c.arguments.keywords.is_empty();

                if !is_intrinsic && !is_struct_init {
                    self.output.push_str("crate::quiche::check!(");
                }

                if func_name == "print" {
                    self.output.push_str("println!(\"{}\", ");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                } else if func_name == "print_str" {
                    self.output.push_str("println!(\"{}\", ");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                } else if func_name == "assert" {
                    self.output.push_str("assert!(");
                    if let Some(arg) = c.arguments.args.first() {
                        self.generate_expr(arg.clone());
                    }
                    if c.arguments.args.len() > 1 {
                        self.output.push_str(", \"{}\", ");
                        self.generate_expr(c.arguments.args[1].clone());
                    }
                    self.output.push_str(")");
                } else if func_name == "assert_eq" || func_name == "assert_str_eq" {
                    self.output.push_str("assert_eq!(");
                    if c.arguments.args.len() >= 2 {
                        self.generate_expr(c.arguments.args[0].clone());
                        self.output.push_str(", ");
                        self.generate_expr(c.arguments.args[1].clone());
                        if c.arguments.args.len() > 2 {
                            self.output.push_str(", \"{}\", ");
                            self.generate_expr(c.arguments.args[2].clone());
                        }
                    }
                    self.output.push_str(")");
                } else if func_name == "exit" {
                    self.output.push_str("std::process::exit(");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                } else if func_name == "assert_true" {
                    self.output.push_str("assert!(");
                    if let Some(arg) = c.arguments.args.first() {
                        self.generate_expr(arg.clone());
                    }
                    if c.arguments.args.len() > 1 {
                        self.output.push_str(", \"{}\", ");
                        self.generate_expr(c.arguments.args[1].clone());
                    }
                    self.output.push_str(")");
                } else if func_name == "range" {
                    if c.arguments.args.len() == 1 {
                        self.output.push_str("0..");
                        self.generate_expr(c.arguments.args[0].clone());
                    } else if c.arguments.args.len() >= 2 {
                        self.generate_expr(c.arguments.args[0].clone());
                        self.output.push_str("..");
                        self.generate_expr(c.arguments.args[1].clone());
                    }
                } else if func_name == "deref" {
                    if let Some(arg) = c.arguments.args.first() {
                        self.output.push_str("*");
                        self.generate_expr(arg.clone());
                    }
                } else if func_name == "as_ref" {
                    if let Some(arg) = c.arguments.args.first() {
                        self.output.push_str("&");
                        self.generate_expr(arg.clone());
                    }
                } else if func_name == "as_mut" {
                    if let Some(arg) = c.arguments.args.first() {
                        self.output.push_str("&mut ");
                        self.generate_expr(arg.clone());
                    }
                } else if !c.arguments.keywords.is_empty() {
                    // Struct Init
                    if matches!(&*c.func, ast::Expr::Subscript(_)) {
                        self.output.push_str(&self.map_type_expr(c.func.as_ref()));
                    } else {
                        self.generate_expr(*c.func.clone());
                    }
                    self.output.push_str(" { ");
                    for (i, kw) in c.arguments.keywords.iter().enumerate() {
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
                    // Generic Function Call
                    if matches!(&*c.func, ast::Expr::Subscript(_)) {
                        self.output.push_str(&self.map_type_expr(c.func.as_ref()));
                    } else {
                        self.generate_expr(*c.func.clone());
                    }
                    self.output.push_str("(");
                    for (i, arg) in c.arguments.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.generate_expr(arg.clone());
                    }
                    self.output.push_str(")");
                }

                if !is_intrinsic && !is_struct_init {
                    self.output.push_str(")"); // End check!
                }
            }
            ast::Expr::Attribute(a) => {
                let (base_str, base_is_type) = if let ast::Expr::Subscript(s) = &*a.value {
                    let base_name = self.expr_to_string(&s.value);
                    if self.is_type_or_mod(&base_name) {
                        (self.map_type_expr(&a.value), true)
                    } else {
                        (self.expr_to_string(&a.value), false)
                    }
                } else {
                    let base = self.expr_to_string(&a.value);
                    let is_type = self.is_type_or_mod(&base);
                    (base, is_type)
                };
                if base_is_type {
                    self.output.push_str(&base_str);
                } else {
                    self.generate_expr(*a.value.clone());
                }

                let sep = if matches!(
                    &*a.value,
                    ast::Expr::StringLiteral(_)
                        | ast::Expr::NumberLiteral(_)
                        | ast::Expr::BooleanLiteral(_)
                        | ast::Expr::NoneLiteral(_)
                        | ast::Expr::List(_)
                        | ast::Expr::Dict(_)
                        | ast::Expr::Tuple(_)
                        | ast::Expr::Lambda(_)
                ) {
                    "."
                } else if base_is_type {
                    "::"
                } else {
                    "."
                };
                let attr_name = if a.attr.as_str() == "def_" {
                    "def"
                } else {
                    a.attr.as_str()
                };
                self.output.push_str(&format!("{}{}", sep, attr_name));
            }
            ast::Expr::Name(n) => {
                self.output.push_str(&n.id);
            }
            ast::Expr::NoneLiteral(_) => self.output.push_str("None"),
            ast::Expr::NumberLiteral(n) => match n.value {
                ast::Number::Int(i) => self.output.push_str(&i.to_string()),
                ast::Number::Float(f) => {
                    let s = f.to_string();
                    if s.contains('.') || s.contains('e') || s.contains('E') {
                        self.output.push_str(&s);
                    } else {
                        self.output.push_str(&format!("{}.0", s));
                    }
                }
                ast::Number::Complex { .. } => self.output.push_str("/* complex number */"),
            },
            ast::Expr::StringLiteral(s) => self.output.push_str(&format!(
                "String::from(\"{}\")",
                s.value.to_str().replace("\"", "\\\"")
            )),
            ast::Expr::BooleanLiteral(b) => {
                self.output.push_str(if b.value { "true" } else { "false" })
            }
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
                self.output
                    .push_str("crate::quiche::check!(std::collections::HashMap::from([");
                for (i, item) in d.items.iter().enumerate() {
                    if let Some(key) = &item.key {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str("(");
                        self.generate_expr(key.clone());
                        self.output.push_str(", ");
                        self.generate_expr(item.value.clone());
                        self.output.push_str(")");
                    } else {
                        // **kwargs
                        self.output.push_str("/* **kwargs not supported */");
                    }
                }
                self.output.push_str("]))");
            }
            ast::Expr::Subscript(s) => {
                // Check for tuple/map access via symbol table
                let expr_type = self.get_expr_type(&s.value);
                let is_tuple = expr_type
                    .as_ref()
                    .map(|t| t.starts_with("("))
                    .unwrap_or(false);
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
                if !is_tuple {
                    self.output.push_str(".clone()");
                }
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
                // Ruff uses parameters? Or just args?
                // Error says: available fields are: `node_index`, `range`, `parameters`, `body`.
                // Parameters has posonlyargs, args, vararg, kwonlyargs, kw_defaults, kwarg, defaults.
                // We likely only care about `args` (positional args) for simple lambdas.
                let params = &l.parameters;
                // Since parameter structure is complex, let's iterate generic 'args'. (ArgWithDefault?)
                // Actually Parameters struct has fields posonlyargs, args, etc.
                if let Some(params) = params.as_deref() {
                    for (i, param_with_default) in params.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(&param_with_default.parameter.name);
                    }
                }

                self.output.push_str("| ");
                self.generate_expr(*l.body);
                self.output.push_str(")");
            }
            ast::Expr::FString(f) => {
                // f-string: f"Hello {name}" -> format!("Hello {}", name)
                self.output.push_str("format!(\"");
                let mut args: Vec<ast::Expr> = Vec::new();

                for part in &f.value {
                    match part {
                        ast::FStringPart::Literal(l) => {
                            self.output.push_str(&l.value);
                        }
                        ast::FStringPart::FString(f) => {
                            for element in &f.elements {
                                match element {
                                    ast::InterpolatedStringElement::Literal(l) => {
                                        self.output.push_str(&l.value)
                                    }
                                    ast::InterpolatedStringElement::Interpolation(i) => {
                                        self.output.push_str("{}");
                                        args.push(*i.expression.clone());
                                    }
                                }
                            }
                        }
                    }
                }

                self.output.push_str("\"");
                for arg in args {
                    self.output.push_str(", ");
                    self.generate_expr(arg);
                }
                self.output.push_str(")");
            }
            _ => self
                .output
                .push_str(&format!("/* unhandled expression: {:?} */", expr)),
        }
    }
}
