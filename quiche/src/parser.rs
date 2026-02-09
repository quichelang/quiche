//! Quiche Parser — converts Quiche tokens directly to Elevate AST.
//!
//! This is a hand-written recursive-descent parser that reuses the Quiche
//! lexer and produces `elevate::ast::Module` with zero intermediate AST.
#![allow(clippy::unwrap_used)]

use crate::lexer::{Keyword, LexError, Lexer, Token, TokenKind};
use elevate::ast as e;

// ─────────────────────────────────────────────────────────────────────────────
// Parser Error
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at {}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self {
        ParseError {
            message: e.message,
            line: e.line,
            column: e.column,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    peeked: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(source).map_err(|e| ParseError {
            message: e,
            line: 1,
            column: 1,
        })?;
        let current = lexer.next_token()?;
        Ok(Parser {
            lexer,
            current,
            peeked: None,
        })
    }

    fn kind(&self) -> &TokenKind {
        &self.current.kind
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.kind()) == std::mem::discriminant(kind)
    }

    fn check_kw(&self, kw: Keyword) -> bool {
        matches!(self.kind(), TokenKind::Keyword(k) if *k == kw)
    }

    fn advance(&mut self) -> Result<Token, ParseError> {
        let cur = std::mem::replace(
            &mut self.current,
            if let Some(p) = self.peeked.take() {
                p
            } else {
                self.lexer.next_token()?
            },
        );
        Ok(cur)
    }

    fn peek(&mut self) -> Result<&Token, ParseError> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        if self.check(kind) {
            self.advance()
        } else {
            Err(self.error(format!("expected {:?}, got {:?}", kind, self.kind())))
        }
    }

    fn expect_kw(&mut self, kw: Keyword) -> Result<Token, ParseError> {
        if self.check_kw(kw) {
            self.advance()
        } else {
            Err(self.error(format!("expected '{:?}', got {:?}", kw, self.kind())))
        }
    }

    fn eat(&mut self, kind: &TokenKind) -> Result<bool, ParseError> {
        if self.check(kind) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            line: self.current.line,
            column: self.current.column,
        }
    }

    fn skip_newlines(&mut self) -> Result<(), ParseError> {
        while matches!(self.kind(), TokenKind::Newline | TokenKind::Comment(_)) {
            self.advance()?;
        }
        Ok(())
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.kind().clone() {
            TokenKind::Ident(name) => {
                self.advance()?;
                Ok(name)
            }
            _ => Err(self.error(format!("expected identifier, got {:?}", self.kind()))),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Module
    // ─────────────────────────────────────────────────────────────────────────

    pub fn parse_module(&mut self) -> Result<e::Module, ParseError> {
        let mut items = Vec::new();
        self.skip_newlines()?;

        while !matches!(self.kind(), TokenKind::Eof) {
            match self.parse_item()? {
                Some(item) => items.push(item),
                None => {} // skip non-item statements at module level
            }
            self.skip_newlines()?;
        }

        Ok(e::Module { items })
    }

    fn parse_item(&mut self) -> Result<Option<e::Item>, ParseError> {
        // Handle decorators
        let decorators = self.parse_decorators()?;

        match self.kind() {
            TokenKind::Keyword(Keyword::Def) => {
                Ok(Some(e::Item::Function(self.parse_function_def()?)))
            }
            TokenKind::Keyword(Keyword::Class) => self.parse_class_def(decorators),
            TokenKind::Keyword(Keyword::From) => self.parse_from_import(),
            TokenKind::Keyword(Keyword::Import) => {
                self.parse_bare_import()?;
                Ok(None)
            }
            _ => {
                // Top-level expression or assignment — skip for now
                self.parse_stmt()?;
                Ok(None)
            }
        }
    }

    fn parse_decorators(&mut self) -> Result<Vec<String>, ParseError> {
        let mut decs = Vec::new();
        while self.check(&TokenKind::At) {
            self.advance()?;
            let name = self.expect_ident()?;
            // Skip decorator arguments like @impl(Trait)
            if self.check(&TokenKind::LParen) {
                self.advance()?;
                let mut depth = 1;
                while depth > 0 {
                    match self.kind() {
                        TokenKind::LParen => {
                            depth += 1;
                            self.advance()?;
                        }
                        TokenKind::RParen => {
                            depth -= 1;
                            self.advance()?;
                        }
                        TokenKind::Eof => {
                            return Err(self.error("unexpected EOF in decorator".into()));
                        }
                        _ => {
                            self.advance()?;
                        }
                    }
                }
                decs.push(name);
            } else {
                decs.push(name);
            }
            self.skip_newlines()?;
        }
        Ok(decs)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Imports: from X.Y import Z → RustUse { path: ["X", "Y", "Z"] }
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_from_import(&mut self) -> Result<Option<e::Item>, ParseError> {
        self.expect_kw(Keyword::From)?;
        let mut module_path = vec![self.expect_ident()?];
        while self.eat(&TokenKind::Dot)? {
            module_path.push(self.expect_ident()?);
        }
        self.expect_kw(Keyword::Import)?;
        let name = self.expect_ident()?;
        module_path.push(name);

        // Consume optional "as alias" — we ignore aliases for now
        if self.check_kw(Keyword::As) {
            self.advance()?;
            self.expect_ident()?;
        }
        Ok(Some(e::Item::RustUse(e::RustUse { path: module_path })))
    }

    fn parse_bare_import(&mut self) -> Result<(), ParseError> {
        self.expect_kw(Keyword::Import)?;
        self.expect_ident()?;
        while self.eat(&TokenKind::Dot)? {
            self.expect_ident()?;
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Functions
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_function_def(&mut self) -> Result<e::FunctionDef, ParseError> {
        self.expect_kw(Keyword::Def)?;
        let name = self.expect_ident()?;

        // Type params [T, U]
        let type_params = self.parse_type_params()?;

        // Params
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        // Return type
        let return_type = if self.eat(&TokenKind::Arrow)? {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        Ok(e::FunctionDef {
            visibility: e::Visibility::Public,
            name,
            type_params,
            params,
            return_type,
            body,
        })
    }

    fn parse_type_params(&mut self) -> Result<Vec<e::GenericParam>, ParseError> {
        if !self.eat(&TokenKind::LBracket)? {
            return Ok(vec![]);
        }
        let mut params = Vec::new();
        loop {
            if self.check(&TokenKind::RBracket) {
                break;
            }
            let name = self.expect_ident()?;
            params.push(e::GenericParam {
                name,
                bounds: vec![],
            });
            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(params)
    }

    fn parse_params(&mut self) -> Result<Vec<e::Param>, ParseError> {
        let mut params = Vec::new();
        while !self.check(&TokenKind::RParen) {
            let name = self.expect_ident()?;
            // Skip `self` as a param — Elevate handles it differently
            if name == "self" {
                // Optional type annotation
                if self.eat(&TokenKind::Colon)? {
                    self.parse_type()?; // consume and discard
                }
                if !self.eat(&TokenKind::Comma)? {
                    break;
                }
                continue;
            }
            let ty = if self.eat(&TokenKind::Colon)? {
                self.parse_type()?
            } else {
                // No type annotation — inferred
                e::Type {
                    path: vec!["_".into()],
                    args: vec![],
                    trait_bounds: vec![],
                }
            };
            params.push(e::Param { name, ty });
            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        Ok(params)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Types
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<e::Type, ParseError> {
        let name = self.expect_ident()?;
        let mut path = vec![name];

        // Dotted path: std.collections.HashMap → ["std", "collections", "HashMap"]
        while self.eat(&TokenKind::Dot)? {
            path.push(self.expect_ident()?);
        }

        // Generic args: Vec[i32] or HashMap[String, i32]
        let args = if self.eat(&TokenKind::LBracket)? {
            let mut args = Vec::new();
            loop {
                if self.check(&TokenKind::RBracket) {
                    break;
                }
                args.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma)? {
                    break;
                }
            }
            self.expect(&TokenKind::RBracket)?;
            args
        } else {
            vec![]
        };

        Ok(e::Type {
            path,
            args,
            trait_bounds: vec![],
        })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Class → Struct / Enum / Trait / Impl
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_class_def(&mut self, decorators: Vec<String>) -> Result<Option<e::Item>, ParseError> {
        self.expect_kw(Keyword::Class)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params()?;

        // Base class: class Foo(Struct): / class Foo(Enum): / class Foo(Trait):
        let base = if self.eat(&TokenKind::LParen)? {
            let base = self.expect_ident()?;
            self.expect(&TokenKind::RParen)?;
            Some(base)
        } else {
            None
        };

        self.expect(&TokenKind::Colon)?;
        let body_stmts = self.parse_block()?;

        // Check for @impl decorator
        if decorators.iter().any(|d| d == "impl") {
            let methods = self.extract_methods(body_stmts, &name);
            return Ok(Some(e::Item::Impl(e::ImplBlock {
                target: name,
                trait_target: None, // TODO: extract trait from decorator args
                methods,
            })));
        }

        match base.as_deref() {
            Some("Struct") => {
                let fields = self.extract_fields(body_stmts.clone());
                let methods = self.extract_methods(body_stmts, &name);
                let mut items = vec![e::Item::Struct(e::StructDef {
                    visibility: e::Visibility::Public,
                    name: name.clone(),
                    fields,
                })];
                if !methods.is_empty() {
                    items.push(e::Item::Impl(e::ImplBlock {
                        target: name,
                        trait_target: None,
                        methods,
                    }));
                }
                // Return first item; for multi-item, we'd need a different approach.
                // For now return struct only — methods come via impl block.
                Ok(items.into_iter().next())
            }
            Some("Enum") | Some("Type") => {
                let variants = self.extract_variants(body_stmts);
                Ok(Some(e::Item::Enum(e::EnumDef {
                    visibility: e::Visibility::Public,
                    name,
                    variants,
                })))
            }
            Some("Trait") => {
                let methods = self.extract_trait_methods(body_stmts);
                Ok(Some(e::Item::Trait(e::TraitDef {
                    visibility: e::Visibility::Public,
                    name,
                    supertraits: vec![],
                    methods,
                })))
            }
            _ => {
                // Plain class → struct + impl
                let fields = self.extract_fields(body_stmts.clone());
                let methods = self.extract_methods(body_stmts, &name);
                let mut result = vec![e::Item::Struct(e::StructDef {
                    visibility: e::Visibility::Public,
                    name: name.clone(),
                    fields,
                })];
                if !methods.is_empty() {
                    result.push(e::Item::Impl(e::ImplBlock {
                        target: name,
                        trait_target: None,
                        methods,
                    }));
                }
                Ok(result.into_iter().next())
            }
        }
    }

    fn extract_fields(&self, body: e::Block) -> Vec<e::Field> {
        let mut fields = Vec::new();
        for stmt in body.statements {
            if let e::Stmt::Const(c) = stmt {
                if let Some(ty) = c.ty {
                    fields.push(e::Field { name: c.name, ty });
                }
            }
        }
        fields
    }

    fn extract_variants(&self, body: e::Block) -> Vec<e::EnumVariant> {
        let mut variants = Vec::new();
        for stmt in body.statements {
            if let e::Stmt::Assign {
                target: e::AssignTarget::Path(name),
                value,
                ..
            } = stmt
            {
                let payload = match value {
                    e::Expr::Tuple(elems) => elems
                        .into_iter()
                        .filter_map(|e| {
                            if let e::Expr::Path(p) = e {
                                Some(e::Type {
                                    path: p,
                                    args: vec![],
                                    trait_bounds: vec![],
                                })
                            } else {
                                None
                            }
                        })
                        .collect(),
                    _ => vec![],
                };
                variants.push(e::EnumVariant { name, payload });
            }
        }
        variants
    }

    fn extract_methods(&self, body: e::Block, _target: &str) -> Vec<e::FunctionDef> {
        let mut methods = Vec::new();
        for stmt in body.statements {
            // Functions inside class body are parsed as Expr(Call) or similar —
            // but actually they're parsed as stmts. We need to handle this
            // differently. For now, we can't extract functions from Block statements.
            // TODO: the parser needs to handle nested function defs.
            let _ = stmt;
        }
        methods
    }

    fn extract_trait_methods(&self, body: e::Block) -> Vec<e::TraitMethodSig> {
        let mut sigs = Vec::new();
        let _ = body;
        sigs
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Block (indented body)
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<e::Block, ParseError> {
        let mut statements = Vec::new();

        // Inline single-statement (e.g., `if x: return 1`)
        if !self.check(&TokenKind::Newline) && !self.check(&TokenKind::Indent) {
            statements.push(self.parse_stmt()?);
            return Ok(e::Block { statements });
        }

        self.skip_newlines()?;
        self.expect(&TokenKind::Indent)?;

        while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::Eof) {
            self.skip_newlines()?;
            if self.check(&TokenKind::Dedent) || self.check(&TokenKind::Eof) {
                break;
            }
            statements.push(self.parse_stmt()?);
            self.skip_newlines()?;
        }

        if self.check(&TokenKind::Dedent) {
            self.advance()?;
        }

        Ok(e::Block { statements })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Statements
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<e::Stmt, ParseError> {
        match self.kind() {
            TokenKind::Keyword(Keyword::Return) => self.parse_return(),
            TokenKind::Keyword(Keyword::If) => self.parse_if_or_elif(),
            TokenKind::Keyword(Keyword::While) => self.parse_while(),
            TokenKind::Keyword(Keyword::For) => self.parse_for(),
            TokenKind::Keyword(Keyword::Match) => self.parse_match(),
            TokenKind::Keyword(Keyword::Pass) => {
                self.advance()?;
                Ok(e::Stmt::Expr(e::Expr::Tuple(vec![])))
            }
            TokenKind::Keyword(Keyword::Break) => {
                self.advance()?;
                Ok(e::Stmt::Break)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                self.advance()?;
                Ok(e::Stmt::Continue)
            }
            TokenKind::Keyword(Keyword::Def) => {
                // Nested function def — parse but skip (not supported in Elevate stmt pos)
                let _func = self.parse_function_def()?;
                Ok(e::Stmt::Expr(e::Expr::Tuple(vec![])))
            }
            _ => self.parse_expr_or_assign(),
        }
    }

    fn parse_return(&mut self) -> Result<e::Stmt, ParseError> {
        self.expect_kw(Keyword::Return)?;
        if matches!(
            self.kind(),
            TokenKind::Newline | TokenKind::Dedent | TokenKind::Eof
        ) {
            Ok(e::Stmt::Return(None))
        } else {
            Ok(e::Stmt::Return(Some(self.parse_expr()?)))
        }
    }

    fn parse_if_or_elif(&mut self) -> Result<e::Stmt, ParseError> {
        // Consume either `if` or `elif`
        self.advance()?;
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        let then_block = self.parse_block()?;

        // elif chains
        let mut else_block = None;

        if self.check_kw(Keyword::Elif) {
            // elif becomes nested if in else
            let elif_stmt = self.parse_if_or_elif()?;
            else_block = Some(e::Block {
                statements: vec![elif_stmt],
            });
        } else if self.check_kw(Keyword::Else) {
            self.advance()?;
            self.expect(&TokenKind::Colon)?;
            else_block = Some(self.parse_block()?);
        }

        Ok(e::Stmt::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while(&mut self) -> Result<e::Stmt, ParseError> {
        self.expect_kw(Keyword::While)?;
        let condition = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;
        Ok(e::Stmt::While { condition, body })
    }

    fn parse_for(&mut self) -> Result<e::Stmt, ParseError> {
        self.expect_kw(Keyword::For)?;
        let binding_name = self.expect_ident()?;
        let binding = e::DestructurePattern::Name(binding_name);
        self.expect_kw(Keyword::In)?;
        let iter = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;
        Ok(e::Stmt::For {
            binding,
            iter,
            body,
        })
    }

    fn parse_match(&mut self) -> Result<e::Stmt, ParseError> {
        self.expect_kw(Keyword::Match)?;
        let scrutinee = self.parse_expr()?;
        self.expect(&TokenKind::Colon)?;

        self.skip_newlines()?;
        self.expect(&TokenKind::Indent)?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::Dedent) && !self.check(&TokenKind::Eof) {
            self.skip_newlines()?;
            if self.check(&TokenKind::Dedent) {
                break;
            }
            self.expect_kw(Keyword::Case)?;
            let pattern = self.parse_pattern()?;
            let guard = if self.check_kw(Keyword::If) {
                self.advance()?;
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(&TokenKind::Colon)?;
            let body = self.parse_block()?;
            // Match arm value: wrap body in IIFE if multiple stmts
            let value = self.block_to_expr(body);
            arms.push(e::MatchArm {
                pattern,
                guard,
                value,
            });
            self.skip_newlines()?;
        }

        if self.check(&TokenKind::Dedent) {
            self.advance()?;
        }

        Ok(e::Stmt::Expr(e::Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
        }))
    }

    fn block_to_expr(&self, block: e::Block) -> e::Expr {
        let stmts = block.statements;
        if stmts.is_empty() {
            return e::Expr::Tuple(vec![]);
        }
        if stmts.len() == 1 {
            return match stmts.into_iter().next().unwrap() {
                e::Stmt::Return(Some(val)) => val,
                e::Stmt::Expr(val) => val,
                other => e::Expr::Closure {
                    params: vec![],
                    return_type: None,
                    body: e::Block {
                        statements: vec![other],
                    },
                },
            };
        }
        // Multiple statements → IIFE: (|| { stmts })()
        e::Expr::Call {
            callee: Box::new(e::Expr::Closure {
                params: vec![],
                return_type: None,
                body: e::Block { statements: stmts },
            }),
            args: vec![],
        }
    }

    fn parse_pattern(&mut self) -> Result<e::Pattern, ParseError> {
        match self.kind().clone() {
            TokenKind::Keyword(Keyword::True) => {
                self.advance()?;
                Ok(e::Pattern::Bool(true))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance()?;
                Ok(e::Pattern::Bool(false))
            }
            TokenKind::Int(n) => {
                let v = n;
                self.advance()?;
                Ok(e::Pattern::Int(v))
            }
            TokenKind::String(s) => {
                let v = s;
                self.advance()?;
                Ok(e::Pattern::String(v))
            }
            TokenKind::Ident(name) if name == "_" => {
                self.advance()?;
                Ok(e::Pattern::Wildcard)
            }
            TokenKind::Ident(name) => {
                self.advance()?;
                // Check for Enum variant: Name.Variant(payload) or Name(payload)
                if self.eat(&TokenKind::Dot)? {
                    let variant = self.expect_ident()?;
                    let payload = if self.eat(&TokenKind::LParen)? {
                        let inner = self.parse_pattern()?;
                        self.expect(&TokenKind::RParen)?;
                        Some(Box::new(inner))
                    } else {
                        None
                    };
                    Ok(e::Pattern::Variant {
                        path: vec![name, variant],
                        payload,
                    })
                } else if self.check(&TokenKind::LParen) {
                    self.advance()?;
                    let inner = if self.check(&TokenKind::RParen) {
                        None
                    } else {
                        Some(Box::new(self.parse_pattern()?))
                    };
                    self.expect(&TokenKind::RParen)?;
                    Ok(e::Pattern::Variant {
                        path: vec![name],
                        payload: inner,
                    })
                } else {
                    Ok(e::Pattern::Binding(name))
                }
            }
            _ => Err(self.error(format!("expected pattern, got {:?}", self.kind()))),
        }
    }

    fn parse_expr_or_assign(&mut self) -> Result<e::Stmt, ParseError> {
        let mut expr = self.parse_expr()?;

        // Handle bare comma-separated expressions: `a, *rest, b = items`
        // Creates an implicit Tuple for destructuring
        if self.check(&TokenKind::Comma) && !self.check(&TokenKind::Newline) {
            let mut elems = vec![expr];
            while self.eat(&TokenKind::Comma)? {
                if self.check(&TokenKind::Eq)
                    || self.check(&TokenKind::Newline)
                    || self.check(&TokenKind::Eof)
                {
                    break; // trailing comma
                }
                elems.push(self.parse_expr()?);
            }
            expr = e::Expr::Tuple(elems);
        }

        // Check for type annotation: `name: Type = value`
        if self.check(&TokenKind::Colon) {
            if let e::Expr::Path(ref path) = expr {
                if path.len() == 1 {
                    let name = path[0].clone();
                    self.advance()?; // consume ':'
                    let ty = self.parse_type()?;
                    if self.eat(&TokenKind::Eq)? {
                        let value = self.parse_expr()?;
                        return Ok(e::Stmt::Const(e::ConstDef {
                            visibility: e::Visibility::Private,
                            name,
                            ty: Some(ty),
                            value,
                            is_const: false,
                        }));
                    }
                    // Bare annotation without value (e.g., field decl in class body)
                    return Ok(e::Stmt::Const(e::ConstDef {
                        visibility: e::Visibility::Public,
                        name,
                        ty: Some(ty),
                        value: e::Expr::Tuple(vec![]),
                        is_const: false,
                    }));
                }
            }
        }

        // Assignment: `expr = value` or `expr += value`
        if self.check(&TokenKind::Eq) {
            self.advance()?;
            let value = self.parse_expr()?;

            // Check if LHS is a tuple containing *splat → destructure
            if let Some(pattern) = self.try_expr_to_destructure(&expr) {
                return Ok(e::Stmt::DestructureConst {
                    pattern,
                    value,
                    is_const: false,
                });
            }

            let target = self.expr_to_assign_target(expr)?;
            return Ok(e::Stmt::Assign {
                target,
                op: e::AssignOp::Assign,
                value,
            });
        }
        if self.check(&TokenKind::PlusEq) {
            self.advance()?;
            let value = self.parse_expr()?;
            let target = self.expr_to_assign_target(expr)?;
            return Ok(e::Stmt::Assign {
                target,
                op: e::AssignOp::AddAssign,
                value,
            });
        }
        // Built-in call transformations
        if let e::Expr::Call {
            ref callee,
            ref args,
        } = expr
        {
            if let e::Expr::Path(ref path) = **callee {
                if path.len() == 1 {
                    let name = path[0].as_str();
                    match name {
                        // rust("code") escape hatch — emit verbatim Rust
                        "rust" if args.len() == 1 => {
                            if let e::Expr::String(ref code) = args[0] {
                                return Ok(e::Stmt::RustBlock(code.clone()));
                            }
                        }
                        // print(a, b, c) → println!("{} {} {}", a, b, c)
                        "print" | "eprint" => {
                            let macro_name = if name == "print" {
                                "println"
                            } else {
                                "eprintln"
                            };
                            let fmt = (0..args.len()).map(|_| "{}").collect::<Vec<_>>().join(" ");
                            let mut macro_args = vec![e::Expr::String(fmt)];
                            macro_args.extend(args.iter().cloned());
                            return Ok(e::Stmt::Expr(e::Expr::MacroCall {
                                path: vec![macro_name.into()],
                                args: macro_args,
                            }));
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(e::Stmt::Expr(expr))
    }

    fn expr_to_assign_target(&self, expr: e::Expr) -> Result<e::AssignTarget, ParseError> {
        match expr {
            e::Expr::Path(p) if p.len() == 1 => {
                Ok(e::AssignTarget::Path(p.into_iter().next().unwrap()))
            }
            e::Expr::Field { base, field } => Ok(e::AssignTarget::Field { base, field }),
            e::Expr::Index { base, index } => Ok(e::AssignTarget::Index { base, index }),
            _ => Err(self.error("invalid assignment target".into())),
        }
    }

    /// Check if an expression is a tuple containing at least one *splat,
    /// and convert it to a DestructurePattern::Slice.
    fn try_expr_to_destructure(&self, expr: &e::Expr) -> Option<e::DestructurePattern> {
        // Single starred expression: *rest = items
        if Self::is_starred(expr) {
            let name = Self::starred_name(expr).unwrap();
            return Some(e::DestructurePattern::Slice {
                prefix: vec![],
                rest: Some(name),
                suffix: vec![],
            });
        }

        // Tuple containing starred: a, *rest, b = items
        let e::Expr::Tuple(elems) = expr else {
            return None;
        };

        // Must contain at least one starred element
        if !elems.iter().any(|e| Self::is_starred(e)) {
            return None;
        }

        let mut prefix = Vec::new();
        let mut rest: Option<String> = None;
        let mut suffix = Vec::new();
        let mut after_star = false;

        for elem in elems {
            if Self::is_starred(elem) {
                if rest.is_some() {
                    return None; // multiple stars not allowed
                }
                rest = Some(Self::starred_name(elem).unwrap());
                after_star = true;
            } else {
                let pat = self.expr_to_destructure_name(elem)?;
                if after_star {
                    suffix.push(pat);
                } else {
                    prefix.push(pat);
                }
            }
        }

        Some(e::DestructurePattern::Slice {
            prefix,
            rest,
            suffix,
        })
    }

    fn is_starred(expr: &e::Expr) -> bool {
        matches!(expr, e::Expr::MacroCall { path, .. } if path.len() == 1 && path[0] == "__star__")
    }

    fn starred_name(expr: &e::Expr) -> Option<String> {
        if let e::Expr::MacroCall { path, args } = expr {
            if path.len() == 1 && path[0] == "__star__" {
                if let Some(e::Expr::Path(p)) = args.first() {
                    return p.first().cloned();
                }
            }
        }
        None
    }

    fn expr_to_destructure_name(&self, expr: &e::Expr) -> Option<e::DestructurePattern> {
        match expr {
            e::Expr::Path(p) if p.len() == 1 => {
                if p[0] == "_" {
                    Some(e::DestructurePattern::Ignore)
                } else {
                    Some(e::DestructurePattern::Name(p[0].clone()))
                }
            }
            e::Expr::Tuple(elems) => {
                let mut items = Vec::new();
                for elem in elems {
                    items.push(self.expr_to_destructure_name(elem)?);
                }
                Some(e::DestructurePattern::Tuple(items))
            }
            _ => None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Expressions
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<e::Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<e::Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.check_kw(Keyword::Or) {
            self.advance()?;
            let right = self.parse_and_expr()?;
            left = e::Expr::Binary {
                op: e::BinaryOp::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<e::Expr, ParseError> {
        let mut left = self.parse_not_expr()?;
        while self.check_kw(Keyword::And) {
            self.advance()?;
            let right = self.parse_not_expr()?;
            left = e::Expr::Binary {
                op: e::BinaryOp::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<e::Expr, ParseError> {
        if self.check_kw(Keyword::Not) {
            self.advance()?;
            let expr = self.parse_not_expr()?;
            return Ok(e::Expr::Unary {
                op: e::UnaryOp::Not,
                expr: Box::new(expr),
            });
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<e::Expr, ParseError> {
        let mut left = self.parse_addition()?;
        loop {
            let op = match self.kind() {
                TokenKind::EqEq => e::BinaryOp::Eq,
                TokenKind::NotEq => e::BinaryOp::Ne,
                TokenKind::Lt => e::BinaryOp::Lt,
                TokenKind::LtEq => e::BinaryOp::Le,
                TokenKind::Gt => e::BinaryOp::Gt,
                TokenKind::GtEq => e::BinaryOp::Ge,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_addition()?;
            left = e::Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<e::Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.kind() {
                TokenKind::Plus => e::BinaryOp::Add,
                TokenKind::Minus => e::BinaryOp::Sub,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_multiplication()?;
            left = e::Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<e::Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.kind() {
                TokenKind::Star => e::BinaryOp::Mul,
                TokenKind::Slash => e::BinaryOp::Div,
                TokenKind::Percent => e::BinaryOp::Rem,
                _ => break,
            };
            self.advance()?;
            let right = self.parse_unary()?;
            left = e::Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<e::Expr, ParseError> {
        if self.check(&TokenKind::Minus) {
            self.advance()?;
            let expr = self.parse_unary()?;
            return Ok(e::Expr::Unary {
                op: e::UnaryOp::Neg,
                expr: Box::new(expr),
            });
        }
        // *name → starred expression (for destructuring: a, *rest = items)
        if self.check(&TokenKind::Star) {
            self.advance()?;
            let name = self.expect_ident()?;
            return Ok(e::Expr::MacroCall {
                path: vec!["__star__".into()],
                args: vec![e::Expr::Path(vec![name])],
            });
        }
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<e::Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.eat(&TokenKind::Dot)? {
                let field = self.expect_ident()?;
                expr = e::Expr::Field {
                    base: Box::new(expr),
                    field,
                };
            } else if self.check(&TokenKind::LParen) {
                self.advance()?;
                let args = self.parse_call_args()?;
                self.expect(&TokenKind::RParen)?;
                expr = e::Expr::Call {
                    callee: Box::new(expr),
                    args,
                };
            } else if self.check(&TokenKind::LBracket) {
                self.advance()?;
                let index = self.parse_expr()?;
                // Check for range: [start..end]
                if self.eat(&TokenKind::DotDot)? {
                    let end = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = e::Expr::Index {
                        base: Box::new(expr),
                        index: Box::new(e::Expr::Range {
                            start: Some(Box::new(index)),
                            end: Some(Box::new(end)),
                            inclusive: false,
                        }),
                    };
                } else {
                    self.expect(&TokenKind::RBracket)?;
                    expr = e::Expr::Index {
                        base: Box::new(expr),
                        index: Box::new(index),
                    };
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<e::Expr, ParseError> {
        match self.kind().clone() {
            TokenKind::Int(n) => {
                self.advance()?;
                Ok(e::Expr::Int(n))
            }
            TokenKind::Float(_f) => {
                // Elevate doesn't have Float in AST — emit as Int for now
                self.advance()?;
                Ok(e::Expr::Int(0)) // TODO: float support
            }
            TokenKind::String(s) => {
                self.advance()?;
                Ok(e::Expr::String(s))
            }
            TokenKind::Keyword(Keyword::True) => {
                self.advance()?;
                Ok(e::Expr::Bool(true))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance()?;
                Ok(e::Expr::Bool(false))
            }
            TokenKind::Keyword(Keyword::None) => {
                self.advance()?;
                Ok(e::Expr::Path(vec!["None".into()]))
            }
            TokenKind::Ident(name) => {
                self.advance()?;
                Ok(e::Expr::Path(vec![name]))
            }
            TokenKind::LParen => {
                self.advance()?;
                if self.check(&TokenKind::RParen) {
                    self.advance()?;
                    return Ok(e::Expr::Tuple(vec![]));
                }
                let expr = self.parse_expr()?;
                // Check for tuple: (a, b, ...)
                if self.eat(&TokenKind::Comma)? {
                    let mut elems = vec![expr];
                    while !self.check(&TokenKind::RParen) {
                        elems.push(self.parse_expr()?);
                        if !self.eat(&TokenKind::Comma)? {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen)?;
                    return Ok(e::Expr::Tuple(elems));
                }
                self.expect(&TokenKind::RParen)?;
                Ok(expr) // parenthesized expression
            }
            TokenKind::LBracket => {
                self.advance()?;
                let mut elems = Vec::new();
                while !self.check(&TokenKind::RBracket) {
                    elems.push(self.parse_expr()?);
                    if !self.eat(&TokenKind::Comma)? {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(e::Expr::Array(elems))
            }
            TokenKind::FString { content, .. } => {
                self.advance()?;
                // f-string → emit as format!() with the raw content
                Ok(e::Expr::MacroCall {
                    path: vec!["format".into()],
                    args: vec![e::Expr::String(content)],
                })
            }
            _ => Err(self.error(format!("expected expression, got {:?}", self.kind()))),
        }
    }

    fn parse_call_args(&mut self) -> Result<Vec<e::Expr>, ParseError> {
        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) {
            args.push(self.parse_expr()?);
            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        Ok(args)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

pub fn parse(source: &str) -> Result<e::Module, ParseError> {
    let mut parser = Parser::new(source)?;
    parser.parse_module()
}
