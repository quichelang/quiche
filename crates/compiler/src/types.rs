use crate::Codegen;
use ruff_python_ast as ast;

impl Codegen {
    pub(crate) fn map_type(&self, expr: &ast::Expr) -> String {
        self.map_type_internal(expr, false)
    }

    pub(crate) fn map_type_expr(&self, expr: &ast::Expr) -> String {
        self.map_type_internal(expr, true)
    }

    fn map_type_internal(&self, expr: &ast::Expr, is_expr: bool) -> String {
        let sep = if is_expr { "::" } else { "" };
        match expr {
            ast::Expr::Name(n) => match n.id.as_str() {
                // Signed Integers
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => n.id.to_string(),
                // Unsigned Integers
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => n.id.to_string(),
                // Floats
                "f32" | "f64" => n.id.to_string(),
                // Bool/String
                "Dict" | "HashMap" => "std::collections::HashMap".to_string(),
                "List" | "Vec" => "Vec".to_string(),
                "Option" => "Option".to_string(),
                "Result" => "Result".to_string(),
                "String" | "str" => "String".to_string(),
                "bool" => "bool".to_string(),
                "StrRef" => "&str".to_string(),
                _ => n.id.to_string(),
            },
            ast::Expr::Subscript(s) => {
                let base = self.map_type_internal(&s.value, false);
                let inner = self.map_type_internal(&s.slice, false);

                // Strip parens from inner if it's a tuple
                let final_inner = if inner.starts_with("(") && inner.ends_with(")") {
                    &inner[1..inner.len() - 1]
                } else {
                    &inner
                };

                if base == "Tuple" {
                    return format!("({})", final_inner);
                }

                let rust_base = match base.as_str() {
                    "Dict" | "HashMap" => "std::collections::HashMap",
                    "List" | "Vec" => "Vec",
                    "Option" => "Option",
                    "Result" => "Result",
                    _ => &base,
                };

                format!("{}{}<{}>", rust_base, sep, final_inner)
            }
            ast::Expr::Tuple(t) => {
                let mut output = String::from("(");
                for (i, elt) in t.elts.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.map_type_internal(elt, false));
                }
                output.push_str(")");
                output
            }
            ast::Expr::Attribute(_) => self.expr_to_string(expr),
            _ => format!("/* complex type: {:?} */", expr),
        }
    }

    pub(crate) fn expr_to_string(&self, expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::Name(n) => n.id.to_string(),
            ast::Expr::Attribute(a) => {
                let base_str = self.expr_to_string(&a.value);
                let sep = if self.is_type_or_mod(&base_str) {
                    "::"
                } else {
                    "."
                };
                format!("{}{}{}", base_str, sep, a.attr)
            }
            ast::Expr::Subscript(_) => self.map_type_expr(expr), // Use turbo-fish for expressions
            _ => format!("/* unknown: {:?} */", expr),
        }
    }

    pub(crate) fn get_expr_type(&self, expr: &ast::Expr) -> Option<String> {
        match expr {
            ast::Expr::Name(n) => self.get_symbol(&n.id).cloned(),
            ast::Expr::Subscript(s) => {
                let base_type = self.get_expr_type(&s.value)?;
                // Check if base is a Tuple (starts with '(')
                if base_type.starts_with("(") {
                    // It's a tuple type: (A, B, C)
                    // We need to extract the Nth element type.
                    // Slice must be an integer constant.
                    if let ast::Expr::NumberLiteral(n) = &*s.slice {
                        // n.value is Number.
                        let idx_str = match &n.value {
                            ast::Number::Int(i) => i.to_string(),
                            ast::Number::Float(f) => f.to_string(),
                            _ => "0".to_string(),
                        };
                        // Parse tuple string to find Nth element
                        let content = &base_type[1..base_type.len() - 1]; // Strip parens
                                                                          // Split by comma respecting parens (< and (
                        let mut depth = 0;
                        let mut start = 0;
                        let mut current_idx = 0;
                        let target_idx = idx_str.parse::<usize>().ok()?;

                        for (i, c) in content.char_indices() {
                            match c {
                                '(' | '<' => depth += 1,
                                ')' | '>' => depth -= 1,
                                ',' => {
                                    if depth == 0 {
                                        if current_idx == target_idx {
                                            return Some(content[start..i].trim().to_string());
                                        }
                                        start = i + 1;
                                        current_idx += 1;
                                    }
                                }
                                _ => {}
                            }
                        }
                        // Last element
                        if current_idx == target_idx {
                            return Some(content[start..].trim().to_string());
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}
