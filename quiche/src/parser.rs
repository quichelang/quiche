//! Quiche Parser — converts Quiche tokens directly to Elevate AST.
//!
//! This is a hand-written recursive-descent parser that reuses the Quiche
//! lexer and produces `elevate::ast::Module` with zero intermediate AST.
#![allow(clippy::unwrap_used)]

use crate::lexer::{Keyword, LexError, Lexer, Token, TokenKind};
use elevate::ast as e;
use elevate::diag::Span;
use std::collections::HashMap;

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

/// Represents a call argument — either positional or keyword (name=expr).
enum CallArg {
    Positional(e::Expr),
    Keyword(String, e::Expr),
}

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    peeked: Option<Token>,
    /// Maps struct names to their ordered field names (for positional construction)
    struct_fields: HashMap<String, Vec<String>>,
    /// Maps function names to their ordered parameter names (for kwarg reordering)
    fn_params: HashMap<String, Vec<String>>,
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
            current: current,
            peeked: None,
            struct_fields: HashMap::new(),
            fn_params: HashMap::new(),
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
            Err(self.error(format!("expected {}, got {}", kind, self.kind())))
        }
    }

    fn expect_kw(&mut self, kw: Keyword) -> Result<Token, ParseError> {
        if self.check_kw(kw) {
            self.advance()
        } else {
            Err(self.error(format!("expected '{kw:?}', got {}", self.kind())))
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

    /// Build an Elevate Span from a start byte offset to the current token.
    fn span_from(&self, start: usize) -> Option<Span> {
        Some(Span::new(start, self.current.start))
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
            _ => Err(self.error(format!("expected identifier, got {}", self.kind()))),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Module
    // ─────────────────────────────────────────────────────────────────────────

    pub fn parse_module(&mut self) -> Result<e::Module, ParseError> {
        let mut items = Vec::new();
        self.skip_newlines()?;

        while !matches!(self.kind(), TokenKind::Eof) {
            let parsed = self.parse_item()?;
            items.extend(parsed);
            self.skip_newlines()?;
        }

        Ok(e::Module { items })
    }

    fn parse_item(&mut self) -> Result<Vec<e::Item>, ParseError> {
        match self.kind() {
            TokenKind::Keyword(Keyword::Def) => {
                Ok(vec![e::Item::Function(self.parse_function_def()?)])
            }
            TokenKind::Keyword(Keyword::Type) => self.parse_type_def(),
            TokenKind::Keyword(Keyword::From) => Ok(self.parse_from_import()?),
            TokenKind::Keyword(Keyword::Import) => {
                self.parse_bare_import()?;
                Ok(vec![])
            }
            _ => {
                // Top-level expression or assignment — skip for now
                self.parse_stmt()?;
                Ok(vec![])
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Imports: from X.Y import Z → RustUse { tree: UseTree }
    // ─────────────────────────────────────────────────────────────────────────

    fn path_to_use_tree(path: Vec<String>) -> e::UseTree {
        let mut iter = path.into_iter().rev();
        let leaf = e::UseTree::Name(iter.next().unwrap());
        iter.fold(leaf, |next, segment| e::UseTree::Path {
            segment,
            next: Box::new(next),
        })
    }

    fn parse_from_import(&mut self) -> Result<Vec<e::Item>, ParseError> {
        self.expect_kw(Keyword::From)?;
        let mut module_path = vec![self.expect_ident()?];
        while self.eat(&TokenKind::Dot)? {
            module_path.push(self.expect_ident()?);
        }
        self.expect_kw(Keyword::Import)?;

        // Parse comma-separated names: from X.Y import A, B, C
        let mut items = Vec::new();
        loop {
            let name = self.expect_ident()?;
            let mut path = module_path.clone();
            path.push(name);

            // Consume optional "as alias" — we ignore aliases for now
            if self.check_kw(Keyword::As) {
                self.advance()?;
                self.expect_ident()?;
            }
            let tree = Self::path_to_use_tree(path);
            items.push(e::Item::RustUse(e::RustUse { tree, span: None }));

            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        Ok(items)
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
        let start_pos = self.current.start;
        self.expect_kw(Keyword::Def)?;
        let name = self.expect_ident()?;

        // Type params [T, U]
        let type_params = self.parse_type_params()?;

        // Params
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        // Register parameter names for kwarg reordering
        let param_names: Vec<String> = params
            .iter()
            .filter(|p| p.name != "self")
            .map(|p| p.name.clone())
            .collect();
        self.fn_params.insert(name.clone(), param_names);

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
            effect_row: None,
            body,
            span: self.span_from(start_pos),
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
            // Parse optional trait bounds: T: Display or T: Display + Debug
            let bounds = if self.eat(&TokenKind::Colon)? {
                let mut bounds = Vec::new();
                bounds.push(self.parse_type()?);
                while self.eat(&TokenKind::Plus)? {
                    bounds.push(self.parse_type()?);
                }
                bounds
            } else {
                vec![]
            };
            params.push(e::GenericParam { name, bounds });
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
            // Emit `self` as a param with type `Self` — Elevate's
            // type_from_ast_with_impl_self resolves Self → impl target type.
            if name == "self" {
                let ty = if self.eat(&TokenKind::Colon)? {
                    self.parse_type()?
                } else {
                    e::Type {
                        path: vec!["Self".into()],
                        args: vec![],
                        trait_bounds: vec![],
                    }
                };
                params.push(e::Param { name, ty });
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
        // Rewrite Quiche primitive type names to newtype names
        let name = match name.as_str() {
            "str" => "Str".into(),
            "list" => "List".into(),
            "dict" => "Dict".into(),
            _ => name,
        };
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

    /// Parse a single variant: `Name`, `Name(T1, T2)`, or `Name(x: T1, y: T2)`.
    fn parse_variant(&mut self) -> Result<e::EnumVariant, ParseError> {
        let name = self.expect_ident()?;

        if !self.eat(&TokenKind::LParen)? {
            return Ok(e::EnumVariant {
                name,
                fields: e::EnumVariantFields::Unit,
            });
        }

        // Empty parens: Name() → unit variant
        if self.eat(&TokenKind::RParen)? {
            return Ok(e::EnumVariant {
                name,
                fields: e::EnumVariantFields::Unit,
            });
        }

        // Peek ahead to distinguish named fields (ident : type) vs tuple (type).
        let is_named = matches!(self.kind(), TokenKind::Ident(_))
            && matches!(self.peek()?.kind, TokenKind::Colon);

        if is_named {
            let mut fields = Vec::new();
            loop {
                let field_name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let ty = self.parse_type()?;
                fields.push(e::Field {
                    name: field_name,
                    ty,
                });
                if !self.eat(&TokenKind::Comma)? {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
            Ok(e::EnumVariant {
                name,
                fields: e::EnumVariantFields::Named(fields),
            })
        } else {
            let mut types = Vec::new();
            loop {
                types.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma)? {
                    break;
                }
            }
            self.expect(&TokenKind::RParen)?;
            Ok(e::EnumVariant {
                name,
                fields: e::EnumVariantFields::Tuple(types),
            })
        }
    }

    /// Parse a `|`-separated variant list. Supports:
    ///
    ///   Inline:    `A | B(i32) | C`          (pipe as separator)
    ///   Inline:    `| A | B(i32) | C`        (optional leading pipe)
    ///   Multiline: `\n    | A\n    | B\n    | C`
    fn parse_variant_list(&mut self) -> Result<Vec<e::EnumVariant>, ParseError> {
        let mut variants = Vec::new();

        // Skip newlines and optional indent (multiline form)
        self.skip_newlines()?;
        let in_block = self.eat(&TokenKind::Indent)?;

        // Eat optional leading pipe, then parse first variant
        self.eat(&TokenKind::Pipe)?;
        variants.push(self.parse_variant()?);
        self.skip_newlines()?;

        // Continue with remaining | Variant pairs
        while self.eat(&TokenKind::Pipe)? {
            variants.push(self.parse_variant()?);
            self.skip_newlines()?;
        }

        if in_block && self.check(&TokenKind::Dedent) {
            self.advance()?;
        }

        Ok(Self::disambiguate_variants(variants))
    }

    /// If any variants share the same name, append the payload arity as a
    /// suffix to each duplicate. Unique names are left untouched.
    fn disambiguate_variants(variants: Vec<e::EnumVariant>) -> Vec<e::EnumVariant> {
        // Count occurrences of each name
        let mut counts: HashMap<String, usize> = HashMap::new();
        for v in &variants {
            *counts.entry(v.name.clone()).or_insert(0) += 1;
        }

        // Only rename if a name appears more than once
        variants
            .into_iter()
            .map(|mut v| {
                if counts.get(&v.name).copied().unwrap_or(0) > 1 {
                    let arity = match &v.fields {
                        e::EnumVariantFields::Unit => 0,
                        e::EnumVariantFields::Tuple(types) => types.len(),
                        e::EnumVariantFields::Named(fields) => fields.len(),
                    };
                    v.name = format!("{}__a{}", v.name, arity);
                }
                v
            })
            .collect()
    }

    // ─────────────────────────────────────────────────────────────────────────
    // type Keyword → Struct / Enum / Union
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse a `type` definition:
    ///
    ///   Struct: `type Point:\n    x: i32\n    y: i32`
    ///   Enum:   `type Color = | Red | Green | Blue(i32)`
    fn parse_type_def(&mut self) -> Result<Vec<e::Item>, ParseError> {
        let type_start = self.current.start;
        self.expect_kw(Keyword::Type)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_type_params()?;

        if self.eat(&TokenKind::Eq)? {
            // Union shorthand: `type Number = i64 | f64`
            // If first token is a lowercase identifier, treat as union types
            let is_union = match self.kind() {
                TokenKind::Ident(id) => id.starts_with(char::is_lowercase),
                _ => false,
            };

            if is_union {
                let mut union_types = vec![self.parse_type()?];
                while self.eat(&TokenKind::Pipe)? {
                    union_types.push(self.parse_type()?);
                }
                let variants = union_types
                    .into_iter()
                    .map(|ty| {
                        let variant_name = Self::type_to_variant_name(&ty);
                        e::EnumVariant {
                            name: variant_name,
                            fields: e::EnumVariantFields::Tuple(vec![ty]),
                        }
                    })
                    .collect();
                return Ok(vec![e::Item::Enum(e::EnumDef {
                    visibility: e::Visibility::Public,
                    name,
                    type_params,
                    variants,
                    span: self.span_from(type_start),
                })]);
            }

            // Enum variant list: `type Color = Red | Green | Blue(i32)`
            let variants = self.parse_variant_list()?;
            return Ok(vec![e::Item::Enum(e::EnumDef {
                visibility: e::Visibility::Public,
                name,
                type_params,
                variants,
                span: self.span_from(type_start),
            })]);
        }

        // ── Struct form: `type Name:\n    field: Type` ───────────────
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let fields: Vec<e::Field> = body
            .statements
            .iter()
            .filter_map(|s| {
                if let e::Stmt::Const(c) = s {
                    c.ty.as_ref().map(|ty| e::Field {
                        name: c.name.clone(),
                        ty: ty.clone(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Register field names for positional struct construction
        self.struct_fields.insert(
            name.clone(),
            fields.iter().map(|f| f.name.clone()).collect(),
        );

        Ok(vec![e::Item::Struct(e::StructDef {
            visibility: e::Visibility::Public,
            name,
            type_params,
            fields,
            span: self.span_from(type_start),
        })])
    }

    /// Convert a Type to a PascalCase variant name for union enum generation.
    /// Examples: `i64` → `I64`, `String` → `String`, `Vec[i32]` → `VecI32`
    fn type_to_variant_name(ty: &e::Type) -> String {
        let base = ty.path.last().unwrap_or(&String::new()).clone();
        let mut name = String::new();
        // Capitalize first letter
        let mut chars = base.chars();
        if let Some(first) = chars.next() {
            name.push(first.to_ascii_uppercase());
            name.extend(chars);
        }
        // Append generic args
        for arg in &ty.args {
            let arg_name = Self::type_to_variant_name(arg);
            name.push_str(&arg_name);
        }
        name
    }

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
            TokenKind::Keyword(Keyword::Assert) => self.parse_assert(),
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

    /// `assert expr` → `assert!(expr)`
    /// `assert expr, "msg"` → `assert!(expr, "{}", msg)`
    fn parse_assert(&mut self) -> Result<e::Stmt, ParseError> {
        self.expect_kw(Keyword::Assert)?;
        let condition = self.parse_expr()?;

        let mut args = vec![condition];

        // Check for optional message: `assert expr, "message"`
        if matches!(self.kind(), TokenKind::Comma) {
            self.advance()?; // consume comma
            let msg = self.parse_expr()?;
            args.push(e::Expr::String("{}".to_string()));
            args.push(msg);
        }

        Ok(e::Stmt::Expr(e::Expr::MacroCall {
            path: vec!["assert".to_string()],
            args,
        }))
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
            TokenKind::Keyword(Keyword::None) => {
                self.advance()?;
                Ok(e::Pattern::Variant {
                    path: vec!["None".to_string()],
                    payload: None,
                })
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
            _ => Err(self.error(format!("expected pattern, got {}", self.kind()))),
        }
    }

    fn parse_expr_or_assign(&mut self) -> Result<e::Stmt, ParseError> {
        let stmt_start = self.current.start;
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
                            span: self.span_from(stmt_start),
                        }));
                    }
                    // Bare annotation without value (e.g., field decl in class body)
                    return Ok(e::Stmt::Const(e::ConstDef {
                        visibility: e::Visibility::Public,
                        name,
                        ty: Some(ty),
                        value: e::Expr::Tuple(vec![]),
                        is_const: false,
                        span: self.span_from(stmt_start),
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
                            // Unwrap str() wrapping if present (smart strings wrap all literals)
                            let inner = match &args[0] {
                                e::Expr::String(code) => Some(code.clone()),
                                e::Expr::Call {
                                    callee,
                                    args: inner_args,
                                } if matches!(**callee, e::Expr::Path(ref p) if p.len() == 1 && p[0] == "str")
                                    && inner_args.len() == 1 =>
                                {
                                    if let e::Expr::String(code) = &inner_args[0] {
                                        Some(code.clone())
                                    } else {
                                        None
                                    }
                                }
                                _ => None,
                            };
                            if let Some(code) = inner {
                                return Ok(e::Stmt::RustBlock(code));
                            }
                        }
                        // print(a, b, c) → println!("{} {:?} {:?}", a, b, c)
                        // String literals use {} (no quotes), everything else uses {:?} (Debug)
                        "print" | "eprint" => {
                            let macro_name = if name == "print" {
                                "println"
                            } else {
                                "eprintln"
                            };
                            let fmt = args.iter().map(|_| "{}").collect::<Vec<_>>().join(" ");
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

        // If no starred element, try plain tuple destructure: a, b = expr
        if !elems.iter().any(|e| Self::is_starred(e)) {
            let mut items = Vec::new();
            for elem in elems {
                items.push(self.expr_to_destructure_name(elem)?);
            }
            return Some(e::DestructurePattern::Tuple(items));
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
        let body = self.parse_pipe_expr()?;
        // Python-style ternary: body if condition else orelse
        // Desugars to: match condition { true => body, _ => orelse }
        if self.check_kw(Keyword::If) {
            self.advance()?;
            let condition = self.parse_or_expr()?;
            self.expect_kw(Keyword::Else)?;
            let orelse = self.parse_expr()?; // right-associative: a if c1 else b if c2 else d
            return Ok(e::Expr::Match {
                scrutinee: Box::new(condition),
                arms: vec![
                    e::MatchArm {
                        pattern: e::Pattern::Bool(true),
                        guard: None,
                        value: body,
                    },
                    e::MatchArm {
                        pattern: e::Pattern::Wildcard,
                        guard: None,
                        value: orelse,
                    },
                ],
            });
        }
        Ok(body)
    }

    /// Pipe operator: `lhs |> f(args)` desugars to `f(lhs, args)`.
    /// Left-associative, lowest precedence among binary ops.
    fn parse_pipe_expr(&mut self) -> Result<e::Expr, ParseError> {
        let mut left = self.parse_or_expr()?;
        while self.check(&TokenKind::PipeRight) {
            self.advance()?;
            let rhs = self.parse_or_expr()?;
            left = Self::desugar_pipe(left, rhs)?;
        }
        Ok(left)
    }

    /// Transform `lhs |> rhs` into a function call:
    ///   - `Call { callee, args }` → `Call { callee, args: [lhs] + args }`
    ///   - `Path(f)` or `Field { .. }` → `Call { callee: rhs, args: [lhs] }`
    fn desugar_pipe(lhs: e::Expr, rhs: e::Expr) -> Result<e::Expr, ParseError> {
        match rhs {
            e::Expr::Call { callee, mut args } => {
                args.insert(0, lhs);
                Ok(e::Expr::Call { callee, args })
            }
            e::Expr::Path(_) | e::Expr::Field { .. } => Ok(e::Expr::Call {
                callee: Box::new(rhs),
                args: vec![lhs],
            }),
            // Allow piping into macro calls too
            e::Expr::MacroCall { path, mut args } => {
                args.insert(0, lhs);
                Ok(e::Expr::MacroCall { path, args })
            }
            _ => {
                // For any other expression, treat as a call with lhs as arg
                Ok(e::Expr::Call {
                    callee: Box::new(rhs),
                    args: vec![lhs],
                })
            }
        }
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
        // Rewrite string + chains to str(format!("{}{}", a, b))
        Ok(Self::rewrite_string_concat(left))
    }

    /// If `expr` is a chain of `Add` ops containing any string-like operand,
    /// flatten it into `str(format!("{}{}{}", a, b, c))`.
    fn rewrite_string_concat(expr: e::Expr) -> e::Expr {
        let mut operands = Vec::new();
        if !Self::flatten_add_chain(&expr, &mut operands) {
            return expr; // contains Sub — not a pure + chain
        }
        // Only rewrite actual + chains (2+ operands), not single expressions
        if operands.len() < 2 {
            return expr;
        }
        // Check if any operand looks string-like
        if !operands.iter().any(Self::is_string_expr) {
            return expr; // pure numeric, leave for Elevate
        }
        // Build format!("{}{}{}", a, b, c) wrapped in str()
        let fmt = operands.iter().map(|_| "{}").collect::<Vec<_>>().join("");
        let mut macro_args = vec![e::Expr::String(fmt)];
        macro_args.extend(operands);
        let format_call = e::Expr::MacroCall {
            path: vec!["format".into()],
            args: macro_args,
        };
        // Wrap in str() constructor
        e::Expr::Call {
            callee: Box::new(e::Expr::Path(vec!["str".into()])),
            args: vec![format_call],
        }
    }

    /// Flatten a left-associative Add chain into a vec of operands.
    /// Returns false if any Sub is encountered (not a pure + chain).
    fn flatten_add_chain(expr: &e::Expr, out: &mut Vec<e::Expr>) -> bool {
        if let e::Expr::Binary { op, left, right } = expr {
            match op {
                e::BinaryOp::Add => {
                    if !Self::flatten_add_chain(left, out) {
                        return false;
                    }
                    out.push(*right.clone());
                    true
                }
                e::BinaryOp::Sub => false,
                _ => {
                    out.push(expr.clone());
                    true
                }
            }
        } else {
            out.push(expr.clone());
            true
        }
    }

    /// Check if an expression is string-like (str() call, string literal, or format! macro).
    fn is_string_expr(expr: &e::Expr) -> bool {
        match expr {
            // Raw string literals
            e::Expr::String(_) => true,
            // str("literal") or str(expr)
            e::Expr::Call { callee, .. } => {
                matches!(**callee, e::Expr::Path(ref p) if p.len() == 1 && p[0] == "str")
            }
            // format!(...) — f-strings
            e::Expr::MacroCall { path, .. } => path.len() == 1 && path[0] == "format",
            _ => false,
        }
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
                // Static method heuristic: if base is a capitalized Path (type name)
                // and this is followed by '(' (a call), merge into path for `Type::method`.
                // e.g. Student.new(...) → Path(["Student", "new"]) → Student::new(...)
                if let e::Expr::Path(ref segments) = expr {
                    if let Some(first) = segments.first() {
                        if first.starts_with(|c: char| c.is_uppercase())
                            && self.check(&TokenKind::LParen)
                        {
                            let mut path = segments.clone();
                            path.push(field);
                            expr = e::Expr::Path(path);
                            continue;
                        }
                    }
                }
                // Quiche value semantics: .iter() → .into_iter()
                // so iterator chains yield T instead of &T, avoiding
                // double-reference issues in filter/any/all closures.
                let field = if field == "iter" {
                    "into_iter".into()
                } else {
                    field
                };
                expr = e::Expr::Field {
                    base: Box::new(expr),
                    field,
                };
            } else if self.check(&TokenKind::LParen) {
                self.advance()?;
                let call_args = self.parse_call_args_with_kwargs()?;
                self.expect(&TokenKind::RParen)?;

                // Check if this is a struct constructor call
                let is_struct_call = if let e::Expr::Path(ref path) = expr {
                    path.len() == 1 && self.struct_fields.contains_key(&path[0])
                } else {
                    false
                };

                if is_struct_call {
                    // Convert to StructLiteral
                    let path = if let e::Expr::Path(ref p) = expr {
                        p.clone()
                    } else {
                        unreachable!()
                    };
                    let field_names = self.struct_fields.get(&path[0]).unwrap().clone();

                    // Check if any args are keyword args
                    let has_kwargs = call_args
                        .iter()
                        .any(|a| matches!(a, CallArg::Keyword(_, _)));

                    let fields = if has_kwargs {
                        // Keyword construction: Point(x=5, y=5)
                        call_args
                            .into_iter()
                            .map(|arg| {
                                match arg {
                                    CallArg::Keyword(name, value) => {
                                        e::StructLiteralField { name, value }
                                    }
                                    CallArg::Positional(_) => {
                                        // Mixed positional+keyword not supported yet — treat as error
                                        // For now, skip positional in kwargs mode
                                        e::StructLiteralField {
                                            name: String::new(),
                                            value: e::Expr::Int(0),
                                        }
                                    }
                                }
                            })
                            .filter(|f| !f.name.is_empty())
                            .collect()
                    } else {
                        // Positional construction: Point(5, 5) → zip with field names
                        field_names
                            .iter()
                            .zip(call_args.into_iter())
                            .map(|(name, arg): (&String, CallArg)| {
                                let value = match arg {
                                    CallArg::Positional(expr) => expr,
                                    CallArg::Keyword(_, expr) => expr,
                                };
                                e::StructLiteralField {
                                    name: name.clone(),
                                    value,
                                }
                            })
                            .collect()
                    };

                    expr = e::Expr::StructLiteral { path, fields };
                    continue;
                }

                // Reorder kwargs for non-struct calls using fn_params
                let has_kwargs = call_args
                    .iter()
                    .any(|a| matches!(a, CallArg::Keyword(_, _)));
                let args: Vec<e::Expr> = if has_kwargs {
                    // Determine the function key for fn_params lookup
                    let fn_key = match &expr {
                        e::Expr::Path(path) if path.len() == 1 => Some(path[0].clone()),
                        e::Expr::Path(path) if path.len() == 2 => {
                            // Point.new(...) → Path(["Point", "new"]) → key "Point::new"
                            Some(format!("{}::{}", path[0], path[1]))
                        }
                        e::Expr::Field { base, field, .. } => {
                            if let e::Expr::Path(ref base_path) = **base {
                                if base_path.len() == 1 {
                                    Some(format!("{}::{}", base_path[0], field))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        _ => None,
                    };

                    if let Some(ref key) = fn_key {
                        if let Some(param_names) = self.fn_params.get(key) {
                            // Build a map of kwarg name → value
                            let mut kwarg_map: HashMap<String, e::Expr> = HashMap::new();
                            let mut positional = Vec::new();
                            for arg in call_args {
                                match arg {
                                    CallArg::Keyword(name, val) => {
                                        kwarg_map.insert(name, val);
                                    }
                                    CallArg::Positional(val) => {
                                        positional.push(val);
                                    }
                                }
                            }
                            // Reorder: fill positional first, then kwargs by param name order
                            let mut result = Vec::new();
                            let mut pos_idx = 0;
                            for pname in param_names {
                                if let Some(val) = kwarg_map.remove(pname) {
                                    result.push(val);
                                } else if pos_idx < positional.len() {
                                    result.push(positional[pos_idx].clone());
                                    pos_idx += 1;
                                }
                            }
                            result
                        } else {
                            // Function not known — pass in order
                            call_args
                                .into_iter()
                                .map(|a| match a {
                                    CallArg::Positional(e) => e,
                                    CallArg::Keyword(_, e) => e,
                                })
                                .collect()
                        }
                    } else {
                        call_args
                            .into_iter()
                            .map(|a| match a {
                                CallArg::Positional(e) => e,
                                CallArg::Keyword(_, e) => e,
                            })
                            .collect()
                    }
                } else {
                    // All positional — pass through
                    call_args
                        .into_iter()
                        .map(|a| match a {
                            CallArg::Positional(e) => e,
                            CallArg::Keyword(_, e) => e,
                        })
                        .collect()
                };

                // Convert str(x) → x.to_string()
                if let e::Expr::Path(ref path) = expr {
                    if path.len() == 1 && path[0] == "str" && args.len() == 1 {
                        let receiver = args.into_iter().next().unwrap();
                        expr = e::Expr::Call {
                            callee: Box::new(e::Expr::Field {
                                base: Box::new(receiver),
                                field: "to_string".into(),
                            }),
                            args: vec![],
                        };
                        continue;
                    }
                }

                // Convert range(end) → 0..end, range(start, end) → start..end
                if let e::Expr::Path(ref path) = expr {
                    if path.len() == 1 && path[0] == "range" {
                        match args.len() {
                            1 => {
                                expr = e::Expr::Range {
                                    start: Some(Box::new(e::Expr::Int(0))),
                                    end: Some(Box::new(args.into_iter().next().unwrap())),
                                    inclusive: false,
                                };
                                continue;
                            }
                            2 => {
                                let mut it = args.into_iter();
                                let start = it.next().unwrap();
                                let end = it.next().unwrap();
                                expr = e::Expr::Range {
                                    start: Some(Box::new(start)),
                                    end: Some(Box::new(end)),
                                    inclusive: false,
                                };
                                continue;
                            }
                            _ => {} // fall through to normal Call
                        }
                    }
                }
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
            } else if self.check_kw(Keyword::As) {
                self.advance()?; // consume 'as'
                let target_type = self.parse_type()?;
                expr = e::Expr::Cast {
                    expr: Box::new(expr),
                    target_type,
                };
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
                // Wrap in str() constructor for Smart String (Arc<str>)
                Ok(e::Expr::Call {
                    callee: Box::new(e::Expr::Path(vec!["str".into()])),
                    args: vec![e::Expr::String(s)],
                })
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
                if self.check(&TokenKind::RBracket) {
                    self.advance()?;
                    return Ok(e::Expr::Array(vec![]));
                }
                let first = self.parse_expr()?;
                // List comprehension: [expr for var in iter] or [expr for var in iter if cond]
                if self.check(&TokenKind::Keyword(Keyword::For)) {
                    self.advance()?; // consume 'for'
                    let var = self.expect_ident()?;
                    self.expect(&TokenKind::Keyword(Keyword::In))?;
                    let iter_expr = self.parse_expr()?;
                    // Optional filter: if cond
                    let filter = if self.eat(&TokenKind::Keyword(Keyword::If))? {
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(&TokenKind::RBracket)?;
                    return Ok(Self::build_list_comprehension(
                        var, iter_expr, first, filter,
                    ));
                }
                // Regular array literal
                let mut elems = vec![first];
                if self.eat(&TokenKind::Comma)? {
                    while !self.check(&TokenKind::RBracket) {
                        elems.push(self.parse_expr()?);
                        if !self.eat(&TokenKind::Comma)? {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(e::Expr::Array(elems))
            }
            TokenKind::FString { content, .. } => {
                self.advance()?;
                // f-string → emit as format!() with ALL expressions as positional args.
                // Rust's format!() only accepts identifiers inside {}, not expressions,
                // so we always extract to positional args:
                //   f"{name} is {age * 2}" → format!("{} is {}", name, age * 2)
                //   f"literal {{braces}}"  → format!("literal {{braces}}")
                let mut format_str = String::new();
                let mut args: Vec<e::Expr> = Vec::new();
                let mut chars = content.chars().peekable();
                while let Some(c) = chars.next() {
                    if c == '{' {
                        // Check for escaped brace: {{ → literal {
                        if chars.peek() == Some(&'{') {
                            chars.next();
                            format_str.push_str("{{");
                            continue;
                        }
                        // Collect the expression inside { ... }
                        let mut expr_str = String::new();
                        let mut brace_depth = 1;
                        while let Some(&nc) = chars.peek() {
                            if nc == '{' {
                                brace_depth += 1;
                                expr_str.push(nc);
                                chars.next();
                            } else if nc == '}' {
                                brace_depth -= 1;
                                if brace_depth == 0 {
                                    chars.next();
                                    break;
                                }
                                expr_str.push(nc);
                                chars.next();
                            } else {
                                expr_str.push(nc);
                                chars.next();
                            }
                        }
                        // ALL expressions become positional args (safe for Rust format!)
                        format_str.push_str("{}");
                        let mut sub = Parser::new(&expr_str)?;
                        let parsed_expr = sub.parse_expr()?;
                        args.push(parsed_expr);
                    } else if c == '}' {
                        // Check for escaped close brace: }} → literal }
                        if chars.peek() == Some(&'}') {
                            chars.next();
                            format_str.push_str("}}");
                        } else {
                            format_str.push(c);
                        }
                    } else {
                        format_str.push(c);
                    }
                }
                let mut macro_args = vec![e::Expr::String(format_str)];
                macro_args.extend(args);
                let macro_call = e::Expr::MacroCall {
                    path: vec!["format".into()],
                    args: macro_args,
                };
                // Wrap in str() to produce Str instead of String
                Ok(e::Expr::Call {
                    callee: Box::new(e::Expr::Path(vec!["str".into()])),
                    args: vec![macro_call],
                })
            }
            TokenKind::LBrace => {
                self.advance()?;
                self.parse_dict_literal()
            }
            TokenKind::Pipe => {
                self.advance()?; // consume opening |
                // Parse params: |x, y: i32, z| or || (empty)
                let mut params = Vec::new();
                while !self.check(&TokenKind::Pipe) {
                    let name = self.expect_ident()?;
                    let ty = if self.eat(&TokenKind::Colon)? {
                        self.parse_type()?
                    } else {
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
                self.expect(&TokenKind::Pipe)?; // consume closing |
                let body_expr = self.parse_expr()?;
                Ok(e::Expr::Closure {
                    params,
                    return_type: None,
                    body: e::Block {
                        statements: vec![e::Stmt::TailExpr(body_expr)],
                    },
                })
            }
            // Python-style lambda: `lambda x, y: expr`
            TokenKind::Keyword(Keyword::Lambda) => {
                self.advance()?; // consume 'lambda'
                let mut params = Vec::new();
                // Parse params until we hit ':'
                while !self.check(&TokenKind::Colon) {
                    let name = self.expect_ident()?;
                    let ty = e::Type {
                        path: vec!["_".into()],
                        args: vec![],
                        trait_bounds: vec![],
                    };
                    params.push(e::Param { name, ty });
                    if !self.eat(&TokenKind::Comma)? {
                        break;
                    }
                }
                self.expect(&TokenKind::Colon)?; // consume ':'
                let body_expr = self.parse_expr()?;
                Ok(e::Expr::Closure {
                    params,
                    return_type: None,
                    body: e::Block {
                        statements: vec![e::Stmt::TailExpr(body_expr)],
                    },
                })
            }
            _ => Err(self.error(format!("expected expression, got {}", self.kind()))),
        }
    }

    /// Parse a dict literal: `{key: val, ...}` or `{**spread, key: val}`
    /// Called after `{` has been consumed.
    fn parse_dict_literal(&mut self) -> Result<e::Expr, ParseError> {
        // Empty dict: {}
        if self.check(&TokenKind::RBrace) {
            self.advance()?;
            return Ok(e::Expr::Call {
                callee: Box::new(e::Expr::Path(vec!["HashMap".into(), "new".into()])),
                args: vec![],
            });
        }

        // Collect entries: either (key, val) pairs or **spread
        enum DictEntry {
            Pair(e::Expr, e::Expr),
            Spread(e::Expr),
        }
        let mut entries = Vec::new();

        loop {
            if self.check(&TokenKind::RBrace) {
                break;
            }

            // Check for **spread
            if self.check(&TokenKind::DoubleStar) {
                self.advance()?;
                let spread_expr = self.parse_expr()?;
                entries.push(DictEntry::Spread(spread_expr));
            } else {
                // key: value
                let key = self.parse_expr()?;
                self.expect(&TokenKind::Colon)?;
                let value = self.parse_expr()?;

                // Dict comprehension: {key: val for var in iter}
                if self.check(&TokenKind::Keyword(Keyword::For)) {
                    self.advance()?; // consume 'for'
                    let var = self.expect_ident()?;
                    self.expect(&TokenKind::Keyword(Keyword::In))?;
                    let iter_expr = self.parse_expr()?;
                    // Optional filter: if cond
                    let filter = if self.eat(&TokenKind::Keyword(Keyword::If))? {
                        Some(self.parse_expr()?)
                    } else {
                        None
                    };
                    self.expect(&TokenKind::RBrace)?;
                    return Ok(Self::build_dict_comprehension(
                        var, iter_expr, key, value, filter,
                    ));
                }

                entries.push(DictEntry::Pair(key, value));
            }

            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        self.expect(&TokenKind::RBrace)?;

        // Check if there are any spreads
        let has_spread = entries.iter().any(|e| matches!(e, DictEntry::Spread(_)));

        if !has_spread {
            // Simple dict: HashMap::from([(k1, v1), (k2, v2)])
            let pairs: Vec<e::Expr> = entries
                .into_iter()
                .map(|e| match e {
                    DictEntry::Pair(k, v) => e::Expr::Tuple(vec![k, v]),
                    _ => unreachable!(),
                })
                .collect();
            Ok(e::Expr::Call {
                callee: Box::new(e::Expr::Path(vec!["HashMap".into(), "from".into()])),
                args: vec![e::Expr::Array(pairs)],
            })
        } else {
            // Dict with spread: emit as RustBlock
            // { let mut __m = HashMap::new(); __m.extend(spread); __m.insert(k, v); __m }
            let mut code = String::from("{ let mut __m = std::collections::HashMap::new(); ");
            for entry in entries {
                match entry {
                    DictEntry::Spread(expr) => {
                        // We need to emit the spread expr as Rust — for simple paths
                        if let e::Expr::Path(ref p) = expr {
                            code.push_str(&format!(
                                "__m.extend({}.iter().map(|(k, v)| (k.clone(), v.clone()))); ",
                                p.join("::")
                            ));
                        }
                    }
                    DictEntry::Pair(key, value) => {
                        // For string keys and simple values
                        let key_str = Self::expr_to_rust_string(&key);
                        let val_str = Self::expr_to_rust_string(&value);
                        code.push_str(&format!("__m.insert({}, {}); ", key_str, val_str));
                    }
                }
            }
            code.push_str("__m }");
            Ok(e::Expr::MacroCall {
                path: vec!["__rust_expr__".into()],
                args: vec![e::Expr::String(code)],
            })
        }
    }

    /// Best-effort conversion of an Expr to a Rust string for RustBlock emission.
    fn expr_to_rust_string(expr: &e::Expr) -> String {
        match expr {
            e::Expr::String(s) => format!("String::from(\"{}\")", s),
            e::Expr::Int(n) => n.to_string(),
            e::Expr::Bool(b) => b.to_string(),
            e::Expr::Path(p) => p.join("::"),
            e::Expr::Binary { op, left, right } => {
                let op_str = match op {
                    e::BinaryOp::Add => "+",
                    e::BinaryOp::Sub => "-",
                    e::BinaryOp::Mul => "*",
                    e::BinaryOp::Div => "/",
                    e::BinaryOp::Rem => "%",
                    e::BinaryOp::And => "&&",
                    e::BinaryOp::Or => "||",
                    e::BinaryOp::Eq => "==",
                    e::BinaryOp::Ne => "!=",
                    e::BinaryOp::Lt => "<",
                    e::BinaryOp::Le => "<=",
                    e::BinaryOp::Gt => ">",
                    e::BinaryOp::Ge => ">=",
                };
                format!(
                    "({} {} {})",
                    Self::expr_to_rust_string(left),
                    op_str,
                    Self::expr_to_rust_string(right)
                )
            }
            e::Expr::Unary { op, expr: inner } => {
                let op_str = match op {
                    e::UnaryOp::Not => "!",
                    e::UnaryOp::Neg => "-",
                };
                format!("({}{})", op_str, Self::expr_to_rust_string(inner))
            }
            e::Expr::Field { base, field } => {
                format!("{}.{}", Self::expr_to_rust_string(base), field)
            }
            e::Expr::Tuple(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::expr_to_rust_string).collect();
                format!("({})", parts.join(", "))
            }
            e::Expr::Call { callee, args } => {
                let args_str: Vec<String> = args.iter().map(Self::expr_to_rust_string).collect();
                format!(
                    "{}({})",
                    Self::expr_to_rust_string(callee),
                    args_str.join(", ")
                )
            }
            e::Expr::Index { base, index } => {
                format!(
                    "{}[{}]",
                    Self::expr_to_rust_string(base),
                    Self::expr_to_rust_string(index)
                )
            }
            _ => "todo!()".into(),
        }
    }

    /// Build a list comprehension: [expr for var in iter if cond]
    /// Desugars to IIFE: (|| { let __v = Vec::new(); for var in iter { [if cond {] __v.push(expr); [}] } __v })()
    fn build_list_comprehension(
        var: String,
        iter_expr: e::Expr,
        map_expr: e::Expr,
        filter: Option<e::Expr>,
    ) -> e::Expr {
        // Build the push statement: __v.push(map_expr)
        let push_stmt = e::Stmt::Expr(e::Expr::Call {
            callee: Box::new(e::Expr::Field {
                base: Box::new(e::Expr::Path(vec!["__v".into()])),
                field: "push".into(),
            }),
            args: vec![map_expr],
        });

        // For body: either just push, or if cond { push }
        let for_body = if let Some(cond) = filter {
            e::Block {
                statements: vec![e::Stmt::If {
                    condition: cond,
                    then_block: e::Block {
                        statements: vec![push_stmt],
                    },
                    else_block: None,
                }],
            }
        } else {
            e::Block {
                statements: vec![push_stmt],
            }
        };

        // Build IIFE body: let __v = Vec::new(); for var in iter { ... }; __v
        let iife_body = e::Block {
            statements: vec![
                // let __v: Vec<_> = Vec::new()
                e::Stmt::Const(e::ConstDef {
                    visibility: e::Visibility::Private,
                    name: "__v".into(),
                    ty: Some(e::Type {
                        path: vec!["Vec".into()],
                        args: vec![e::Type {
                            path: vec!["_".into()],
                            args: vec![],
                            trait_bounds: vec![],
                        }],
                        trait_bounds: vec![],
                    }),
                    value: e::Expr::Call {
                        callee: Box::new(e::Expr::Path(vec!["Vec".into(), "new".into()])),
                        args: vec![],
                    },
                    is_const: false,
                    span: None,
                }),
                // for var in iter { ... }
                e::Stmt::For {
                    binding: e::DestructurePattern::Name(var),
                    iter: iter_expr,
                    body: for_body,
                },
                // __v (tail expression — return the vec)
                e::Stmt::TailExpr(e::Expr::Path(vec!["__v".into()])),
            ],
        };

        // Wrap in IIFE: (|| { ... })()
        e::Expr::Call {
            callee: Box::new(e::Expr::Closure {
                params: vec![],
                return_type: None,
                body: iife_body,
            }),
            args: vec![],
        }
    }

    /// Build a dict comprehension: {key: val for var in iter if cond}
    /// Desugars to IIFE: build Vec<(K,V)> with push, then .into_iter().collect()
    fn build_dict_comprehension(
        var: String,
        iter_expr: e::Expr,
        key_expr: e::Expr,
        val_expr: e::Expr,
        filter: Option<e::Expr>,
    ) -> e::Expr {
        // Build: __v.push((key, val))
        let push_stmt = e::Stmt::Expr(e::Expr::Call {
            callee: Box::new(e::Expr::Field {
                base: Box::new(e::Expr::Path(vec!["__v".into()])),
                field: "push".into(),
            }),
            args: vec![e::Expr::Tuple(vec![key_expr, val_expr])],
        });

        let for_body = if let Some(cond) = filter {
            e::Block {
                statements: vec![e::Stmt::If {
                    condition: cond,
                    then_block: e::Block {
                        statements: vec![push_stmt],
                    },
                    else_block: None,
                }],
            }
        } else {
            e::Block {
                statements: vec![push_stmt],
            }
        };

        // __v.into_iter().collect()
        let collect_expr = e::Expr::Call {
            callee: Box::new(e::Expr::Field {
                base: Box::new(e::Expr::Call {
                    callee: Box::new(e::Expr::Field {
                        base: Box::new(e::Expr::Path(vec!["__v".into()])),
                        field: "into_iter".into(),
                    }),
                    args: vec![],
                }),
                field: "collect".into(),
            }),
            args: vec![],
        };

        let iife_body = e::Block {
            statements: vec![
                // let __v: Vec<_> = Vec::new()
                e::Stmt::Const(e::ConstDef {
                    visibility: e::Visibility::Private,
                    name: "__v".into(),
                    ty: Some(e::Type {
                        path: vec!["Vec".into()],
                        args: vec![e::Type {
                            path: vec!["_".into()],
                            args: vec![],
                            trait_bounds: vec![],
                        }],
                        trait_bounds: vec![],
                    }),
                    value: e::Expr::Call {
                        callee: Box::new(e::Expr::Path(vec!["Vec".into(), "new".into()])),
                        args: vec![],
                    },
                    is_const: false,
                    span: None,
                }),
                // for var in iter { __v.push((key, val)); }
                e::Stmt::For {
                    binding: e::DestructurePattern::Name(var),
                    iter: iter_expr,
                    body: for_body,
                },
                // __v.into_iter().collect()
                e::Stmt::TailExpr(collect_expr),
            ],
        };

        e::Expr::Call {
            callee: Box::new(e::Expr::Closure {
                params: vec![],
                return_type: None,
                body: iife_body,
            }),
            args: vec![],
        }
    }

    /// Parse call arguments, detecting keyword args (name=expr).
    fn parse_call_args_with_kwargs(&mut self) -> Result<Vec<CallArg>, ParseError> {
        let mut args = Vec::new();
        while !self.check(&TokenKind::RParen) {
            // Check for keyword arg: Ident followed by '='
            let is_kwarg = if let TokenKind::Ident(_) = self.kind() {
                matches!(self.peek()?.kind, TokenKind::Eq)
            } else {
                false
            };
            if is_kwarg {
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Eq)?;
                let value = self.parse_expr()?;
                args.push(CallArg::Keyword(name, value));
            } else {
                let expr = self.parse_expr()?;
                args.push(CallArg::Positional(expr));
            }
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

/// Quiche primitive type prelude — imports types from `quiche-lib` crate.
///
/// Two RustBlocks:
/// 1. `use quiche_lib::*;` — actual import
/// 2. Stub fn `str()` — so Elevate's `extract_rust_block_function_names` resolves it
fn quiche_prelude() -> Vec<e::Item> {
    vec![
        e::Item::RustBlock("use quiche_lib::*;".into()),
        e::Item::RustBlock(
            "pub fn str<T: std::fmt::Display>(x: T) -> Str { quiche_lib::str(x) }".into(),
        ),
    ]
}

pub fn parse(source: &str) -> Result<e::Module, ParseError> {
    let mut parser = Parser::new(source)?;
    let mut module = parser.parse_module()?;

    // Inject Quiche primitive type prelude at the top
    let mut new_items = quiche_prelude();
    new_items.extend(module.items);
    module.items = new_items;

    Ok(module)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::parse;
    use elevate::ast::*;

    fn parse_body(source: &str) -> Vec<Stmt> {
        let module = parse(source).unwrap();
        // Skip Quiche prelude items (2 RustBlocks: use + str stub)
        let user_items = &module.items[2..];
        match &user_items[0] {
            Item::Function(f) => f.body.statements.clone(),
            other => panic!("Expected Function, got {:?}", other),
        }
    }

    // ─── Star Destructuring ─────────────────────────────────────────────────

    #[test]
    fn test_star_prefix_rest() {
        let stmts = parse_body("def test():\n    first, *rest = items\n");
        match &stmts[0] {
            Stmt::DestructureConst { pattern, .. } => match pattern {
                DestructurePattern::Slice {
                    prefix,
                    rest,
                    suffix,
                } => {
                    assert_eq!(prefix.len(), 1);
                    assert!(matches!(&prefix[0], DestructurePattern::Name(n) if n == "first"));
                    assert_eq!(rest.as_deref(), Some("rest"));
                    assert!(suffix.is_empty());
                }
                other => panic!("Expected Slice pattern, got {:?}", other),
            },
            other => panic!("Expected DestructureConst, got {:?}", other),
        }
    }

    #[test]
    fn test_star_prefix_rest_suffix() {
        let stmts = parse_body("def test():\n    a, *middle, last = items\n");
        match &stmts[0] {
            Stmt::DestructureConst { pattern, .. } => match pattern {
                DestructurePattern::Slice {
                    prefix,
                    rest,
                    suffix,
                } => {
                    assert_eq!(prefix.len(), 1);
                    assert!(matches!(&prefix[0], DestructurePattern::Name(n) if n == "a"));
                    assert_eq!(rest.as_deref(), Some("middle"));
                    assert_eq!(suffix.len(), 1);
                    assert!(matches!(&suffix[0], DestructurePattern::Name(n) if n == "last"));
                }
                other => panic!("Expected Slice pattern, got {:?}", other),
            },
            other => panic!("Expected DestructureConst, got {:?}", other),
        }
    }

    #[test]
    fn test_star_only() {
        let stmts = parse_body("def test():\n    *everything = items\n");
        match &stmts[0] {
            Stmt::DestructureConst { pattern, .. } => match pattern {
                DestructurePattern::Slice {
                    prefix,
                    rest,
                    suffix,
                } => {
                    assert!(prefix.is_empty());
                    assert_eq!(rest.as_deref(), Some("everything"));
                    assert!(suffix.is_empty());
                }
                other => panic!("Expected Slice pattern, got {:?}", other),
            },
            other => panic!("Expected DestructureConst, got {:?}", other),
        }
    }

    #[test]
    fn test_star_with_ignore() {
        let stmts = parse_body("def test():\n    _, *rest, _ = items\n");
        match &stmts[0] {
            Stmt::DestructureConst { pattern, .. } => match pattern {
                DestructurePattern::Slice {
                    prefix,
                    rest,
                    suffix,
                } => {
                    assert_eq!(prefix.len(), 1);
                    assert!(matches!(&prefix[0], DestructurePattern::Ignore));
                    assert_eq!(rest.as_deref(), Some("rest"));
                    assert_eq!(suffix.len(), 1);
                    assert!(matches!(&suffix[0], DestructurePattern::Ignore));
                }
                other => panic!("Expected Slice pattern, got {:?}", other),
            },
            other => panic!("Expected DestructureConst, got {:?}", other),
        }
    }

    // ─── Print Built-in ─────────────────────────────────────────────────────

    #[test]
    fn test_print_no_args() {
        let stmts = parse_body("def test():\n    print()\n");
        match &stmts[0] {
            Stmt::Expr(Expr::MacroCall { path, args }) => {
                assert_eq!(path, &["println"]);
                assert_eq!(args.len(), 1); // just the format string
                assert!(matches!(&args[0], Expr::String(s) if s.is_empty()));
            }
            other => panic!("Expected MacroCall, got {:?}", other),
        }
    }

    #[test]
    fn test_print_multiple_args() {
        let stmts = parse_body("def test():\n    print(a, b, c)\n");
        match &stmts[0] {
            Stmt::Expr(Expr::MacroCall { path, args }) => {
                assert_eq!(path, &["println"]);
                assert_eq!(args.len(), 4); // format string + 3 args
                assert!(matches!(&args[0], Expr::String(s) if s == "{} {} {}"));
            }
            other => panic!("Expected MacroCall, got {:?}", other),
        }
    }

    // ─── Rust Escape Hatch ──────────────────────────────────────────────────

    #[test]
    fn test_rust_escape_hatch() {
        let stmts = parse_body("def test():\n    rust(\"println!(42);\")\n");
        match &stmts[0] {
            Stmt::RustBlock(code) => {
                assert_eq!(code, "println!(42);");
            }
            other => panic!("Expected RustBlock, got {:?}", other),
        }
    }

    // ─── Static Method Calls ─────────────────────────────────────────────────

    #[test]
    fn test_static_method_call_path() {
        // Student.new("Alice", 16) should generate Path(["Student", "new"]) as callee
        let stmts = parse_body("def test():\n    x = Student.new()\n");
        match &stmts[0] {
            Stmt::Assign { value, .. } => match value {
                Expr::Call { callee, .. } => match callee.as_ref() {
                    Expr::Path(segments) => {
                        assert_eq!(segments, &["Student", "new"]);
                    }
                    other => panic!("Expected Path callee, got {:?}", other),
                },
                other => panic!("Expected Call, got {:?}", other),
            },
            other => panic!("Expected Assign, got {:?}", other),
        }
    }

    #[test]
    fn test_instance_method_remains_field() {
        // obj.method() where obj is lowercase should remain as Field access
        let stmts = parse_body("def test():\n    x = obj.method()\n");
        match &stmts[0] {
            Stmt::Assign { value, .. } => match value {
                Expr::Call { callee, .. } => match callee.as_ref() {
                    Expr::Field { base, field } => {
                        assert!(matches!(base.as_ref(), Expr::Path(p) if p == &["obj"]));
                        assert_eq!(field, "method");
                    }
                    other => panic!("Expected Field callee, got {:?}", other),
                },
                other => panic!("Expected Call, got {:?}", other),
            },
            other => panic!("Expected Assign, got {:?}", other),
        }
    }

    #[test]
    fn test_static_field_access_no_call() {
        // Type.field without () should remain as Field (not merged into path)
        let stmts = parse_body("def test():\n    x = Type.CONST\n");
        match &stmts[0] {
            Stmt::Assign { value, .. } => match value {
                Expr::Field { base, field } => {
                    assert!(matches!(base.as_ref(), Expr::Path(p) if p == &["Type"]));
                    assert_eq!(field, "CONST");
                }
                other => panic!("Expected Field, got {:?}", other),
            },
            other => panic!("Expected Assign, got {:?}", other),
        }
    }

    // ─── Enum Definitions ────────────────────────────────────────────────────
}
