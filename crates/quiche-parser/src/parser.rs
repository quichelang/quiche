use crate::ast::*;
use ruff_python_ast as ast;
use ruff_python_parser::parse_module;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Ruff Parse Error: {0}")]
    RuffError(String),
    #[error("Semantic Error: {0}")]
    SemanticError(String),
    #[error("Unsupported Syntax: {0}")]
    Unsupported(String),
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
    match stmt {
        ast::Stmt::ClassDef(c) => lower_class_def(c),
        ast::Stmt::FunctionDef(f) => lower_function_def(f),
        ast::Stmt::Expr(e) => {
            // Check for rust("...")
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
            Ok(QuicheStmt::Expr(Box::new(lower_expr(*e.value)?)))
        }
        ast::Stmt::Return(r) => {
            let value = if let Some(v) = r.value {
                Some(Box::new(lower_expr(*v)?))
            } else {
                None
            };
            Ok(QuicheStmt::Return(value))
        }
        ast::Stmt::Assign(a) => {
            let mut targets = Vec::new();
            for t in a.targets {
                targets.push(lower_expr(t)?);
            }
            Ok(QuicheStmt::Assign(Assign {
                targets,
                value: Box::new(lower_expr(*a.value)?),
            }))
        }
        ast::Stmt::AnnAssign(a) => Ok(QuicheStmt::AnnAssign(AnnAssign {
            target: Box::new(lower_expr(*a.target)?),
            annotation: Box::new(lower_expr(*a.annotation)?),
            value: if let Some(v) = a.value {
                Some(Box::new(lower_expr(*v)?))
            } else {
                None
            },
        })),
        ast::Stmt::If(i) => lower_if_stmt(i),
        ast::Stmt::Pass(_) => Ok(QuicheStmt::Pass),
        ast::Stmt::Break(_) => Ok(QuicheStmt::Break),
        ast::Stmt::Continue(_) => Ok(QuicheStmt::Continue),
        ast::Stmt::While(w) => Ok(QuicheStmt::While(WhileStmt {
            test: Box::new(lower_expr(*w.test)?),
            body: lower_block(w.body)?,
            orelse: lower_block(w.orelse)?,
        })),
        ast::Stmt::For(f) => Ok(QuicheStmt::For(ForStmt {
            target: Box::new(lower_expr(*f.target)?),
            iter: Box::new(lower_expr(*f.iter)?),
            body: lower_block(f.body)?,
            orelse: lower_block(f.orelse)?,
        })),
        // Simplified fallback for now to pass compilation of struct/enum tests
        _ => Err(ParseError::Unsupported(format!("{:?}", stmt))),
    }
}

// Re-implement If lowering robustly
fn lower_block(stmts: Vec<ast::Stmt>) -> Result<Vec<QuicheStmt>, ParseError> {
    stmts.into_iter().map(lower_stmt).collect()
}

// Fix logic for If:
// Ruff: If { test, body, elif_else_clauses }
// Quiche: IfStmt { test, body, orelse: Vec<Stmt> }
// We need to convert elif_else_clauses into a nested If chain or flat block.
// Python: if A: ... elif B: ... else: ...
// QuicheAST `orelse` is a block.
// Equivalent: if A: ... else: { if B: ... else: ... }
fn lower_if_stmt(i: ast::StmtIf) -> Result<QuicheStmt, ParseError> {
    let test = lower_expr(*i.test)?;
    let body = lower_block(i.body)?;

    let mut orelse = Vec::new();

    // Handle first elif as the start of the else block
    if let Some(first) = i.elif_else_clauses.first() {
        // Recursive reconstruction
        // But wait, Ruff provides a list of clauses.
        // We can reconstruct the chain.
        orelse = lower_elif_chain(i.elif_else_clauses)?;
    }

    Ok(QuicheStmt::If(IfStmt {
        test: Box::new(test),
        body,
        orelse,
    }))
}

fn lower_elif_chain(clauses: Vec<ast::ElifElseClause>) -> Result<Vec<QuicheStmt>, ParseError> {
    if clauses.is_empty() {
        return Ok(Vec::new());
    }

    let first = &clauses[0];
    let rest = clauses[1..].to_vec();

    if let Some(test) = &first.test {
        // syntax: elif test: body
        // becomes: [ If(test, body, lower(rest)) ]
        let body = lower_block(first.body.clone())?;
        let orelse = lower_elif_chain(rest)?;
        let nested_if = QuicheStmt::If(IfStmt {
            test: Box::new(lower_expr(test.clone())?),
            body,
            orelse,
        });
        Ok(vec![nested_if])
    } else {
        // syntax: else: body
        // Final else. 'rest' must be empty if valid python.
        lower_block(first.body.clone())
    }
}

