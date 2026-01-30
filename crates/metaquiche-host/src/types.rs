use crate::Codegen;
use quiche_parser::ast;

impl Codegen {
    pub(crate) fn map_type(&self, expr: &ast::QuicheExpr) -> String {
        self.map_type_internal(expr, false)
    }

    pub(crate) fn map_type_expr(&self, expr: &ast::QuicheExpr) -> String {
        self.map_type_internal(expr, true)
    }

    fn map_type_internal(&self, expr: &ast::QuicheExpr, is_expr: bool) -> String {
        let sep = if is_expr { "::" } else { "" };
        match expr {
            ast::QuicheExpr::Name(n) => match n.as_str() {
                // Signed Integers
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => n.to_string(),
                // Unsigned Integers
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => n.to_string(),
                // Floats
                "f32" | "f64" => n.to_string(),
                // Bool/String
                "Dict" | "HashMap" => "std::collections::HashMap".to_string(),
                "List" | "Vec" => "Vec".to_string(),
                "Option" => "Option".to_string(),
                "Result" => "Result".to_string(),
                "String" | "str" => "String".to_string(),
                "bool" => "bool".to_string(),
                "StrRef" => "&str".to_string(),
                _ => n.to_string(),
            },
            ast::QuicheExpr::Subscript { value, slice } => {
                let base = self.map_type_internal(value, false);
                let inner = self.map_type_internal(slice, false);

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
                    "Ref" => "&",
                    "MutRef" => "&mut",
                    _ => &base,
                };

                if rust_base == "std::collections::HashMap" {
                    format!("std::collections::HashMap{}<{}>", sep, final_inner)
                } else if rust_base == "&" {
                    format!("&{}", final_inner)
                } else if rust_base == "&mut" {
                    format!("&mut {}", final_inner)
                } else {
                    format!("{}{}<{}>", rust_base, sep, final_inner)
                }
            }
            ast::QuicheExpr::Tuple(elts) => {
                let mut output = String::from("(");
                for (i, elt) in elts.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    output.push_str(&self.map_type_internal(elt, false));
                }
                output.push_str(")");
                output
            }
            ast::QuicheExpr::Attribute { .. } => self.expr_to_string(expr),
            ast::QuicheExpr::Constant(c) => match c {
                ast::Constant::Str(_) => "String".to_string(),
                ast::Constant::Bool(_) => "bool".to_string(),
                ast::Constant::Int(_) => "i32".to_string(),
                ast::Constant::Float(_) => "f64".to_string(),
                _ => "UnknownConstant".to_string(),
            },
            _ => format!("/* complex type: {:?} */", expr),
        }
    }

    pub(crate) fn expr_to_string(&self, expr: &ast::QuicheExpr) -> String {
        match expr {
            ast::QuicheExpr::Name(n) => n.to_string(),
            ast::QuicheExpr::Attribute { value, attr } => {
                let base_str = self.expr_to_string(value);
                let sep = if self.is_type_or_mod(&base_str) {
                    "::"
                } else {
                    "."
                };
                format!("{}{}{}", base_str, sep, attr)
            }
            ast::QuicheExpr::Subscript { value, slice } => {
                let base = self.expr_to_string(value);
                let idx = self.expr_to_string(slice);
                format!("{}[{}]", base, idx)
            }
            ast::QuicheExpr::Constant(c) => match c {
                ast::Constant::Str(s) => s.to_string(),
                ast::Constant::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
                ast::Constant::Int(i) => i.to_string(),
                ast::Constant::Float(f) => f.to_string(),
                _ => "0".to_string(),
            },
            _ => format!("/* unknown: {:?} */", expr),
        }
    }

    pub(crate) fn get_expr_type(&self, expr: &ast::QuicheExpr) -> Option<String> {
        match expr {
            ast::QuicheExpr::Name(n) => self.get_symbol(n).cloned(),
            ast::QuicheExpr::Subscript { value, slice } => {
                let base_type = self.get_expr_type(value)?;
                // Check if base is a Tuple (starts with '(')
                if base_type.starts_with("(") {
                    // It's a tuple type: (A, B, C)
                    // We need to extract the Nth element type.
                    // Slice must be an integer constant.
                    if let ast::QuicheExpr::Constant(ast::Constant::Int(i)) = &**slice {
                        let idx_str = i.to_string();
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
