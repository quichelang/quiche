use crate::ast::*;
use ruff_python_ast as ast;
use ruff_python_parser::parse_module;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Ruff Parse Error: {0}")]
    RuffError(String),
    #[error("Semantic Error: {0}")]
    SemanticError(String),
}

pub fn parse(source: &str) -> Result<QuicheModule, ParseError> {
    let parsed = parse_module(source).map_err(|e| ParseError::RuffError(e.to_string()))?;
    let suite = parsed.into_suite();

    let mut quiche_body = Vec::new();

    for stmt in suite {
        quiche_body.push(lower_stmt(stmt)?);
    }

    Ok(QuicheModule { body: quiche_body })
}

fn lower_stmt(stmt: ast::Stmt) -> Result<QuicheStmt, ParseError> {
    match &stmt {
        ast::Stmt::ClassDef(c) => lower_class_def(c.clone(), stmt),
        ast::Stmt::Expr(e) => {
            if let ast::Expr::Call(c) = &*e.value {
                if let ast::Expr::Name(n) = &*c.func {
                    if n.id.as_str() == "rust" {
                        if let Some(arg) = c.arguments.args.first() {
                            if let ast::Expr::StringLiteral(s) = arg {
                                return Ok(QuicheStmt::RustBlock(s.value.to_string()));
                            }
                        }
                    }
                }
            }
            Ok(QuicheStmt::Stmt(stmt))
        }
        _ => Ok(QuicheStmt::Stmt(stmt)),
    }
}

fn lower_class_def(c: ast::StmtClassDef, original: ast::Stmt) -> Result<QuicheStmt, ParseError> {
    // Check Bases
    let is_struct = has_base(&c, "Struct");
    let is_enum = has_base(&c, "Enum");
    let is_trait = has_base(&c, "Trait");

    // Check Decorators
    let impl_trait = get_decorator(&c, "impl");

    if impl_trait.is_some() {
        // Handle Impl
        // TODO: Extract target type and trait name
        return Ok(QuicheStmt::ImplDef(ImplDef {
            trait_name: None, // Logic needed
            target_type: c.name.to_string(),
            body: vec![], // Logic needed to recurse
        }));
    }

    if is_struct {
        // Extract fields
        let mut fields = Vec::new();
        for s in &c.body {
            if let ast::Stmt::AnnAssign(a) = s {
                if let ast::Expr::Name(n) = &*a.target {
                    // Extract type annotation
                    // AnnAssign annotation is Box<Expr>, not Option
                    let ty = expr_to_string(&a.annotation);
                    fields.push(FieldDef {
                        name: n.id.to_string(),
                        ty,
                    });
                }
            }
        }

        return Ok(QuicheStmt::StructDef(StructDef {
            name: c.name.to_string(),
            type_params: extract_type_params(&c),
            fields,
        }));
    }

    if is_enum {
        // Extract variants
        let mut variants = Vec::new();
        for s in &c.body {
            if let ast::Stmt::Assign(a) = s {
                for target in &a.targets {
                    if let ast::Expr::Name(n) = target {
                        // Parse variant value for tuple fields: Red = (int,)
                        let mut fields = Vec::new();
                        // Assign value is Box<Expr>, not Option
                        if let ast::Expr::Tuple(t) = &*a.value {
                            for elt in &t.elts {
                                fields.push(expr_to_string(elt));
                            }
                        }

                        variants.push(VariantDef {
                            name: n.id.to_string(),
                            fields,
                        });
                    }
                }
            }
        }

        return Ok(QuicheStmt::EnumDef(EnumDef {
            name: c.name.to_string(),
            type_params: extract_type_params(&c),
            variants,
        }));
    }

    if is_trait {
        return Ok(QuicheStmt::TraitDef(TraitDef {
            name: c.name.to_string(),
            body: vec![], // Recurse
        }));
    }

    Ok(QuicheStmt::ClassDef(c))
}

fn has_base(c: &ast::StmtClassDef, name: &str) -> bool {
    // bases might be a method or field depending on ruff version. Compiler said method.
    // Wait, ruff AST usually has pub fields. Maybe I am using a newer version where it's different?
    // The compiler said `c.bases` is a method.
    c.bases().iter().any(|b| {
        if let ast::Expr::Name(n) = b {
            n.id.as_str() == name
        } else {
            false
        }
    })
}

fn get_decorator(c: &ast::StmtClassDef, name: &str) -> Option<ast::Expr> {
    c.decorator_list.iter().find_map(|d| {
        if let ast::Expr::Call(call) = &d.expression {
            if let ast::Expr::Name(n) = &*call.func {
                if n.id.as_str() == name {
                    return Some(d.expression.clone());
                }
            }
        }
        None
    })
}

fn extract_type_params(c: &ast::StmtClassDef) -> Vec<String> {
    // Extract [T] from Python 3.12 syntax
    if let Some(params) = &c.type_params {
        params
            .type_params
            .iter()
            .map(|p| match p {
                ast::TypeParam::TypeVar(t) => t.name.to_string(),
                _ => "?".to_string(),
            })
            .collect()
    } else {
        vec![]
    }
}

fn expr_to_string(expr: &ast::Expr) -> String {
    match expr {
        ast::Expr::Name(n) => n.id.to_string(),
        ast::Expr::Attribute(a) => format!("{}.{}", expr_to_string(&a.value), a.attr),
        ast::Expr::Subscript(s) => {
            let value = expr_to_string(&s.value);
            let slice = expr_to_string(&s.slice);
            format!("{}[{}]", value, slice)
        }
        ast::Expr::Tuple(t) => {
            let elts: Vec<String> = t.elts.iter().map(expr_to_string).collect();
            format!("({})", elts.join(", "))
        }
        ast::Expr::List(l) => {
            let elts: Vec<String> = l.elts.iter().map(expr_to_string).collect();
            format!("[{}]", elts.join(", "))
        }
        ast::Expr::StringLiteral(_) => "str".to_string(),
        ast::Expr::NumberLiteral(_) => "number".to_string(),
        ast::Expr::NoneLiteral(_) => "None".to_string(),
        ast::Expr::BooleanLiteral(b) => {
            if b.value {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        _ => "?".to_string(),
    }
}