fn lower_function_def(f: ast::StmtFunctionDef) -> Result<QuicheStmt, ParseError> {
    Ok(QuicheStmt::FunctionDef(FunctionDef {
        name: f.name.to_string(),
        args: f
            .parameters
            .args
            .into_iter()
            .map(|p| {
                let annotation = p.parameter.annotation.map(|e| {
                    Box::new(lower_expr(*e).unwrap_or_else(|err| {
                        // Handle error or provide a default/placeholder expression
                        eprintln!(
                            "Warning: Failed to lower function argument annotation: {}",
                            err
                        );
                        QuicheExpr::Name("ErrorType".to_string()) // Placeholder
                    }))
                });
                Arg {
                    arg: p.parameter.name.to_string(),
                    annotation,
                }
            })
            .collect(),
        body: lower_block(f.body)?,
        decorator_list: f
            .decorator_list
            .into_iter()
            .map(|d| lower_expr(d.expression))
            .collect::<Result<_, _>>()?,
        returns: f.returns.map(|e| {
            Box::new(lower_expr(*e).unwrap_or_else(|err| {
                eprintln!(
                    "Warning: Failed to lower function return annotation: {}",
                    err
                );
                QuicheExpr::Name("ErrorType".to_string()) // Placeholder
            }))
        }),
        type_params: extract_type_params_def(&f.type_params),
    }))
}

