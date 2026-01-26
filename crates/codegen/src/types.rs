use crate::Codegen;
use rustpython_parser::ast;

impl Codegen {
    pub(crate) fn map_type(&self, expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::Name(n) => match n.id.as_str() {
                // Signed Integers
                "i8" => "i8".to_string(),
                "i16" => "i16".to_string(),
                "i32" => "i32".to_string(),
                "i64" => "i64".to_string(),
                "i128" => "i128".to_string(),
                "isize" => "isize".to_string(),

                // Unsigned Integers
                "u8" => "u8".to_string(),
                "u16" => "u16".to_string(),
                "u32" => "u32".to_string(),
                "u64" => "u64".to_string(),
                "u128" => "u128".to_string(),
                "usize" => "usize".to_string(),

                // Floats
                "f32" => "f32".to_string(),
                "f64" => "f64".to_string(),

                // Bool/String
                "bool" => "bool".to_string(),
                "str" => "&str".to_string(),
                "String" => "String".to_string(),

                // Pass through others
                _ => n.id.to_string(),
            },
            ast::Expr::Subscript(s) => {
                let base = self.map_type(&s.value);
                let inner = self.map_type(&s.slice);

                // Handle Vec[T] -> Vec<T>
                if base == "Vec" || base == "List" {
                    format!("Vec<{}>", inner)
                } else if base == "Option" {
                    format!("Option<{}>", inner)
                } else if base == "Result" || base == "HashMap" || base == "Dict" {
                    // Result[A, B] -> inner is (A, B) -> Result<A, B>
                    // We need to strip the parens from the tuple inner
                    let final_inner = if inner.starts_with("(") && inner.ends_with(")") {
                        &inner[1..inner.len() - 1]
                    } else {
                        &inner
                    };
                    let rust_base = if base == "Dict" || base == "HashMap" {
                        "std::collections::HashMap"
                    } else {
                        &base
                    };
                    format!("{}<{}>", rust_base, final_inner)
                } else if base == "Tuple" {
                    // Tuple[T, U] -> inner is (T, U)
                    // Tuple[T] -> inner is T -> (T,)
                    if inner.starts_with("(") && inner.ends_with(")") {
                        inner
                    } else {
                        format!("({},)", inner)
                    }
                } else {
                    format!("{}<{}>", base, inner)
                }
            }
            ast::Expr::Tuple(t) => {
                let mut output = String::from("(");
                for (i, elt) in t.elts.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.map_type(elt));
                }
                output.push_str(")");
                output
            }
            _ => format!("/* complex type: {:?} */", expr),
        }
    }

    pub(crate) fn expr_to_string(&self, expr: &ast::Expr) -> String {
        match expr {
            ast::Expr::Name(n) => n.id.to_string(),
            _ => "unknown".to_string(),
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
                    if let ast::Expr::Constant(c) = &*s.slice {
                        if let ast::Constant::Int(idx) = &c.value {
                            // Parse tuple string to find Nth element
                            let content = &base_type[1..base_type.len() - 1]; // Strip parens
                                                                              // Split by comma respecting parens (< and (
                            let mut depth = 0;
                            let mut start = 0;
                            let mut current_idx = 0;
                            let target_idx = idx.to_string().parse::<usize>().ok()?;

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
                }
                None
            }
            _ => None,
        }
    }
}
