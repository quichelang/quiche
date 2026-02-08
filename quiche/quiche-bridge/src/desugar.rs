//! Quiche → Elevate AST bridge.
//!
//! Converts a `metaquiche_parser::ast::QuicheModule` into an
//! `elevate::ast::Module`, desugaring Python-like constructs (f-strings,
//! list comprehensions, range(), etc.) into their Elevate equivalents.

use elevate::ast as e;
use metaquiche_parser::ast as q;

// ─────────────────────────────────────────────────────────────────────────
// Top-level entry point
// ─────────────────────────────────────────────────────────────────────────

/// Convert a Quiche module to an Elevate AST module.
pub fn lower(module: &q::QuicheModule) -> e::Module {
    let mut items = Vec::new();
    for stmt in &module.body {
        lower_top_level_stmt(stmt, &mut items);
    }
    e::Module { items }
}

// ─────────────────────────────────────────────────────────────────────────
// Top-level statements → Items
// ─────────────────────────────────────────────────────────────────────────

fn lower_top_level_stmt(stmt: &q::QuicheStmt, items: &mut Vec<e::Item>) {
    match stmt {
        q::QuicheStmt::StructDef(s) => items.push(e::Item::Struct(lower_struct(s))),
        q::QuicheStmt::EnumDef(en) => items.push(e::Item::Enum(lower_enum(en))),
        q::QuicheStmt::FunctionDef(f) => items.push(e::Item::Function(lower_function(f, true))),
        q::QuicheStmt::ConstDef(c) => items.push(e::Item::Const(lower_const_def(c))),
        q::QuicheStmt::RustBlock(code) => items.push(e::Item::RustBlock(code.clone())),
        q::QuicheStmt::ImplDef(imp) => items.push(e::Item::Impl(lower_impl(imp))),
        q::QuicheStmt::TraitDef(_) => {
            // Traits are emitted as raw Rust blocks for now
        }
        q::QuicheStmt::ImportFrom(imp) => {
            if let Some(item) = lower_import_from(imp) {
                items.push(item);
            }
        }
        q::QuicheStmt::Import(_) => {} // bare imports not supported in Elevate
        q::QuicheStmt::ClassDef(c) => {
            // ClassDef can produce multiple items (struct + impl, enum + impl, etc.)
            items.extend(lower_class_def(c));
        }
        // Statements that aren't valid at top level but we handle gracefully
        q::QuicheStmt::Expr(e_expr) => {
            // Top-level expression (like a bare `run()` call)
            // Wrap in a __quiche_init function
            let call_stmt = lower_expr_to_stmt(e_expr);
            items.push(e::Item::Function(e::FunctionDef {
                visibility: e::Visibility::Private,
                name: "__quiche_main".to_string(),
                type_params: vec![],
                params: vec![],
                return_type: None,
                body: e::Block {
                    statements: vec![call_stmt],
                },
            }));
        }
        _ => {}
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Struct
// ─────────────────────────────────────────────────────────────────────────

fn lower_struct(s: &q::StructDef) -> e::StructDef {
    e::StructDef {
        visibility: e::Visibility::Public,
        name: s.name.clone(),
        fields: s
            .fields
            .iter()
            .map(|f| e::Field {
                name: f.name.clone(),
                ty: parse_type_string(&f.ty),
            })
            .collect(),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Enum
// ─────────────────────────────────────────────────────────────────────────

fn lower_enum(en: &q::EnumDef) -> e::EnumDef {
    e::EnumDef {
        visibility: e::Visibility::Public,
        name: en.name.clone(),
        variants: en
            .variants
            .iter()
            .map(|v| e::EnumVariant {
                name: v.name.clone(),
                payload: v.fields.iter().map(|f| parse_type_string(f)).collect(),
            })
            .collect(),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Function
// ─────────────────────────────────────────────────────────────────────────

fn lower_function(f: &q::FunctionDef, is_public: bool) -> e::FunctionDef {
    let vis = if is_public {
        e::Visibility::Public
    } else {
        e::Visibility::Private
    };

    // Handle type params
    let type_params = f
        .type_params
        .iter()
        .map(|tp| e::GenericParam {
            name: tp.clone(),
            bounds: vec![],
        })
        .collect();

    // Handle self parameter — if present, add as first param
    let mut params: Vec<e::Param> = Vec::new();
    match f.self_kind {
        q::SelfKind::Value(_) => {
            params.push(e::Param {
                name: "self".to_string(),
                ty: e::Type {
                    path: vec!["Self".to_string()],
                    args: vec![],
                    trait_bounds: vec![],
                },
            });
        }
        q::SelfKind::Ref(_) => {
            params.push(e::Param {
                name: "self".to_string(),
                ty: e::Type {
                    path: vec!["Self".to_string()],
                    args: vec![],
                    trait_bounds: vec![],
                },
            });
        }
        q::SelfKind::NoSelf => {}
    }

    // Add remaining params
    for arg in &f.args {
        params.push(e::Param {
            name: arg.arg.clone(),
            ty: arg
                .annotation
                .as_ref()
                .map(|ann| lower_type_expr(ann))
                .unwrap_or_else(|| e::Type {
                    path: vec!["i64".to_string()],
                    args: vec![],
                    trait_bounds: vec![],
                }),
        });
    }

    // Return type
    let return_type = f.returns.as_ref().map(|ret| lower_type_expr(ret));

    // Body
    let body = lower_body(&f.body);

    e::FunctionDef {
        visibility: vis,
        name: f.name.clone(),
        type_params,
        params,
        return_type,
        body,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Impl block
// ─────────────────────────────────────────────────────────────────────────

fn lower_impl(imp: &q::ImplDef) -> e::ImplBlock {
    let methods = imp
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            q::QuicheStmt::FunctionDef(f) => Some(lower_function(f, true)),
            _ => None,
        })
        .collect();
    e::ImplBlock {
        target: imp.target_type.clone(),
        methods,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Class def → Struct + Impl
// ─────────────────────────────────────────────────────────────────────────

fn lower_class_def(c: &q::ClassDef) -> Vec<e::Item> {
    let mut items = Vec::new();

    // Check if any base is "Struct", "Enum", or "Trait"
    let base_name = c.bases.first().and_then(|b| {
        if let q::QuicheExpr::Name(n) = b {
            Some(n.as_str())
        } else {
            None
        }
    });

    match base_name {
        Some("Struct") => {
            // Extract fields from AnnAssign statements
            let fields = c
                .body
                .iter()
                .filter_map(|stmt| {
                    if let q::QuicheStmt::AnnAssign(a) = stmt {
                        let name = extract_name(&a.target)?;
                        Some(e::Field {
                            name,
                            ty: lower_type_expr(&a.annotation),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            items.push(e::Item::Struct(e::StructDef {
                visibility: e::Visibility::Public,
                name: c.name.clone(),
                fields,
            }));
        }
        Some("Enum") => {
            // Extract variants from Assign statements
            let variants = c
                .body
                .iter()
                .filter_map(|stmt| {
                    if let q::QuicheStmt::Assign(a) = stmt {
                        let var_name = extract_name(a.targets.first()?)?;
                        let payload = match &*a.value {
                            q::QuicheExpr::Tuple(elts) => {
                                elts.iter().map(|e| lower_type_expr(e)).collect()
                            }
                            q::QuicheExpr::Constant(q::Constant::NoneVal) => vec![],
                            _ => vec![],
                        };
                        Some(e::EnumVariant {
                            name: var_name,
                            payload,
                        })
                    } else {
                        None
                    }
                })
                .collect();
            items.push(e::Item::Enum(e::EnumDef {
                visibility: e::Visibility::Public,
                name: c.name.clone(),
                variants,
            }));
        }
        Some("Trait") => {
            // Trait definitions are not yet supported in Elevate bridge
        }
        _ => {}
    }

    // Extract methods into an impl block
    let methods: Vec<e::FunctionDef> = c
        .body
        .iter()
        .filter_map(|stmt| match stmt {
            q::QuicheStmt::FunctionDef(f) => Some(lower_function(f, true)),
            _ => None,
        })
        .collect();

    if !methods.is_empty() {
        items.push(e::Item::Impl(e::ImplBlock {
            target: c.name.clone(),
            methods,
        }));
    }

    items
}

// ─────────────────────────────────────────────────────────────────────────
// ConstDef
// ─────────────────────────────────────────────────────────────────────────

fn lower_const_def(c: &q::ConstDef) -> e::ConstDef {
    e::ConstDef {
        visibility: e::Visibility::Public,
        name: c.name.clone(),
        ty: Some(lower_type_expr(&c.ty)),
        value: lower_expr(&c.value),
        is_const: true,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Import → RustUse
// ─────────────────────────────────────────────────────────────────────────

fn lower_import_from(imp: &q::ImportFrom) -> Option<e::Item> {
    let module = imp.module.as_ref()?;
    // Convert "from rust.std.collections import HashMap" → "rust use std::collections::HashMap"
    let stripped = module.strip_prefix("rust.")?;
    for alias in &imp.names {
        let mut path: Vec<String> = stripped.split('.').map(String::from).collect();
        path.push(alias.name.clone());
        return Some(e::Item::RustUse(e::RustUse { path }));
    }
    None
}

// ─────────────────────────────────────────────────────────────────────────
// Body (Vec<QuicheStmt> → Block)
// ─────────────────────────────────────────────────────────────────────────

fn lower_body(stmts: &[q::QuicheStmt]) -> e::Block {
    let statements = stmts.iter().filter_map(|s| lower_stmt(s)).collect();
    e::Block { statements }
}

// ─────────────────────────────────────────────────────────────────────────
// Statements
// ─────────────────────────────────────────────────────────────────────────

fn lower_stmt(stmt: &q::QuicheStmt) -> Option<e::Stmt> {
    match stmt {
        q::QuicheStmt::Return(expr) => Some(e::Stmt::Return(expr.as_ref().map(|e| lower_expr(e)))),
        q::QuicheStmt::Assign(a) => lower_assign(a),
        q::QuicheStmt::AnnAssign(a) => lower_ann_assign(a),
        q::QuicheStmt::If(i) => Some(lower_if(i)),
        q::QuicheStmt::While(w) => Some(lower_while(w)),
        q::QuicheStmt::For(f) => Some(lower_for(f)),
        q::QuicheStmt::Expr(e) => Some(lower_expr_to_stmt(e)),
        q::QuicheStmt::Match(m) => Some(lower_match_stmt(m)),
        q::QuicheStmt::Break => Some(e::Stmt::Break),
        q::QuicheStmt::Continue => Some(e::Stmt::Continue),
        q::QuicheStmt::Pass => None,
        q::QuicheStmt::Assert(a) => Some(lower_assert(a)),
        q::QuicheStmt::RustBlock(code) => Some(e::Stmt::RustBlock(code.clone())),
        // Nested function/struct/enum defs not supported in Elevate block context
        _ => None,
    }
}

fn lower_assign(a: &q::Assign) -> Option<e::Stmt> {
    if a.targets.is_empty() {
        return None;
    }
    let value = lower_expr(&a.value);

    // In Quiche (like Python), `x = value` is a variable declaration.
    // Only field/index targets are true reassignments.
    match &a.targets[0] {
        q::QuicheExpr::Name(name) => {
            // Simple name → let binding (Elevate's Const with is_const=false)
            Some(e::Stmt::Const(e::ConstDef {
                visibility: e::Visibility::Private,
                name: name.clone(),
                ty: None, // let Elevate infer
                value,
                is_const: false,
            }))
        }
        q::QuicheExpr::Tuple(elts) => {
            // Tuple destructuring → let (a, b) = value
            let pattern = e::DestructurePattern::Tuple(
                elts.iter().map(|e| lower_destructure_pattern(e)).collect(),
            );
            Some(e::Stmt::DestructureConst {
                pattern,
                value,
                is_const: false,
            })
        }
        _ => {
            // Field/index → actual reassignment
            let target = lower_assign_target(&a.targets[0]);
            Some(e::Stmt::Assign {
                target,
                op: e::AssignOp::Assign,
                value,
            })
        }
    }
}

fn lower_ann_assign(a: &q::AnnAssign) -> Option<e::Stmt> {
    let value = a.value.as_ref()?;
    let name = extract_name(&a.target)?;
    let ty = Some(lower_type_expr(&a.annotation));
    Some(e::Stmt::Const(e::ConstDef {
        visibility: e::Visibility::Private,
        name,
        ty,
        value: lower_expr(value),
        is_const: false, // `let` binding, not `const`
    }))
}

fn lower_if(i: &q::IfStmt) -> e::Stmt {
    e::Stmt::If {
        condition: lower_expr(&i.test),
        then_block: lower_body(&i.body),
        else_block: if i.orelse.is_empty() {
            None
        } else {
            Some(lower_body(&i.orelse))
        },
    }
}

fn lower_while(w: &q::WhileStmt) -> e::Stmt {
    e::Stmt::While {
        condition: lower_expr(&w.test),
        body: lower_body(&w.body),
    }
}

fn lower_for(f: &q::ForStmt) -> e::Stmt {
    let binding = lower_destructure_pattern(&f.target);
    e::Stmt::For {
        binding,
        iter: lower_expr(&f.iter),
        body: lower_body(&f.body),
    }
}

fn lower_match_stmt(m: &q::MatchStmt) -> e::Stmt {
    // Match as expression statement
    e::Stmt::Expr(e::Expr::Match {
        scrutinee: Box::new(lower_expr(&m.subject)),
        arms: m.cases.iter().map(|c| lower_match_arm(c)).collect(),
    })
}

fn lower_assert(a: &q::AssertStmt) -> e::Stmt {
    // assert(expr) → assert!(expr)
    let args = vec![lower_expr(&a.test)];
    e::Stmt::Expr(e::Expr::MacroCall {
        path: vec!["assert".to_string()],
        args,
    })
}

fn lower_expr_to_stmt(expr: &q::QuicheExpr) -> e::Stmt {
    e::Stmt::Expr(lower_expr(expr))
}

// ─────────────────────────────────────────────────────────────────────────
// Assignment targets
// ─────────────────────────────────────────────────────────────────────────

fn lower_assign_target(expr: &q::QuicheExpr) -> e::AssignTarget {
    match expr {
        q::QuicheExpr::Name(n) => e::AssignTarget::Path(n.clone()),
        q::QuicheExpr::Attribute { value, attr } => e::AssignTarget::Field {
            base: Box::new(lower_expr(value)),
            field: attr.clone(),
        },
        q::QuicheExpr::Subscript { value, slice } => e::AssignTarget::Index {
            base: Box::new(lower_expr(value)),
            index: Box::new(lower_expr(slice)),
        },
        q::QuicheExpr::Tuple(elts) => {
            e::AssignTarget::Tuple(elts.iter().map(|e| lower_assign_target(e)).collect())
        }
        _ => e::AssignTarget::Path("__unknown".to_string()),
    }
}

fn lower_destructure_pattern(expr: &q::QuicheExpr) -> e::DestructurePattern {
    match expr {
        q::QuicheExpr::Name(n) => {
            if n == "_" {
                e::DestructurePattern::Ignore
            } else {
                e::DestructurePattern::Name(n.clone())
            }
        }
        q::QuicheExpr::Tuple(elts) => e::DestructurePattern::Tuple(
            elts.iter().map(|e| lower_destructure_pattern(e)).collect(),
        ),
        _ => e::DestructurePattern::Name("__unknown".to_string()),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Expressions
// ─────────────────────────────────────────────────────────────────────────

fn lower_expr(expr: &q::QuicheExpr) -> e::Expr {
    match expr {
        q::QuicheExpr::Constant(c) => lower_constant(c),
        q::QuicheExpr::Name(n) => {
            // Special names
            match n.as_str() {
                "True" | "true" => e::Expr::Bool(true),
                "False" | "false" => e::Expr::Bool(false),
                "None" => e::Expr::Path(vec!["None".to_string()]),
                _ => e::Expr::Path(vec![n.clone()]),
            }
        }
        q::QuicheExpr::BinOp { left, op, right } => lower_binop(left, op, right),
        q::QuicheExpr::BoolOp { op, values } => lower_boolop(op, values),
        q::QuicheExpr::UnaryOp { op, operand } => lower_unaryop(op, operand),
        q::QuicheExpr::Compare {
            left,
            ops,
            comparators,
        } => lower_compare(left, ops, comparators),
        q::QuicheExpr::Call {
            func,
            args,
            keywords,
        } => lower_call(func, args, keywords),
        q::QuicheExpr::Attribute { value, attr } => {
            // Check if this is a type::method pattern (e.g., Student.new)
            // Capitalized names are likely types, emit as path
            if let q::QuicheExpr::Name(name) = value.as_ref() {
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    return e::Expr::Path(vec![name.clone(), attr.clone()]);
                }
            }
            e::Expr::Field {
                base: Box::new(lower_expr(value)),
                field: attr.clone(),
            }
        }
        q::QuicheExpr::Subscript { value, slice } => lower_subscript(value, slice),
        q::QuicheExpr::List(elts) => e::Expr::Array(elts.iter().map(lower_expr).collect()),
        q::QuicheExpr::Tuple(elts) => e::Expr::Tuple(elts.iter().map(lower_expr).collect()),
        q::QuicheExpr::Lambda {
            args,
            return_type,
            body,
        } => lower_lambda(args, return_type, body),
        q::QuicheExpr::ListComp {
            element,
            generators,
        } => lower_list_comp(element, generators),
        q::QuicheExpr::DictComp {
            key,
            value,
            generators,
        } => lower_dict_comp(key, value, generators),
        q::QuicheExpr::IfExp { test, body, orelse } => lower_if_expr(test, body, orelse),
        q::QuicheExpr::Cast { expr, target_type } => lower_cast(expr, target_type),
        q::QuicheExpr::FString(parts) => lower_fstring(parts),
        q::QuicheExpr::Slice {
            lower: lo,
            upper: hi,
            step: _,
        } => {
            // Slice → Range
            e::Expr::Range {
                start: lo.as_ref().map(|e| Box::new(lower_expr(e))),
                end: hi.as_ref().map(|e| Box::new(lower_expr(e))),
                inclusive: false,
            }
        }
        q::QuicheExpr::Borrow { kind, expr } => {
            let inner = lower_expr(expr);
            let macro_name = match kind {
                q::BorrowKind::Ref => "qref",
                q::BorrowKind::RefMut => "mutref",
            };
            e::Expr::MacroCall {
                path: vec![macro_name.to_string()],
                args: vec![inner],
            }
        }
        q::QuicheExpr::Deref(expr) => e::Expr::MacroCall {
            path: vec!["deref".to_string()],
            args: vec![lower_expr(expr)],
        },
    }
}

fn lower_constant(c: &q::Constant) -> e::Expr {
    match c {
        q::Constant::Int(i) => e::Expr::Int(*i),
        q::Constant::Float(f) => {
            // Elevate doesn't have float literals in ast::Expr — encode as a
            // string path for now and let codegen handle it.
            e::Expr::Path(vec![format!("{f}_f64")])
        }
        q::Constant::Bool(b) => e::Expr::Bool(*b),
        q::Constant::Str(s) => e::Expr::String(s.clone()),
        q::Constant::NoneVal => e::Expr::Path(vec!["None".to_string()]),
        q::Constant::Ellipsis => e::Expr::Tuple(vec![]), // () unit
    }
}

fn lower_binop(left: &q::QuicheExpr, op: &q::Operator, right: &q::QuicheExpr) -> e::Expr {
    let e_op = match op {
        q::Operator::Add => e::BinaryOp::Add,
        q::Operator::Sub => e::BinaryOp::Sub,
        q::Operator::Mult => e::BinaryOp::Mul,
        q::Operator::Div => e::BinaryOp::Div,
        q::Operator::Mod => e::BinaryOp::Rem,
        q::Operator::FloorDiv => e::BinaryOp::Div, // approximate
        // Bitwise ops aren't in Elevate's BinaryOp, emit as method calls
        _ => {
            return e::Expr::Call {
                callee: Box::new(e::Expr::Path(vec!["__quiche_bitop".to_string()])),
                args: vec![lower_expr(left), lower_expr(right)],
            };
        }
    };
    e::Expr::Binary {
        op: e_op,
        left: Box::new(lower_expr(left)),
        right: Box::new(lower_expr(right)),
    }
}

fn lower_boolop(op: &q::BoolOperator, values: &[q::QuicheExpr]) -> e::Expr {
    let e_op = match op {
        q::BoolOperator::And => e::BinaryOp::And,
        q::BoolOperator::Or => e::BinaryOp::Or,
    };
    // Chain: a and b and c → (a and b) and c
    let mut iter = values.iter();
    let first = lower_expr(
        iter.next()
            .unwrap_or(&q::QuicheExpr::Constant(q::Constant::Bool(true))),
    );
    iter.fold(first, |acc, val| e::Expr::Binary {
        op: e_op.clone(),
        left: Box::new(acc),
        right: Box::new(lower_expr(val)),
    })
}

fn lower_unaryop(op: &q::UnaryOperator, operand: &q::QuicheExpr) -> e::Expr {
    let e_op = match op {
        q::UnaryOperator::Not => e::UnaryOp::Not,
        q::UnaryOperator::USub => e::UnaryOp::Neg,
        q::UnaryOperator::UAdd => return lower_expr(operand), // +x is just x
        q::UnaryOperator::Invert => {
            // Bitwise invert: !x
            return e::Expr::Unary {
                op: e::UnaryOp::Not,
                expr: Box::new(lower_expr(operand)),
            };
        }
    };
    e::Expr::Unary {
        op: e_op,
        expr: Box::new(lower_expr(operand)),
    }
}

fn lower_compare(
    left: &q::QuicheExpr,
    ops: &[q::CmpOperator],
    comparators: &[q::QuicheExpr],
) -> e::Expr {
    // Single comparison: left op right
    // Chain: left op1 mid op2 right → (left op1 mid) && (mid op2 right)
    if ops.len() == 1 && comparators.len() == 1 {
        let e_op = lower_cmp_op(&ops[0]);
        return e::Expr::Binary {
            op: e_op,
            left: Box::new(lower_expr(left)),
            right: Box::new(lower_expr(&comparators[0])),
        };
    }

    // Chain comparisons
    let mut parts = Vec::new();
    let mut prev = lower_expr(left);
    for (op, comp) in ops.iter().zip(comparators.iter()) {
        let e_op = lower_cmp_op(op);
        let right = lower_expr(comp);
        parts.push(e::Expr::Binary {
            op: e_op,
            left: Box::new(prev.clone()),
            right: Box::new(right.clone()),
        });
        prev = right;
    }

    // AND all parts together
    let first = parts.remove(0);
    parts.into_iter().fold(first, |acc, part| e::Expr::Binary {
        op: e::BinaryOp::And,
        left: Box::new(acc),
        right: Box::new(part),
    })
}

fn lower_cmp_op(op: &q::CmpOperator) -> e::BinaryOp {
    match op {
        q::CmpOperator::Eq => e::BinaryOp::Eq,
        q::CmpOperator::NotEq => e::BinaryOp::Ne,
        q::CmpOperator::Lt => e::BinaryOp::Lt,
        q::CmpOperator::LtE => e::BinaryOp::Le,
        q::CmpOperator::Gt => e::BinaryOp::Gt,
        q::CmpOperator::GtE => e::BinaryOp::Ge,
        // Is/IsNot/In/NotIn → equality for now
        q::CmpOperator::Is => e::BinaryOp::Eq,
        q::CmpOperator::IsNot => e::BinaryOp::Ne,
        q::CmpOperator::In => e::BinaryOp::Eq, // placeholder
        q::CmpOperator::NotIn => e::BinaryOp::Ne, // placeholder
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Calls
// ─────────────────────────────────────────────────────────────────────────

fn lower_call(func: &q::QuicheExpr, args: &[q::QuicheExpr], keywords: &[q::Keyword]) -> e::Expr {
    // Special-case: range(n), range(a, b), range(a, b, step)
    if let q::QuicheExpr::Name(name) = func {
        match name.as_str() {
            "range" => return lower_range_call(args),
            "print" => return lower_print_call(args),
            "len" => {
                // len(x) → x.len()
                if let Some(arg) = args.first() {
                    return e::Expr::Call {
                        callee: Box::new(e::Expr::Field {
                            base: Box::new(lower_expr(arg)),
                            field: "len".to_string(),
                        }),
                        args: vec![],
                    };
                }
            }
            "ref" | "mutref" | "deref" => {
                // Strip ownership operators — Elevate handles this automatically
                if let Some(arg) = args.first() {
                    return lower_expr(arg);
                }
            }
            _ => {}
        }
    }

    // Handle keyword arguments → struct literal
    if !keywords.is_empty() {
        if let Some(struct_name) = extract_struct_constructor_name(func) {
            // Struct construction: Student(name="Alice", age=20)
            let fields = keywords
                .iter()
                .filter_map(|kw| {
                    kw.arg.as_ref().map(|name| e::StructLiteralField {
                        name: name.clone(),
                        value: lower_expr(&kw.value),
                    })
                })
                .collect();
            return e::Expr::StructLiteral {
                path: struct_name,
                fields,
            };
        }
    }

    // Regular call
    let lowered_args: Vec<e::Expr> = args.iter().map(lower_expr).collect();
    e::Expr::Call {
        callee: Box::new(lower_expr(func)),
        args: lowered_args,
    }
}

fn lower_range_call(args: &[q::QuicheExpr]) -> e::Expr {
    match args.len() {
        0 => e::Expr::Range {
            start: None,
            end: None,
            inclusive: false,
        },
        1 => e::Expr::Range {
            start: Some(Box::new(e::Expr::Int(0))),
            end: Some(Box::new(lower_expr(&args[0]))),
            inclusive: false,
        },
        _ => e::Expr::Range {
            start: Some(Box::new(lower_expr(&args[0]))),
            end: Some(Box::new(lower_expr(&args[1]))),
            inclusive: false,
        },
    }
}

fn lower_print_call(args: &[q::QuicheExpr]) -> e::Expr {
    // print(x) → println!("{}", x) or println!("{}", format!(...))
    let lowered_args: Vec<e::Expr> = if args.is_empty() {
        vec![e::Expr::String(String::new())]
    } else {
        // If single string arg, use it directly; otherwise format
        match &args[0] {
            q::QuicheExpr::FString(parts) => {
                // Turn f-string into format! args
                let (fmt, fmt_args) = build_format_string(parts);
                let mut all_args = vec![e::Expr::String(fmt)];
                all_args.extend(fmt_args);
                return e::Expr::MacroCall {
                    path: vec!["println".to_string()],
                    args: all_args,
                };
            }
            _ => {
                vec![e::Expr::String("{}".to_string()), lower_expr(&args[0])]
            }
        }
    };
    e::Expr::MacroCall {
        path: vec!["println".to_string()],
        args: lowered_args,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// F-strings → format!()
// ─────────────────────────────────────────────────────────────────────────

fn lower_fstring(parts: &[q::FStringPart]) -> e::Expr {
    let (fmt, args) = build_format_string(parts);
    let mut all_args = vec![e::Expr::String(fmt)];
    all_args.extend(args);
    e::Expr::MacroCall {
        path: vec!["format".to_string()],
        args: all_args,
    }
}

fn build_format_string(parts: &[q::FStringPart]) -> (String, Vec<e::Expr>) {
    let mut fmt = String::new();
    let mut args = Vec::new();
    for part in parts {
        match part {
            q::FStringPart::Literal(s) => {
                // Escape curly braces in literal parts
                fmt.push_str(&s.replace('{', "{{").replace('}', "}}"));
            }
            q::FStringPart::Replacement {
                value,
                debug: _,
                conversion: _,
                format_spec,
            } => {
                if let Some(spec) = format_spec {
                    fmt.push_str(&format!("{{:{spec}}}"));
                } else {
                    fmt.push_str("{}");
                }
                args.push(lower_expr(value));
            }
        }
    }
    (fmt, args)
}

// ─────────────────────────────────────────────────────────────────────────
// List comprehension → .iter().map().collect()
// ─────────────────────────────────────────────────────────────────────────

fn lower_list_comp(element: &q::QuicheExpr, generators: &[q::Comprehension]) -> e::Expr {
    if generators.is_empty() {
        return e::Expr::Array(vec![]);
    }

    let generator = &generators[0];
    let iter_expr = lower_expr(&generator.iter);
    let var = extract_name(&generator.target).unwrap_or_else(|| "x".to_string());

    // Build closure for map: |var| element
    let map_closure = e::Expr::Closure {
        params: vec![e::Param {
            name: var,
            ty: e::Type {
                path: vec!["_".to_string()],
                args: vec![],
                trait_bounds: vec![],
            },
        }],
        return_type: None,
        body: e::Block {
            statements: vec![e::Stmt::Return(Some(lower_expr(element)))],
        },
    };

    // iter.iter().map(closure).collect()
    let iter_call = e::Expr::Call {
        callee: Box::new(e::Expr::Field {
            base: Box::new(iter_expr),
            field: "iter".to_string(),
        }),
        args: vec![],
    };
    let map_call = e::Expr::Call {
        callee: Box::new(e::Expr::Field {
            base: Box::new(iter_call),
            field: "map".to_string(),
        }),
        args: vec![map_closure],
    };
    e::Expr::Call {
        callee: Box::new(e::Expr::Field {
            base: Box::new(map_call),
            field: "collect".to_string(),
        }),
        args: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Dict comprehension → HashMap::from_iter(iter.into_iter().map(|v| (k, v)))
// ─────────────────────────────────────────────────────────────────────────

fn lower_dict_comp(
    key: &q::QuicheExpr,
    value: &q::QuicheExpr,
    generators: &[q::Comprehension],
) -> e::Expr {
    if generators.is_empty() {
        return e::Expr::Array(vec![]);
    }

    let generator = &generators[0];
    let iter_expr = lower_expr(&generator.iter);
    let var = extract_name(&generator.target).unwrap_or_else(|| "x".to_string());

    // Build closure: |var| (key, value)
    let map_closure = e::Expr::Closure {
        params: vec![e::Param {
            name: var,
            ty: e::Type {
                path: vec!["_".to_string()],
                args: vec![],
                trait_bounds: vec![],
            },
        }],
        return_type: None,
        body: e::Block {
            statements: vec![e::Stmt::Return(Some(e::Expr::Tuple(vec![
                lower_expr(key),
                lower_expr(value),
            ])))],
        },
    };

    let iter_call = e::Expr::Call {
        callee: Box::new(e::Expr::Field {
            base: Box::new(iter_expr),
            field: "into_iter".to_string(),
        }),
        args: vec![],
    };
    let map_call = e::Expr::Call {
        callee: Box::new(e::Expr::Field {
            base: Box::new(iter_call),
            field: "map".to_string(),
        }),
        args: vec![map_closure],
    };

    // Use HashMap::from_iter() instead of .collect() so Elevate's type
    // inference correctly resolves the result as HashMap<K, V>.
    e::Expr::Call {
        callee: Box::new(e::Expr::Path(vec![
            "HashMap".to_string(),
            "from_iter".to_string(),
        ])),
        args: vec![map_call],
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Subscript
// ─────────────────────────────────────────────────────────────────────────

fn lower_subscript(value: &q::QuicheExpr, slice: &q::QuicheExpr) -> e::Expr {
    // Check if it's a slice
    if let q::QuicheExpr::Slice {
        lower: lo,
        upper: hi,
        step: _,
    } = slice
    {
        // value[lo..hi] → value[lo..hi]
        let range = e::Expr::Range {
            start: lo.as_ref().map(|e| Box::new(lower_expr(e))),
            end: hi.as_ref().map(|e| Box::new(lower_expr(e))),
            inclusive: false,
        };
        return e::Expr::Index {
            base: Box::new(lower_expr(value)),
            index: Box::new(range),
        };
    }
    e::Expr::Index {
        base: Box::new(lower_expr(value)),
        index: Box::new(lower_expr(slice)),
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Lambda
// ─────────────────────────────────────────────────────────────────────────

fn lower_lambda(
    args: &[q::LambdaArg],
    return_type: &Option<Box<q::QuicheExpr>>,
    body: &q::LambdaBody,
) -> e::Expr {
    let params = args
        .iter()
        .map(|a| e::Param {
            name: a.name.clone(),
            ty: a
                .ty
                .as_ref()
                .map(|t| lower_type_expr(t))
                .unwrap_or_else(|| e::Type {
                    path: vec!["_".to_string()],
                    args: vec![],
                    trait_bounds: vec![],
                }),
        })
        .collect();

    let ret_ty = return_type.as_ref().map(|rt| lower_type_expr(rt));

    let block = match body {
        q::LambdaBody::Expr(expr) => e::Block {
            statements: vec![e::Stmt::Return(Some(lower_expr(expr)))],
        },
        q::LambdaBody::Block(stmts) => lower_body(stmts),
    };

    e::Expr::Closure {
        params,
        return_type: ret_ty,
        body: block,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// If expression
// ─────────────────────────────────────────────────────────────────────────

fn lower_if_expr(test: &q::QuicheExpr, body: &q::QuicheExpr, orelse: &q::QuicheExpr) -> e::Expr {
    // Python: body if test else orelse
    // Elevate: match test { true => body; false => orelse; }
    e::Expr::Match {
        scrutinee: Box::new(lower_expr(test)),
        arms: vec![
            e::MatchArm {
                pattern: e::Pattern::Bool(true),
                guard: None,
                value: lower_expr(body),
            },
            e::MatchArm {
                pattern: e::Pattern::Wildcard,
                guard: None,
                value: lower_expr(orelse),
            },
        ],
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Cast
// ─────────────────────────────────────────────────────────────────────────

fn lower_cast(expr: &q::QuicheExpr, target_type: &q::QuicheExpr) -> e::Expr {
    // Emit `as` casts as MacroCall to a special __quiche_as! macro
    // which will be defined as: macro_rules! __quiche_as { ($e:expr, $t:ty) => { $e as $t } }
    let type_name = extract_type_name(target_type);
    e::Expr::MacroCall {
        path: vec!["__quiche_as".to_string()],
        args: vec![lower_expr(expr), e::Expr::Path(vec![type_name])],
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Match arms + patterns
// ─────────────────────────────────────────────────────────────────────────

fn lower_match_arm(case: &q::MatchCase) -> e::MatchArm {
    // Match case body: use last expression as value, rest as statements
    let body_stmts: Vec<e::Stmt> = case.body.iter().filter_map(|s| lower_stmt(s)).collect();
    let value = if body_stmts.is_empty() {
        e::Expr::Tuple(vec![]) // unit
    } else if body_stmts.len() == 1 {
        // Single statement — if it's a return or expr, extract
        match &body_stmts[0] {
            e::Stmt::Return(Some(val)) => val.clone(),
            e::Stmt::Expr(val) => val.clone(),
            _ => e::Expr::Tuple(vec![]),
        }
    } else {
        // Multiple statements — wrap in block (Elevate doesn't have block exprs,
        // so emit the last one as value and rest via rustblock)
        e::Expr::Tuple(vec![]) // placeholder
    };

    e::MatchArm {
        pattern: lower_pattern(&case.pattern),
        guard: case.guard.as_ref().map(|g| lower_expr(g)),
        value,
    }
}

fn lower_pattern(pat: &q::Pattern) -> e::Pattern {
    match pat {
        q::Pattern::MatchValue(expr) => match expr.as_ref() {
            q::QuicheExpr::Constant(q::Constant::Int(i)) => e::Pattern::Int(*i),
            q::QuicheExpr::Constant(q::Constant::Bool(b)) => e::Pattern::Bool(*b),
            q::QuicheExpr::Constant(q::Constant::Str(s)) => e::Pattern::String(s.clone()),
            q::QuicheExpr::Name(n) => e::Pattern::Binding(n.clone()),
            q::QuicheExpr::Attribute { value, attr } => {
                // Enum variant like Color.Red → Color::Red
                let base = extract_name(value).unwrap_or_default();
                e::Pattern::Variant {
                    path: vec![base, attr.clone()],
                    payload: None,
                }
            }
            _ => e::Pattern::Wildcard,
        },
        q::Pattern::MatchAs { pattern, name } => {
            if let Some(n) = name {
                if pattern.is_none() {
                    // _ or just a variable binding
                    if n == "_" {
                        e::Pattern::Wildcard
                    } else {
                        e::Pattern::Binding(n.clone())
                    }
                } else {
                    e::Pattern::BindingAt {
                        name: n.clone(),
                        pattern: Box::new(
                            pattern
                                .as_ref()
                                .map(|p| lower_pattern(p))
                                .unwrap_or(e::Pattern::Wildcard),
                        ),
                    }
                }
            } else {
                e::Pattern::Wildcard
            }
        }
        q::Pattern::MatchClass(mc) => {
            // ClassName(field=pattern, ...)
            let path_name = extract_name(&mc.cls).unwrap_or_default();
            let path: Vec<String> = path_name.split("::").map(String::from).collect();

            if mc.patterns.is_empty() && mc.kwd_patterns.is_empty() {
                e::Pattern::Variant {
                    path,
                    payload: None,
                }
            } else if !mc.kwd_patterns.is_empty() {
                // Named fields
                let fields = mc
                    .kwd_attrs
                    .iter()
                    .zip(mc.kwd_patterns.iter())
                    .map(|(name, pat)| e::PatternField {
                        name: name.clone(),
                        pattern: lower_pattern(pat),
                    })
                    .collect();
                e::Pattern::Struct {
                    path,
                    fields,
                    has_rest: false,
                }
            } else {
                // Positional patterns → tuple
                let payload = if mc.patterns.len() == 1 {
                    lower_pattern(&mc.patterns[0])
                } else {
                    e::Pattern::Tuple(mc.patterns.iter().map(|p| lower_pattern(p)).collect())
                };
                e::Pattern::Variant {
                    path,
                    payload: Some(Box::new(payload)),
                }
            }
        }
        q::Pattern::MatchSequence(pats) => {
            e::Pattern::Tuple(pats.iter().map(|p| lower_pattern(p)).collect())
        }
        q::Pattern::MatchOr(pats) => {
            e::Pattern::Or(pats.iter().map(|p| lower_pattern(p)).collect())
        }
        q::Pattern::MatchSingleton(c) => match c {
            q::Constant::Int(i) => e::Pattern::Int(*i),
            q::Constant::Bool(b) => e::Pattern::Bool(*b),
            q::Constant::Str(s) => e::Pattern::String(s.clone()),
            _ => e::Pattern::Wildcard,
        },
        q::Pattern::MatchStar(name) => name
            .as_ref()
            .map(|n| e::Pattern::Binding(n.clone()))
            .unwrap_or(e::Pattern::Wildcard),
        q::Pattern::MatchMapping { .. } => e::Pattern::Wildcard, // TODO
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Type expressions → Elevate Type
// ─────────────────────────────────────────────────────────────────────────

fn lower_type_expr(expr: &q::QuicheExpr) -> e::Type {
    match expr {
        q::QuicheExpr::Name(n) => {
            let mapped = match n.as_str() {
                "int" => "i64",
                "float" => "f64",
                "str" | "String" => "String",
                "bool" => "bool",
                "List" | "Vec" => "Vec",
                "Dict" | "HashMap" => "HashMap",
                other => other,
            };
            e::Type {
                path: vec![mapped.to_string()],
                args: vec![],
                trait_bounds: vec![],
            }
        }
        q::QuicheExpr::Subscript { value, slice } => {
            // Vec[i32] → Vec<i32>
            let mut base = lower_type_expr(value);
            match slice.as_ref() {
                q::QuicheExpr::Tuple(elts) => {
                    base.args = elts.iter().map(|e| lower_type_expr(e)).collect();
                }
                _ => {
                    base.args = vec![lower_type_expr(slice)];
                }
            }
            base
        }
        q::QuicheExpr::Attribute { value, attr } => {
            // std.collections.HashMap → std::collections::HashMap
            let mut path = extract_path(value);
            path.push(attr.clone());
            e::Type {
                path,
                args: vec![],
                trait_bounds: vec![],
            }
        }
        _ => e::Type {
            path: vec!["_".to_string()],
            args: vec![],
            trait_bounds: vec![],
        },
    }
}

fn parse_type_string(s: &str) -> e::Type {
    let mapped = match s {
        "int" => "i64",
        "float" => "f64",
        "str" => "String",
        "bool" => "bool",
        _ => s,
    };
    e::Type {
        path: vec![mapped.to_string()],
        args: vec![],
        trait_bounds: vec![],
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────

fn extract_name(expr: &q::QuicheExpr) -> Option<String> {
    match expr {
        q::QuicheExpr::Name(n) => Some(n.clone()),
        q::QuicheExpr::Attribute { value, attr } => {
            let base = extract_name(value)?;
            Some(format!("{base}::{attr}"))
        }
        _ => None,
    }
}

fn extract_path(expr: &q::QuicheExpr) -> Vec<String> {
    match expr {
        q::QuicheExpr::Name(n) => vec![n.clone()],
        q::QuicheExpr::Attribute { value, attr } => {
            let mut path = extract_path(value);
            path.push(attr.clone());
            path
        }
        _ => vec![],
    }
}

fn extract_type_name(expr: &q::QuicheExpr) -> String {
    match expr {
        q::QuicheExpr::Name(n) => n.clone(),
        _ => "unknown".to_string(),
    }
}

fn extract_struct_constructor_name(func: &q::QuicheExpr) -> Option<Vec<String>> {
    match func {
        q::QuicheExpr::Name(n) => {
            // First char uppercase → struct constructor
            if n.starts_with(|c: char| c.is_uppercase()) {
                Some(vec![n.clone()])
            } else {
                None
            }
        }
        _ => None,
    }
}