fn lower_class_def(c: ast::StmtClassDef) -> Result<QuicheStmt, ParseError> {
    // Semantic checks (Struct/Enum/Trait)
    let is_struct = has_base(&c, "Struct");
    let is_enum = has_base(&c, "Enum");
    let is_trait = has_base(&c, "Trait");

    // Check Decorators for Impl
    let impl_trait = get_decorator(&c, "impl");

    if is_trait {
        return Ok(QuicheStmt::TraitDef(TraitDef {
            name: c.name.to_string(),
            body: lower_block(c.body)?,
        }));
    }

    if impl_trait.is_some() {
        // Handle Impl
        return Ok(QuicheStmt::ImplDef(ImplDef {
            trait_name: None, // TODO: Extract from decorator args if needed
            target_type: c.name.to_string(),
            body: lower_block(c.body)?,
        }));
    }

    if is_struct {
        // Extract fields logic from previous step, but now returning StructDef
        let mut fields = Vec::new();
        for s in &c.body {
            if let ast::Stmt::AnnAssign(a) = s {
                if let ast::Expr::Name(n) = &*a.target {
                    let ty = expr_to_string_compat(&*a.annotation);
                    fields.push(FieldDef {
                        name: n.id.to_string(),
                        ty,
                    });
                }
            }
        }
        return Ok(QuicheStmt::StructDef(StructDef {
            name: c.name.to_string(),
            type_params: extract_type_params_class(&c.type_params),
            fields,
        }));
    }

    if is_enum {
        let mut variants = Vec::new();
        for s in &c.body {
            if let ast::Stmt::Assign(a) = s {
                for target in &a.targets {
                    if let ast::Expr::Name(n) = target {
                        let mut fields = Vec::new();
                        if let ast::Expr::Tuple(t) = &*a.value {
                            for elt in &t.elts {
                                fields.push(expr_to_string_compat(elt));
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
            type_params: extract_type_params_class(&c.type_params),
            variants,
        }));
    }

    // Default to ClassDef proxy
    Ok(QuicheStmt::ClassDef(ClassDef {
        name: c.name.to_string(),
        bases: c
            .bases()
            .iter()
            .map(|b| lower_expr(b.clone()))
            .collect::<Result<_, _>>()?,
        body: lower_block(c.body)?,
        decorator_list: c
            .decorator_list
            .into_iter()
            .map(|d| lower_expr(d.expression))
            .collect::<Result<_, _>>()?,
        type_params: extract_type_params_class(&c.type_params),
    }))
}

fn lower_expr(expr: ast::Expr) -> Result<QuicheExpr, ParseError> {
    match expr {
        ast::Expr::Name(n) => Ok(QuicheExpr::Name(n.id.to_string())),
        ast::Expr::NumberLiteral(n) => match n.value {
            ast::Number::Int(i) => Ok(QuicheExpr::Constant(Constant::Int(i.as_i64().unwrap_or(0)))),
            ast::Number::Float(f) => Ok(QuicheExpr::Constant(Constant::Float(f))),
            ast::Number::Complex { .. } => Err(ParseError::Unsupported("Complex numbers".into())),
        },
        ast::Expr::StringLiteral(s) => Ok(QuicheExpr::Constant(Constant::Str(s.value.to_string()))),
        ast::Expr::BooleanLiteral(b) => Ok(QuicheExpr::Constant(Constant::Bool(b.value))),
        ast::Expr::NoneLiteral(_) => Ok(QuicheExpr::Constant(Constant::None)),
        ast::Expr::BinOp(b) => Ok(QuicheExpr::BinOp {
            left: Box::new(lower_expr(*b.left)?),
            op: lower_operator(b.op),
            right: Box::new(lower_expr(*b.right)?),
        }),
        ast::Expr::Call(c) => Ok(QuicheExpr::Call {
            func: Box::new(lower_expr(*c.func)?),
            args: c
                .arguments
                .args
                .into_iter()
                .map(lower_expr)
                .collect::<Result<_, _>>()?,
            keywords: c
                .arguments
                .keywords
                .into_iter()
                .map(|k| {
                    Ok(Keyword {
                        arg: k.arg.map(|id| id.to_string()),
                        value: Box::new(lower_expr(k.value)?),
                    })
                })
                .collect::<Result<_, _>>()?,
        }),
        ast::Expr::Attribute(a) => Ok(QuicheExpr::Attribute {
            value: Box::new(lower_expr(*a.value)?),
            attr: a.attr.to_string(),
        }),
        ast::Expr::Tuple(t) => Ok(QuicheExpr::Tuple(
            t.elts
                .into_iter()
                .map(lower_expr)
                .collect::<Result<_, _>>()?,
        )),
        ast::Expr::List(l) => Ok(QuicheExpr::List(
            l.elts
                .into_iter()
                .map(lower_expr)
                .collect::<Result<_, _>>()?,
        )),
        _ => Err(ParseError::Unsupported(format!("{:?}", expr))),
    }
}

fn lower_operator(op: ast::Operator) -> Operator {
    match op {
        ast::Operator::Add => Operator::Add,
        ast::Operator::Sub => Operator::Sub,
        ast::Operator::Mult => Operator::Mult,
        ast::Operator::Div => Operator::Div,
        ast::Operator::Mod => Operator::Mod,
        ast::Operator::Pow => Operator::Pow,
        ast::Operator::LShift => Operator::LShift,
        ast::Operator::RShift => Operator::RShift,
        ast::Operator::BitOr => Operator::BitOr,
        ast::Operator::BitXor => Operator::BitXor,
        ast::Operator::BitAnd => Operator::BitAnd,
        ast::Operator::FloorDiv => Operator::FloorDiv,
        ast::Operator::MatMult => Operator::Mult, // fallback
    }
}

// Helpers
fn has_base(c: &ast::StmtClassDef, name: &str) -> bool {
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

fn extract_type_params_class(params: &Option<Box<ast::TypeParams>>) -> Vec<String> {
    if let Some(p) = params {
        p.type_params
            .iter()
            .map(|tp| match tp {
                ast::TypeParam::TypeVar(t) => t.name.to_string(),
                _ => "?".to_string(),
            })
            .collect()
    } else {
        vec![]
    }
}

fn extract_type_params_def(params: &Option<Box<ast::TypeParams>>) -> Vec<String> {
    if let Some(p) = params {
        p.type_params
            .iter()
            .map(|tp| match tp {
                ast::TypeParam::TypeVar(t) => t.name.to_string(),
                _ => "?".to_string(),
            })
            .collect()
    } else {
        vec![]
    }
}

// Temporary compat helper for fields (legacy expr_to_string)
// In real use, we should traverse the QuicheExpr for printing, or store QuicheExpr in FieldDef
// For now, let's keep string for type representation
fn expr_to_string_compat(expr: &ast::Expr) -> String {
    match expr {
        ast::Expr::Name(n) => n.id.to_string(),
        _ => "?".to_string(),
    }
}
