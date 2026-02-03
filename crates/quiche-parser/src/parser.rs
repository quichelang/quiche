//! Quiche Parser - Hand-written recursive descent parser
//!
//! Parses token streams from the lexer into the Quiche AST.
//! Replaces the Ruff-based parser for full control and fewer dependencies.
#![allow(clippy::unwrap_used)]
use crate::ast::*;
use crate::lexer::{Keyword, LexError, Lexer, Token, TokenKind};

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
    /// Create a new parser for the given source code
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

    /// Get current token kind
    fn current_kind(&self) -> &TokenKind {
        &self.current.kind
    }

    /// Check if current token matches a kind
    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(self.current_kind()) == std::mem::discriminant(kind)
    }

    /// Check if current token is a specific keyword
    fn check_keyword(&self, kw: Keyword) -> bool {
        matches!(self.current_kind(), TokenKind::Keyword(k) if *k == kw)
    }

    /// Advance to next token, returning the current one
    fn advance(&mut self) -> Result<Token, ParseError> {
        let current = std::mem::replace(
            &mut self.current,
            if let Some(peeked) = self.peeked.take() {
                peeked
            } else {
                self.lexer.next_token()?
            },
        );
        Ok(current)
    }

    /// Peek at the next token without consuming
    fn peek(&mut self) -> Result<&Token, ParseError> {
        if self.peeked.is_none() {
            self.peeked = Some(self.lexer.next_token()?);
        }
        Ok(self.peeked.as_ref().unwrap())
    }

    /// Consume token if it matches, return error otherwise
    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        if self.check(kind) {
            self.advance()
        } else {
            Err(self.error(format!(
                "Expected {:?}, got {:?}",
                kind,
                self.current_kind()
            )))
        }
    }

    /// Consume keyword if matches
    fn expect_keyword(&mut self, kw: Keyword) -> Result<Token, ParseError> {
        if self.check_keyword(kw) {
            self.advance()
        } else {
            Err(self.error(format!(
                "Expected keyword {:?}, got {:?}",
                kw,
                self.current_kind()
            )))
        }
    }

    /// Consume token if it matches, return true if consumed
    fn eat(&mut self, kind: &TokenKind) -> Result<bool, ParseError> {
        if self.check(kind) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create error at current position
    fn error(&self, message: String) -> ParseError {
        ParseError {
            message,
            line: self.current.line,
            column: self.current.column,
        }
    }

    /// Skip newlines and comments
    fn skip_newlines(&mut self) -> Result<(), ParseError> {
        while matches!(
            self.current_kind(),
            TokenKind::Newline | TokenKind::Comment(_)
        ) {
            self.advance()?;
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Top-level parsing
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse entire module
    pub fn parse_module(&mut self) -> Result<QuicheModule, ParseError> {
        let mut body = Vec::new();
        self.skip_newlines()?;

        while !matches!(self.current_kind(), TokenKind::Eof) {
            let stmt = self.parse_statement()?;
            body.push(stmt);
            self.skip_newlines()?;
        }

        Ok(QuicheModule { body })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Statement parsing
    // ─────────────────────────────────────────────────────────────────────────

    fn parse_statement(&mut self) -> Result<QuicheStmt, ParseError> {
        // Handle decorators
        let decorators = self.parse_decorators()?;

        match self.current_kind() {
            TokenKind::Keyword(Keyword::Def) => self.parse_function_def(decorators),
            TokenKind::Keyword(Keyword::Class) => self.parse_class_def(decorators),
            TokenKind::Keyword(Keyword::If) => self.parse_if_stmt(),
            TokenKind::Keyword(Keyword::While) => self.parse_while_stmt(),
            TokenKind::Keyword(Keyword::For) => self.parse_for_stmt(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_stmt(),
            TokenKind::Keyword(Keyword::Pass) => {
                self.advance()?;
                Ok(QuicheStmt::Pass)
            }
            TokenKind::Keyword(Keyword::Break) => {
                self.advance()?;
                Ok(QuicheStmt::Break)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                self.advance()?;
                Ok(QuicheStmt::Continue)
            }
            TokenKind::Keyword(Keyword::Import) => self.parse_import_stmt(),
            TokenKind::Keyword(Keyword::From) => self.parse_from_import_stmt(),
            TokenKind::Keyword(Keyword::Match) => self.parse_match_stmt(),
            TokenKind::Keyword(Keyword::Assert) => self.parse_assert_stmt(),
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    /// Parse decorator list (@ expressions before def/class)
    fn parse_decorators(&mut self) -> Result<Vec<QuicheExpr>, ParseError> {
        let mut decorators = Vec::new();
        while self.check(&TokenKind::At) {
            self.advance()?; // consume @
            let expr = self.parse_expression()?;
            decorators.push(expr);
            self.skip_newlines()?;
        }
        Ok(decorators)
    }

    /// Parse function definition
    fn parse_function_def(
        &mut self,
        decorator_list: Vec<QuicheExpr>,
    ) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::Def)?;

        // Function name
        let name = self.expect_ident()?;

        // Type parameters [T, U, ...]
        let type_params = self.parse_type_params()?;

        // Arguments
        self.expect(&TokenKind::LParen)?;
        let args = self.parse_args()?;
        self.expect(&TokenKind::RParen)?;

        // Return type annotation
        let returns = if self.eat(&TokenKind::Arrow)? {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        // Body
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        // Determine self_kind from first argument annotation, or scan body if no annotation
        let self_kind = if let Some(first_arg) = args.first() {
            if first_arg.arg == "self" {
                Self::determine_self_kind(first_arg.annotation.as_deref(), &body)
            } else {
                SelfKind::NoSelf
            }
        } else {
            SelfKind::NoSelf
        };

        Ok(QuicheStmt::FunctionDef(FunctionDef {
            name,
            args,
            self_kind,
            body,
            decorator_list,
            returns,
            type_params,
        }))
    }

    /// Determine SelfKind from a self parameter's type annotation.
    /// If no explicit mutability annotation, scan the body to detect if self is mutated.
    fn determine_self_kind(annotation: Option<&QuicheExpr>, body: &[QuicheStmt]) -> SelfKind {
        match annotation {
            // self: MutRef[Self] or self: Mut[Self] -> &mut self (explicit mutable)
            Some(QuicheExpr::Subscript { value, .. }) => {
                if let QuicheExpr::Name(name) = value.as_ref() {
                    match name.as_str() {
                        "MutRef" | "Mut" => SelfKind::Ref(Mutability::Mut),
                        // Ref[Self] means explicit immutable
                        "Ref" => SelfKind::Ref(Mutability::Not),
                        // Other generic types like Box[Self] - scan body
                        _ => {
                            if Self::is_self_mutated_in_stmts(body) {
                                SelfKind::Ref(Mutability::Mut)
                            } else {
                                SelfKind::Ref(Mutability::Not)
                            }
                        }
                    }
                } else {
                    // Complex subscript - scan body
                    if Self::is_self_mutated_in_stmts(body) {
                        SelfKind::Ref(Mutability::Mut)
                    } else {
                        SelfKind::Ref(Mutability::Not)
                    }
                }
            }
            // self: Self -> self (by value, immutable)
            Some(QuicheExpr::Name(name)) if name == "Self" => SelfKind::Value(Mutability::Not),
            // self: SomeConcreteType (like `self: Parsley`) -> scan body to determine mutability
            Some(QuicheExpr::Name(_)) => {
                if Self::is_self_mutated_in_stmts(body) {
                    SelfKind::Ref(Mutability::Mut)
                } else {
                    SelfKind::Ref(Mutability::Not)
                }
            }
            // self (no annotation) -> auto-detect by scanning body
            None => {
                if Self::is_self_mutated_in_stmts(body) {
                    SelfKind::Ref(Mutability::Mut)
                } else {
                    SelfKind::Ref(Mutability::Not)
                }
            }
            // Other annotations -> scan body
            _ => {
                if Self::is_self_mutated_in_stmts(body) {
                    SelfKind::Ref(Mutability::Mut)
                } else {
                    SelfKind::Ref(Mutability::Not)
                }
            }
        }
    }

    /// Check if self is mutated in a list of statements (body scanning)
    fn is_self_mutated_in_stmts(stmts: &[QuicheStmt]) -> bool {
        for stmt in stmts {
            match stmt {
                QuicheStmt::Assign(a) => {
                    for target in &a.targets {
                        match target {
                            QuicheExpr::Attribute { value, .. } => {
                                if Self::is_self_expr(value) {
                                    return true;
                                }
                            }
                            QuicheExpr::Subscript { value, .. } => {
                                if Self::is_nested_self(value) {
                                    return true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                QuicheStmt::Expr(e) => {
                    if Self::is_mutating_call(e) {
                        return true;
                    }
                }
                QuicheStmt::If(i) => {
                    if Self::is_self_mutated_in_stmts(&i.body)
                        || Self::is_self_mutated_in_stmts(&i.orelse)
                    {
                        return true;
                    }
                }
                QuicheStmt::While(w) => {
                    if Self::is_self_mutated_in_stmts(&w.body)
                        || Self::is_self_mutated_in_stmts(&w.orelse)
                    {
                        return true;
                    }
                }
                QuicheStmt::For(f) => {
                    if Self::is_self_mutated_in_stmts(&f.body)
                        || Self::is_self_mutated_in_stmts(&f.orelse)
                    {
                        return true;
                    }
                }
                QuicheStmt::Match(m) => {
                    for case in &m.cases {
                        if Self::is_self_mutated_in_stmts(&case.body) {
                            return true;
                        }
                    }
                }
                QuicheStmt::Return(r) => {
                    if let Some(v) = r {
                        if Self::is_mutating_call(v) {
                            return true;
                        }
                    }
                }
                QuicheStmt::AnnAssign(a) => {
                    if let QuicheExpr::Attribute { value, .. } = &*a.target {
                        if Self::is_self_expr(value) {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Check if an expression is just `self`
    fn is_self_expr(expr: &QuicheExpr) -> bool {
        matches!(expr, QuicheExpr::Name(n) if n == "self")
    }

    /// Check if an expression is `self` or `self.field` or `self.field[i]` etc.
    fn is_nested_self(expr: &QuicheExpr) -> bool {
        match expr {
            QuicheExpr::Name(n) => n == "self",
            QuicheExpr::Attribute { value, .. } => Self::is_nested_self(value),
            QuicheExpr::Subscript { value, .. } => Self::is_nested_self(value),
            _ => false,
        }
    }

    /// Check if an expression is a mutating method call on self or contains one
    fn is_mutating_call(expr: &QuicheExpr) -> bool {
        match expr {
            QuicheExpr::Call { func, args, .. } => {
                // Check if it's a mutating method call on self
                if let QuicheExpr::Attribute { value, attr } = &**func {
                    if Self::is_nested_self(value) {
                        // Check for known mutating methods
                        if attr == "push"
                            || attr == "pop"
                            || attr == "insert"
                            || attr == "remove"
                            || attr == "clear"
                            || attr == "update"
                            || attr == "append"
                            || attr == "extend"
                            || attr.starts_with("transform_")
                            || attr == "visit_def"
                            || attr == "collect_extern"
                            || attr == "emit"
                            || attr == "T"
                            || attr.starts_with("generate_")
                            || attr == "enter_var_scope"
                            || attr == "exit_var_scope"
                            || attr.starts_with("define_")
                            || attr.starts_with("mark_")
                            || attr == "push_str"
                            || attr == "add_flag"
                            || attr == "add_option"
                            || attr == "add_command"
                            || attr == "register_aliases"
                            || attr.starts_with("set_")
                            || attr.starts_with("add_")
                        {
                            return true;
                        }
                    }
                }
                // Recursively check call arguments
                for arg in args {
                    if Self::is_mutating_call(arg) {
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    /// Parse class definition (also handles Struct, Enum, Trait, Impl)
    fn parse_class_def(
        &mut self,
        decorator_list: Vec<QuicheExpr>,
    ) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::Class)?;

        // Class name
        let name = self.expect_ident()?;

        // Type parameters [T, U, ...]
        let type_params = self.parse_type_params()?;

        // Base classes
        let bases = if self.eat(&TokenKind::LParen)? {
            let bases = self.parse_expr_list()?;
            self.expect(&TokenKind::RParen)?;
            bases
        } else {
            Vec::new()
        };

        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        // Check for special base classes to determine type
        let base_names: Vec<&str> = bases
            .iter()
            .filter_map(|b| {
                if let QuicheExpr::Name(n) = b {
                    Some(n.as_str())
                } else {
                    None
                }
            })
            .collect();

        // Note: class Foo(Struct): is kept as ClassDef so methods are preserved.
        // The codegen handles the Struct base by emitting a struct + impl block.

        // Check if this is an Enum definition
        if base_names.contains(&"Enum") {
            return Ok(QuicheStmt::EnumDef(self.lower_to_enum(
                name,
                type_params,
                body,
            )?));
        }

        // Check if this is a Trait definition
        if base_names.contains(&"Trait") {
            return Ok(QuicheStmt::TraitDef(TraitDef { name, body }));
        }

        // Check for @impl decorator
        for dec in &decorator_list {
            if let QuicheExpr::Call { func, args, .. } = dec {
                if let QuicheExpr::Name(n) = func.as_ref() {
                    if n == "impl" {
                        let trait_name = args.first().and_then(|a| {
                            if let QuicheExpr::Name(tn) = a {
                                Some(tn.clone())
                            } else {
                                None
                            }
                        });
                        return Ok(QuicheStmt::ImplDef(ImplDef {
                            trait_name,
                            target_type: name,
                            body,
                        }));
                    }
                }
            }
        }

        Ok(QuicheStmt::ClassDef(ClassDef {
            name,
            bases,
            body,
            decorator_list,
            type_params,
        }))
    }

    /// Lower class body to struct fields
    fn lower_to_struct(
        &self,
        name: String,
        type_params: Vec<String>,
        body: Vec<QuicheStmt>,
    ) -> Result<StructDef, ParseError> {
        let mut fields = Vec::new();
        for stmt in body {
            if let QuicheStmt::AnnAssign(ann) = stmt {
                if let QuicheExpr::Name(field_name) = *ann.target {
                    let ty = self.expr_to_type_string(&ann.annotation);
                    fields.push(FieldDef {
                        name: field_name,
                        ty,
                    });
                }
            }
        }
        Ok(StructDef {
            name,
            type_params,
            fields,
        })
    }

    /// Lower class body to enum variants
    fn lower_to_enum(
        &self,
        name: String,
        type_params: Vec<String>,
        body: Vec<QuicheStmt>,
    ) -> Result<EnumDef, ParseError> {
        let mut variants = Vec::new();
        for stmt in body {
            if let QuicheStmt::Assign(assign) = stmt {
                if let Some(QuicheExpr::Name(variant_name)) = assign.targets.first().cloned() {
                    let fields = if let QuicheExpr::Tuple(elems) = *assign.value {
                        elems.iter().map(|e| self.expr_to_type_string(e)).collect()
                    } else {
                        Vec::new()
                    };
                    variants.push(VariantDef {
                        name: variant_name,
                        fields,
                    });
                }
            }
        }
        Ok(EnumDef {
            name,
            type_params,
            variants,
        })
    }

    /// Convert expression to type string (for annotations)
    fn expr_to_type_string(&self, expr: &QuicheExpr) -> String {
        match expr {
            QuicheExpr::Name(n) => n.clone(),
            QuicheExpr::Subscript { value, slice } => {
                format!(
                    "{}<{}>",
                    self.expr_to_type_string(value),
                    self.expr_to_type_string(slice)
                )
            }
            QuicheExpr::Attribute { value, attr } => {
                format!("{}.{}", self.expr_to_type_string(value), attr)
            }
            QuicheExpr::Tuple(elems) => {
                let parts: Vec<String> =
                    elems.iter().map(|e| self.expr_to_type_string(e)).collect();
                parts.join(", ")
            }
            QuicheExpr::Constant(Constant::Str(s)) => s.clone(),
            _ => format!("{:?}", expr),
        }
    }

    /// Check if this annotated assignment should be a constant definition.
    /// Returns (is_const, inner_type) where inner_type is Some if Const[T] was used.
    fn check_const_annotation(
        &self,
        target: &QuicheExpr,
        annotation: &QuicheExpr,
    ) -> (bool, Option<Box<QuicheExpr>>) {
        // Check 1: Is the type annotation Const[T]?
        if let QuicheExpr::Subscript { value, slice } = annotation {
            if let QuicheExpr::Name(type_name) = value.as_ref() {
                if type_name == "Const" {
                    return (true, Some(slice.clone()));
                }
            }
        }

        // Check 2: Is the identifier ALL_UPPER_CASE (SCREAMING_SNAKE_CASE)?
        if let QuicheExpr::Name(name) = target {
            if Self::is_screaming_snake_case(name) {
                return (true, None);
            }
        }

        (false, None)
    }

    /// Check if a string is SCREAMING_SNAKE_CASE (all uppercase with underscores)
    fn is_screaming_snake_case(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }
        // Must start with uppercase letter
        let mut chars = s.chars().peekable();
        if !chars.next().map_or(false, |c| c.is_ascii_uppercase()) {
            return false;
        }
        // Rest must be uppercase letters, digits, or underscores
        // And must contain at least one more character to avoid single letters
        let mut has_multiple = false;
        for c in chars {
            has_multiple = true;
            if !c.is_ascii_uppercase() && !c.is_ascii_digit() && c != '_' {
                return false;
            }
        }
        // Require at least 2 characters to avoid matching 'T', 'U', etc.
        has_multiple
    }

    /// Parse if statement
    fn parse_if_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::If)?;
        let test = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let mut orelse = Vec::new();

        // Handle elif chains - build from inside out
        let mut elif_chain: Vec<(Box<QuicheExpr>, Vec<QuicheStmt>)> = Vec::new();
        while self.check_keyword(Keyword::Elif) {
            self.advance()?;
            let elif_test = Box::new(self.parse_expression()?);
            self.expect(&TokenKind::Colon)?;
            let elif_body = self.parse_block()?;
            elif_chain.push((elif_test, elif_body));
        }

        // Handle else
        if self.check_keyword(Keyword::Else) {
            self.advance()?;
            self.expect(&TokenKind::Colon)?;
            orelse = self.parse_block()?;
        }

        // Build elif chain from inside out (last elif wraps the else, etc.)
        for (elif_test, elif_body) in elif_chain.into_iter().rev() {
            orelse = vec![QuicheStmt::If(IfStmt {
                test: elif_test,
                body: elif_body,
                orelse,
            })];
        }

        Ok(QuicheStmt::If(IfStmt { test, body, orelse }))
    }

    /// Parse while statement
    fn parse_while_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::While)?;
        let test = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let orelse = if self.check_keyword(Keyword::Else) {
            self.advance()?;
            self.expect(&TokenKind::Colon)?;
            self.parse_block()?
        } else {
            Vec::new()
        };

        Ok(QuicheStmt::While(WhileStmt { test, body, orelse }))
    }

    /// Parse for statement
    fn parse_for_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::For)?;
        // Parse target as a simple expression (id, tuple, etc.) - not full expression
        // to avoid the comparison parser consuming 'in' as an operator
        let target = Box::new(self.parse_for_target()?);
        self.expect_keyword(Keyword::In)?;
        let iter = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        let orelse = if self.check_keyword(Keyword::Else) {
            self.advance()?;
            self.expect(&TokenKind::Colon)?;
            self.parse_block()?
        } else {
            Vec::new()
        };

        Ok(QuicheStmt::For(ForStmt {
            target,
            iter,
            body,
            orelse,
        }))
    }

    /// Parse for loop target (simple expression that doesn't consume 'in')
    fn parse_for_target(&mut self) -> Result<QuicheExpr, ParseError> {
        // For targets are typically: i, (a, b), x.attr, x[i]
        // We parse postfix expressions only (no operators)
        let mut expr = self.parse_atom()?;

        loop {
            match self.current_kind() {
                TokenKind::LBracket => {
                    self.advance()?;
                    let slice = self.parse_expression()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = QuicheExpr::Subscript {
                        value: Box::new(expr),
                        slice: Box::new(slice),
                    };
                }
                TokenKind::Dot => {
                    self.advance()?;
                    let attr = self.expect_ident()?;
                    expr = QuicheExpr::Attribute {
                        value: Box::new(expr),
                        attr,
                    };
                }
                TokenKind::Comma => {
                    // Tuple unpacking: for a, b in ...
                    let mut elements = vec![expr];
                    while self.eat(&TokenKind::Comma)? {
                        if self.check_keyword(Keyword::In) {
                            break;
                        }
                        elements.push(self.parse_atom()?);
                    }
                    return Ok(QuicheExpr::Tuple(elements));
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse return statement
    fn parse_return_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::Return)?;
        let value = if !matches!(self.current_kind(), TokenKind::Newline | TokenKind::Eof) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(QuicheStmt::Return(value))
    }

    /// Parse import statement
    fn parse_import_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::Import)?;
        let names = self.parse_import_names()?;
        Ok(QuicheStmt::Import(Import { names }))
    }

    /// Parse from ... import statement
    fn parse_from_import_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::From)?;

        // Count leading dots for relative imports
        let mut level = 0u32;
        while self.check(&TokenKind::Dot) {
            self.advance()?;
            level += 1;
        }

        // Module name (optional after dots)
        let module = if let TokenKind::Ident(_) = self.current_kind() {
            Some(self.parse_dotted_name()?)
        } else {
            None
        };

        self.expect_keyword(Keyword::Import)?;
        let names = self.parse_import_names()?;

        Ok(QuicheStmt::ImportFrom(ImportFrom {
            module,
            names,
            level,
        }))
    }

    /// Parse import names (a, b as c, d)
    fn parse_import_names(&mut self) -> Result<Vec<Alias>, ParseError> {
        let mut names = Vec::new();

        loop {
            let name = self.parse_dotted_name()?;
            let asname = if self.check_keyword(Keyword::As) {
                self.advance()?;
                Some(self.expect_ident()?)
            } else {
                None
            };
            names.push(Alias { name, asname });

            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        Ok(names)
    }

    /// Parse dotted name (a.b.c)
    fn parse_dotted_name(&mut self) -> Result<String, ParseError> {
        let mut name = self.expect_ident()?;
        while self.eat(&TokenKind::Dot)? {
            name.push('.');
            name.push_str(&self.expect_ident()?);
        }
        Ok(name)
    }

    /// Parse match statement
    fn parse_match_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::Match)?;
        let subject = Box::new(self.parse_expression()?);
        self.expect(&TokenKind::Colon)?;
        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut cases = Vec::new();
        while self.check_keyword(Keyword::Case) {
            cases.push(self.parse_match_case()?);
        }

        self.expect(&TokenKind::Dedent)?;
        Ok(QuicheStmt::Match(MatchStmt { subject, cases }))
    }

    /// Parse single match case
    fn parse_match_case(&mut self) -> Result<MatchCase, ParseError> {
        self.expect_keyword(Keyword::Case)?;
        let pattern = self.parse_pattern()?;

        let guard = if self.check_keyword(Keyword::If) {
            self.advance()?;
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.expect(&TokenKind::Colon)?;
        let body = self.parse_block()?;

        Ok(MatchCase {
            pattern,
            guard,
            body,
        })
    }

    /// Parse pattern (simplified)
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        // Check for wildcard _
        if let TokenKind::Ident(name) = self.current_kind() {
            if name == "_" {
                self.advance()?;
                return Ok(Pattern::MatchAs {
                    pattern: None,
                    name: None,
                });
            }
        }

        // Parse as expression for now (simplified)
        let expr = self.parse_expression()?;

        // Check for "as name"
        if self.check_keyword(Keyword::As) {
            self.advance()?;
            let name = self.expect_ident()?;
            return Ok(Pattern::MatchAs {
                pattern: Some(Box::new(Pattern::MatchValue(Box::new(expr)))),
                name: Some(name),
            });
        }

        // Check for class pattern: Name(...) or Name(key=value, ...)
        if let QuicheExpr::Call {
            func,
            args,
            keywords,
        } = expr
        {
            // Positional arguments become patterns
            let patterns = args
                .into_iter()
                .map(|a| Pattern::MatchValue(Box::new(a)))
                .collect();

            // Keyword arguments become kwd_attrs and kwd_patterns
            let (kwd_attrs, kwd_patterns): (Vec<_>, Vec<_>) = keywords
                .into_iter()
                .filter_map(|kw| {
                    kw.arg.map(|name| {
                        let binding_name = match *kw.value {
                            // If the value is a simple Name, use that as the binding name
                            QuicheExpr::Name(ref n) => n.clone(),
                            // Otherwise, use the key name as the binding
                            _ => name.clone(),
                        };
                        (
                            name,
                            Pattern::MatchAs {
                                pattern: None,
                                name: Some(binding_name),
                            },
                        )
                    })
                })
                .unzip();

            return Ok(Pattern::MatchClass(MatchClassPattern {
                cls: func,
                patterns,
                kwd_attrs,
                kwd_patterns,
            }));
        }

        Ok(Pattern::MatchValue(Box::new(expr)))
    }

    /// Parse assert statement
    fn parse_assert_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        self.expect_keyword(Keyword::Assert)?;
        let test = Box::new(self.parse_expression()?);
        let msg = if self.eat(&TokenKind::Comma)? {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(QuicheStmt::Assert(AssertStmt { test, msg }))
    }

    /// Parse expression statement or assignment
    fn parse_expr_or_assign_stmt(&mut self) -> Result<QuicheStmt, ParseError> {
        let expr = self.parse_expression()?;

        // Check for rust(...) call
        if let QuicheExpr::Call { func, args, .. } = &expr {
            if let QuicheExpr::Name(n) = func.as_ref() {
                if n == "rust" {
                    if let Some(QuicheExpr::Constant(Constant::Str(code))) = args.first() {
                        return Ok(QuicheStmt::RustBlock(code.clone()));
                    }
                }
            }
        }

        // Assignment: x = ...
        if self.check(&TokenKind::Eq) {
            self.advance()?;
            let value = Box::new(self.parse_expression()?);
            return Ok(QuicheStmt::Assign(Assign {
                targets: vec![expr],
                value,
            }));
        }

        // Annotated assignment: x: Type = ...
        if self.check(&TokenKind::Colon) {
            self.advance()?;
            let annotation = Box::new(self.parse_expression()?);
            let value = if self.eat(&TokenKind::Eq)? {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };

            // Check if this should be a ConstDef:
            // 1. Identifier is ALL_UPPER_CASE (SCREAMING_SNAKE_CASE)
            // 2. Type annotation is Const[T]
            let (is_const, inner_type) = self.check_const_annotation(&expr, &annotation);

            if is_const {
                // Validate: constants must have a value
                let const_value = match value {
                    Some(v) => v,
                    None => {
                        return Err(
                            self.error("Constants must have an initializer value".to_string())
                        );
                    }
                };

                // Get the name from the expression
                let name = match &expr {
                    QuicheExpr::Name(n) => n.clone(),
                    _ => {
                        return Err(
                            self.error("Constant name must be a simple identifier".to_string())
                        );
                    }
                };

                return Ok(QuicheStmt::ConstDef(ConstDef {
                    name,
                    ty: inner_type.unwrap_or(annotation),
                    value: const_value,
                }));
            }

            return Ok(QuicheStmt::AnnAssign(AnnAssign {
                target: Box::new(expr),
                annotation,
                value,
            }));
        }

        // Augmented assignment: x += ...
        if let Some(op) = self.check_aug_assign() {
            self.advance()?;
            let right = self.parse_expression()?;
            // Convert to x = x op right
            return Ok(QuicheStmt::Assign(Assign {
                targets: vec![expr.clone()],
                value: Box::new(QuicheExpr::BinOp {
                    left: Box::new(expr),
                    op,
                    right: Box::new(right),
                }),
            }));
        }

        Ok(QuicheStmt::Expr(Box::new(expr)))
    }

    /// Check for augmented assignment operators
    fn check_aug_assign(&self) -> Option<Operator> {
        match self.current_kind() {
            TokenKind::PlusEq => Some(Operator::Add),
            TokenKind::MinusEq => Some(Operator::Sub),
            TokenKind::StarEq => Some(Operator::Mult),
            TokenKind::SlashEq => Some(Operator::Div),
            TokenKind::PercentEq => Some(Operator::Mod),
            TokenKind::DoubleStarEq => Some(Operator::Pow),
            TokenKind::DoubleSlashEq => Some(Operator::FloorDiv),
            TokenKind::PipeEq => Some(Operator::BitOr),
            TokenKind::AmpEq => Some(Operator::BitAnd),
            TokenKind::CaretEq => Some(Operator::BitXor),
            TokenKind::LShiftEq => Some(Operator::LShift),
            TokenKind::RShiftEq => Some(Operator::RShift),
            _ => None,
        }
    }

    /// Parse indented block or inline statement
    fn parse_block(&mut self) -> Result<Vec<QuicheStmt>, ParseError> {
        // Check for inline statement (e.g., "def foo(): pass" or "case Ok(v): return v")
        if !matches!(self.current_kind(), TokenKind::Newline) {
            // Inline statement - parse single statement
            let stmt = self.parse_simple_statement()?;
            // Consume trailing newline if present
            self.eat(&TokenKind::Newline)?;
            return Ok(vec![stmt]);
        }

        self.expect(&TokenKind::Newline)?;
        self.expect(&TokenKind::Indent)?;

        let mut stmts = Vec::new();
        while !matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
            self.skip_newlines()?;
            if matches!(self.current_kind(), TokenKind::Dedent | TokenKind::Eof) {
                break;
            }
            stmts.push(self.parse_statement()?);
            self.skip_newlines()?;
        }

        if self.check(&TokenKind::Dedent) {
            self.advance()?;
        }
        Ok(stmts)
    }

    /// Parse a simple (inline) statement - no compound statements
    fn parse_simple_statement(&mut self) -> Result<QuicheStmt, ParseError> {
        match self.current_kind() {
            TokenKind::Keyword(Keyword::Pass) => {
                self.advance()?;
                Ok(QuicheStmt::Pass)
            }
            TokenKind::Keyword(Keyword::Break) => {
                self.advance()?;
                Ok(QuicheStmt::Break)
            }
            TokenKind::Keyword(Keyword::Continue) => {
                self.advance()?;
                Ok(QuicheStmt::Continue)
            }
            TokenKind::Keyword(Keyword::Return) => self.parse_return_stmt(),
            TokenKind::Keyword(Keyword::Assert) => self.parse_assert_stmt(),
            _ => self.parse_expr_or_assign_stmt(),
        }
    }

    /// Parse type parameters [T, U, V]
    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.eat(&TokenKind::LBracket)? {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        loop {
            params.push(self.expect_ident()?);
            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        self.expect(&TokenKind::RBracket)?;
        Ok(params)
    }

    /// Parse function arguments
    fn parse_args(&mut self) -> Result<Vec<Arg>, ParseError> {
        let mut args = Vec::new();

        while !self.check(&TokenKind::RParen) {
            let arg = self.expect_ident()?;
            let annotation = if self.eat(&TokenKind::Colon)? {
                Some(Box::new(self.parse_expression()?))
            } else {
                None
            };
            args.push(Arg { arg, annotation });

            // Skip default value for now
            if self.eat(&TokenKind::Eq)? {
                let _ = self.parse_expression()?;
            }

            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        Ok(args)
    }

    /// Expect and consume an identifier
    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.current_kind().clone() {
            TokenKind::Ident(name) => {
                self.advance()?;
                Ok(name)
            }
            _ => Err(self.error(format!(
                "Expected identifier, got {:?}",
                self.current_kind()
            ))),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Expression parsing (Pratt parser style)
    // ─────────────────────────────────────────────────────────────────────────

    /// Parse expression
    pub fn parse_expression(&mut self) -> Result<QuicheExpr, ParseError> {
        self.parse_ternary()
    }

    /// Parse ternary: expr if cond else expr
    fn parse_ternary(&mut self) -> Result<QuicheExpr, ParseError> {
        let body = self.parse_or()?;

        if self.check_keyword(Keyword::If) {
            self.advance()?;
            let test = Box::new(self.parse_or()?);
            self.expect_keyword(Keyword::Else)?;
            let orelse = Box::new(self.parse_ternary()?);
            return Ok(QuicheExpr::IfExp {
                test,
                body: Box::new(body),
                orelse,
            });
        }

        Ok(body)
    }

    /// Parse or: a or b
    fn parse_or(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_and()?;

        while self.check_keyword(Keyword::Or) {
            self.advance()?;
            let right = self.parse_and()?;
            left = QuicheExpr::BoolOp {
                op: BoolOperator::Or,
                values: vec![left, right],
            };
        }

        Ok(left)
    }

    /// Parse and: a and b
    fn parse_and(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_not()?;

        while self.check_keyword(Keyword::And) {
            self.advance()?;
            let right = self.parse_not()?;
            left = QuicheExpr::BoolOp {
                op: BoolOperator::And,
                values: vec![left, right],
            };
        }

        Ok(left)
    }

    /// Parse not: not a
    fn parse_not(&mut self) -> Result<QuicheExpr, ParseError> {
        if self.check_keyword(Keyword::Not) {
            self.advance()?;
            let operand = Box::new(self.parse_not()?);
            return Ok(QuicheExpr::UnaryOp {
                op: UnaryOperator::Not,
                operand,
            });
        }
        self.parse_comparison()
    }

    /// Parse comparison: a < b < c
    fn parse_comparison(&mut self) -> Result<QuicheExpr, ParseError> {
        let left = self.parse_bitor()?;

        let mut ops = Vec::new();
        let mut comparators = Vec::new();

        loop {
            let op = match self.current_kind() {
                TokenKind::EqEq => Some(CmpOperator::Eq),
                TokenKind::NotEq => Some(CmpOperator::NotEq),
                TokenKind::Lt => Some(CmpOperator::Lt),
                TokenKind::LtEq => Some(CmpOperator::LtE),
                TokenKind::Gt => Some(CmpOperator::Gt),
                TokenKind::GtEq => Some(CmpOperator::GtE),
                TokenKind::Keyword(Keyword::Is) => {
                    self.advance()?;
                    if self.check_keyword(Keyword::Not) {
                        self.advance()?;
                        Some(CmpOperator::IsNot)
                    } else {
                        Some(CmpOperator::Is)
                    }
                }
                TokenKind::Keyword(Keyword::In) => Some(CmpOperator::In),
                TokenKind::Keyword(Keyword::Not) => {
                    // Check for "not in"
                    let peek = self.peek()?;
                    if matches!(&peek.kind, TokenKind::Keyword(Keyword::In)) {
                        self.advance()?; // consume "not"
                        self.advance()?; // consume "in"
                        Some(CmpOperator::NotIn)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(cmp_op) = op {
                if !matches!(
                    self.current_kind(),
                    TokenKind::Keyword(Keyword::Is | Keyword::Not)
                ) {
                    self.advance()?;
                }
                ops.push(cmp_op);
                comparators.push(self.parse_bitor()?);
            } else {
                break;
            }
        }

        if ops.is_empty() {
            // Check for "as" cast after expression
            if self.check_keyword(Keyword::As) {
                self.advance()?; // consume "as"
                let target_type = self.parse_unary()?; // Parse type as primary expression
                return Ok(QuicheExpr::Cast {
                    expr: Box::new(left),
                    target_type: Box::new(target_type),
                });
            }
            Ok(left)
        } else {
            let result = QuicheExpr::Compare {
                left: Box::new(left),
                ops,
                comparators,
            };
            // Check for "as" cast after comparison expression
            if self.check_keyword(Keyword::As) {
                self.advance()?; // consume "as"
                let target_type = self.parse_unary()?;
                return Ok(QuicheExpr::Cast {
                    expr: Box::new(result),
                    target_type: Box::new(target_type),
                });
            }
            Ok(result)
        }
    }

    /// Parse bitwise or: a | b
    fn parse_bitor(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_bitxor()?;
        while self.check(&TokenKind::Pipe) {
            self.advance()?;
            let right = self.parse_bitxor()?;
            left = QuicheExpr::BinOp {
                left: Box::new(left),
                op: Operator::BitOr,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Parse bitwise xor: a ^ b
    fn parse_bitxor(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_bitand()?;
        while self.check(&TokenKind::Caret) {
            self.advance()?;
            let right = self.parse_bitand()?;
            left = QuicheExpr::BinOp {
                left: Box::new(left),
                op: Operator::BitXor,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Parse bitwise and: a & b
    fn parse_bitand(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_shift()?;
        while self.check(&TokenKind::Amp) {
            self.advance()?;
            let right = self.parse_shift()?;
            left = QuicheExpr::BinOp {
                left: Box::new(left),
                op: Operator::BitAnd,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Parse shift: a << b, a >> b
    fn parse_shift(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.current_kind() {
                TokenKind::LShift => Some(Operator::LShift),
                TokenKind::RShift => Some(Operator::RShift),
                _ => None,
            };
            if let Some(op) = op {
                self.advance()?;
                let right = self.parse_additive()?;
                left = QuicheExpr::BinOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse additive: a + b, a - b
    fn parse_additive(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.current_kind() {
                TokenKind::Plus => Some(Operator::Add),
                TokenKind::Minus => Some(Operator::Sub),
                _ => None,
            };
            if let Some(op) = op {
                self.advance()?;
                let right = self.parse_multiplicative()?;
                left = QuicheExpr::BinOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse multiplicative: a * b, a / b, a % b, a // b
    fn parse_multiplicative(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.current_kind() {
                TokenKind::Star => Some(Operator::Mult),
                TokenKind::Slash => Some(Operator::Div),
                TokenKind::Percent => Some(Operator::Mod),
                TokenKind::DoubleSlash => Some(Operator::FloorDiv),
                _ => None,
            };
            if let Some(op) = op {
                self.advance()?;
                let right = self.parse_unary()?;
                left = QuicheExpr::BinOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    /// Parse unary: -a, +a, ~a
    fn parse_unary(&mut self) -> Result<QuicheExpr, ParseError> {
        match self.current_kind() {
            TokenKind::Minus => {
                self.advance()?;
                Ok(QuicheExpr::UnaryOp {
                    op: UnaryOperator::USub,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            TokenKind::Plus => {
                self.advance()?;
                Ok(QuicheExpr::UnaryOp {
                    op: UnaryOperator::UAdd,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            TokenKind::Tilde => {
                self.advance()?;
                Ok(QuicheExpr::UnaryOp {
                    op: UnaryOperator::Invert,
                    operand: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_power(),
        }
    }

    /// Parse power: a ** b (right associative)
    fn parse_power(&mut self) -> Result<QuicheExpr, ParseError> {
        let base = self.parse_postfix()?;
        if self.check(&TokenKind::DoubleStar) {
            self.advance()?;
            let exp = self.parse_unary()?; // Right associative
            Ok(QuicheExpr::BinOp {
                left: Box::new(base),
                op: Operator::Pow,
                right: Box::new(exp),
            })
        } else {
            Ok(base)
        }
    }

    /// Parse subscript content - handles both regular indexing and slice syntax
    /// Syntax: [expr], [start..], [..end], [start..end], [expr, expr, ...]
    fn parse_subscript_content(&mut self) -> Result<QuicheExpr, ParseError> {
        // Check for [..end] - slice starting with DotDot
        if self.check(&TokenKind::DotDot) {
            self.advance()?;
            let upper = if self.check(&TokenKind::RBracket) {
                None // [..]
            } else {
                Some(Box::new(self.parse_expression()?)) // [..end]
            };
            return Ok(QuicheExpr::Slice {
                lower: None,
                upper,
                step: None,
            });
        }

        // Parse first expression
        let first = self.parse_expression()?;

        // Check for slice syntax: [start..]  or  [start..end]
        if self.check(&TokenKind::DotDot) {
            self.advance()?;
            let upper = if self.check(&TokenKind::RBracket) {
                None // [start..]
            } else {
                Some(Box::new(self.parse_expression()?)) // [start..end]
            };
            return Ok(QuicheExpr::Slice {
                lower: Some(Box::new(first)),
                upper,
                step: None,
            });
        }

        // Handle comma-separated type parameters (e.g., HashMap[String, bool])
        if self.check(&TokenKind::Comma) {
            let mut elements = vec![first];
            while self.eat(&TokenKind::Comma)? {
                if self.check(&TokenKind::RBracket) {
                    break; // trailing comma
                }
                elements.push(self.parse_expression()?);
            }
            return Ok(QuicheExpr::Tuple(elements));
        }

        // Regular single-element subscript
        Ok(first)
    }

    /// Parse postfix: calls, subscripts, attributes
    fn parse_postfix(&mut self) -> Result<QuicheExpr, ParseError> {
        let mut expr = self.parse_atom()?;

        loop {
            match self.current_kind() {
                TokenKind::LParen => {
                    self.advance()?;
                    let (args, keywords) = self.parse_call_args()?;
                    self.expect(&TokenKind::RParen)?;
                    expr = QuicheExpr::Call {
                        func: Box::new(expr),
                        args,
                        keywords,
                    };
                }
                TokenKind::LBracket => {
                    self.advance()?;
                    // Check for slice syntax: [..], [start..], [..end], [start..end]
                    let slice_expr = self.parse_subscript_content()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = QuicheExpr::Subscript {
                        value: Box::new(expr),
                        slice: Box::new(slice_expr),
                    };
                }
                TokenKind::Dot => {
                    self.advance()?;
                    let attr = self.expect_ident()?;
                    expr = QuicheExpr::Attribute {
                        value: Box::new(expr),
                        attr,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    /// Parse call arguments (positional and keyword)
    fn parse_call_args(
        &mut self,
    ) -> Result<(Vec<QuicheExpr>, Vec<crate::ast::Keyword>), ParseError> {
        let mut args = Vec::new();
        let mut keywords = Vec::new();

        while !self.check(&TokenKind::RParen) {
            // Check for keyword argument: name=value
            let is_kwarg = if let TokenKind::Ident(_) = self.current_kind() {
                let peek = self.peek()?;
                matches!(peek.kind, TokenKind::Eq)
            } else {
                false
            };

            if is_kwarg {
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Eq)?;
                let value = self.parse_expression()?;
                keywords.push(crate::ast::Keyword {
                    arg: Some(name),
                    value: Box::new(value),
                });
            } else {
                args.push(self.parse_expression()?);
            }

            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }

        Ok((args, keywords))
    }

    /// Parse atom (literals, names, parens, lists, etc.)
    fn parse_atom(&mut self) -> Result<QuicheExpr, ParseError> {
        match self.current_kind().clone() {
            TokenKind::Ident(name) => {
                self.advance()?;
                Ok(QuicheExpr::Name(name))
            }
            TokenKind::Int(n) => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::Int(n)))
            }
            TokenKind::Float(f) => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::Float(f)))
            }
            TokenKind::String(s) => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::Str(s)))
            }
            TokenKind::FString { content, .. } => {
                self.advance()?;
                let parts = crate::fstring::parse_fstring_content(&content, self)?;
                Ok(QuicheExpr::FString(parts))
            }
            TokenKind::Keyword(Keyword::True) => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::Bool(true)))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::Bool(false)))
            }
            TokenKind::Keyword(Keyword::None) => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::NoneVal))
            }
            TokenKind::Ellipsis => {
                self.advance()?;
                Ok(QuicheExpr::Constant(Constant::Ellipsis))
            }
            TokenKind::LParen => {
                self.advance()?;
                if self.check(&TokenKind::RParen) {
                    self.advance()?;
                    return Ok(QuicheExpr::Tuple(Vec::new()));
                }
                let expr = self.parse_expression()?;
                if self.check(&TokenKind::Comma) {
                    // Tuple
                    let mut elements = vec![expr];
                    while self.eat(&TokenKind::Comma)? {
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(QuicheExpr::Tuple(elements))
                } else {
                    self.expect(&TokenKind::RParen)?;
                    Ok(expr)
                }
            }
            TokenKind::LBracket => {
                self.advance()?;
                // Check for list comprehension: [expr for x in iter]
                if !self.check(&TokenKind::RBracket) {
                    let first = self.parse_expression()?;
                    if self.check_keyword(Keyword::For) {
                        // List comprehension
                        let generators = self.parse_comprehension_generators()?;
                        self.expect(&TokenKind::RBracket)?;
                        return Ok(QuicheExpr::ListComp {
                            element: Box::new(first),
                            generators,
                        });
                    }
                    // Regular list literal
                    let mut elements = vec![first];
                    while self.eat(&TokenKind::Comma)? {
                        if self.check(&TokenKind::RBracket) {
                            break; // trailing comma
                        }
                        elements.push(self.parse_expression()?);
                    }
                    self.expect(&TokenKind::RBracket)?;
                    Ok(QuicheExpr::List(elements))
                } else {
                    self.advance()?; // consume ]
                    Ok(QuicheExpr::List(Vec::new()))
                }
            }
            TokenKind::LBrace => {
                self.advance()?;
                // Check for dict comprehension: {k: v for x in iter}
                if !self.check(&TokenKind::RBrace) {
                    let first_key = self.parse_expression()?;
                    if self.check(&TokenKind::Colon) {
                        self.advance()?; // consume :
                        let first_value = self.parse_expression()?;
                        if self.check_keyword(Keyword::For) {
                            // Dict comprehension
                            let generators = self.parse_comprehension_generators()?;
                            self.expect(&TokenKind::RBrace)?;
                            return Ok(QuicheExpr::DictComp {
                                key: Box::new(first_key),
                                value: Box::new(first_value),
                                generators,
                            });
                        }
                        // Regular dict literal (not yet implemented - fall through to error)
                    }
                }
                Err(self.error("Dict literals not yet supported, use HashMap::new()".to_string()))
            }
            TokenKind::Pipe => {
                // Rust-style closure: |x: T, y: U| expr or |x: T| -> R { stmts }
                self.parse_closure()
            }
            TokenKind::Keyword(Keyword::Lambda) => {
                // Python-style lambda (deprecated, kept for backwards compat)
                self.advance()?;
                let mut args = Vec::new();
                while !self.check(&TokenKind::Colon) {
                    let name = self.expect_ident()?;
                    args.push(LambdaArg { name, ty: None });
                    if !self.eat(&TokenKind::Comma)? {
                        break;
                    }
                }
                self.expect(&TokenKind::Colon)?;
                let body = Box::new(self.parse_expression()?);
                Ok(QuicheExpr::Lambda {
                    args,
                    return_type: None,
                    body: LambdaBody::Expr(body),
                })
            }
            _ => Err(self.error(format!("Unexpected token: {:?}", self.current_kind()))),
        }
    }

    /// Parse a Rust-style closure: |x: T, y: U| expr or |x: T| -> R { stmts }
    fn parse_closure(&mut self) -> Result<QuicheExpr, ParseError> {
        self.expect(&TokenKind::Pipe)?; // consume opening |

        // Parse arguments
        let mut args = Vec::new();
        while !self.check(&TokenKind::Pipe) {
            let name = self.expect_ident()?;
            // Parse type annotation if present, use restricted parsing (stop at |, ,)
            let ty = if self.eat(&TokenKind::Colon)? {
                Some(Box::new(self.parse_type_annotation()?))
            } else {
                None
            };
            args.push(LambdaArg { name, ty });
            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        self.expect(&TokenKind::Pipe)?; // consume closing |

        // Check for return type: -> Type
        let return_type = if self.check(&TokenKind::Arrow) {
            self.advance()?;
            Some(Box::new(self.parse_type_annotation()?))
        } else {
            None
        };

        // Check for block body: { stmts }
        let body = if self.check(&TokenKind::LBrace) {
            self.advance()?;
            let mut stmts = Vec::new();
            self.skip_newlines()?;
            while !self.check(&TokenKind::RBrace) {
                stmts.push(self.parse_statement()?);
                self.skip_newlines()?;
            }
            self.expect(&TokenKind::RBrace)?;
            LambdaBody::Block(stmts)
        } else {
            // Single expression body
            LambdaBody::Expr(Box::new(self.parse_expression()?))
        };

        Ok(QuicheExpr::Lambda {
            args,
            return_type,
            body,
        })
    }

    /// Parse a type annotation (stops at operators like |, ,, etc.)
    fn parse_type_annotation(&mut self) -> Result<QuicheExpr, ParseError> {
        // Types are typically Name, Name[T], or Name.Attr patterns
        // Use postfix parsing which handles these without binary operators
        self.parse_postfix()
    }

    /// Parse comprehension generators: for x in iter [if cond] [for y in iter2 ...]
    fn parse_comprehension_generators(&mut self) -> Result<Vec<Comprehension>, ParseError> {
        let mut generators = Vec::new();

        while self.check_keyword(Keyword::For) {
            self.advance()?; // consume 'for'
            // Parse target - typically a simple name or tuple pattern
            let target = Box::new(self.parse_comp_target()?);
            self.expect_keyword(Keyword::In)?;
            // Parse iterator - stop at 'if', 'for', ']', '}'
            let iter = Box::new(self.parse_comp_expr()?);

            // Parse optional 'if' conditions
            let mut ifs = Vec::new();
            while self.check_keyword(Keyword::If) {
                self.advance()?;
                // Parse condition - stop at 'if', 'for', ']', '}'
                ifs.push(self.parse_comp_expr()?);
            }

            generators.push(Comprehension { target, iter, ifs });
        }

        Ok(generators)
    }

    /// Parse an expression in comprehension context (stops at for/if/]/})
    fn parse_comp_expr(&mut self) -> Result<QuicheExpr, ParseError> {
        // Parse until we hit a comprehension boundary (for, if, ], })
        // This is essentially parse_or() level and below
        self.parse_or()
    }

    /// Parse a comprehension target (typically Name or Tuple)
    fn parse_comp_target(&mut self) -> Result<QuicheExpr, ParseError> {
        // Check for tuple pattern: (a, b) or a, b
        if self.check(&TokenKind::LParen) {
            self.advance()?;
            let mut elements = vec![self.parse_atom()?];
            while self.eat(&TokenKind::Comma)? {
                if self.check(&TokenKind::RParen) {
                    break;
                }
                elements.push(self.parse_atom()?);
            }
            self.expect(&TokenKind::RParen)?;
            Ok(QuicheExpr::Tuple(elements))
        } else {
            // Simple name
            self.parse_atom()
        }
    }

    /// Parse comma-separated expression list
    fn parse_expr_list(&mut self) -> Result<Vec<QuicheExpr>, ParseError> {
        let mut exprs = Vec::new();
        while !matches!(
            self.current_kind(),
            TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace
        ) {
            exprs.push(self.parse_expression()?);
            if !self.eat(&TokenKind::Comma)? {
                break;
            }
        }
        Ok(exprs)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Parse source code into a QuicheModule
pub fn parse_new(source: &str) -> Result<QuicheModule, ParseError> {
    let mut parser = Parser::new(source)?;
    parser.parse_module()
}

/// Parse source code into a QuicheModule (drop-in replacement for ruff-based parser)
pub fn parse(source: &str) -> Result<QuicheModule, ParseError> {
    parse_new(source)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_expression() {
        let module = parse_new("x + 1").unwrap();
        assert_eq!(module.body.len(), 1);
        match &module.body[0] {
            QuicheStmt::Expr(e) => match e.as_ref() {
                QuicheExpr::BinOp { op, .. } => assert_eq!(*op, Operator::Add),
                _ => panic!("Expected BinOp"),
            },
            _ => panic!("Expected Expr"),
        }
    }

    #[test]
    fn test_parse_function_def() {
        let source = "def foo(x: int) -> int:\n    return x + 1\n";
        let module = parse_new(source).unwrap();
        assert_eq!(module.body.len(), 1);
        match &module.body[0] {
            QuicheStmt::FunctionDef(f) => {
                assert_eq!(f.name, "foo");
                assert_eq!(f.args.len(), 1);
                assert!(f.returns.is_some());
            }
            _ => panic!("Expected FunctionDef"),
        }
    }

    #[test]
    fn test_parse_struct() {
        let source = "class Point(Struct):\n    x: int\n    y: int\n";
        let module = parse_new(source).unwrap();
        match &module.body[0] {
            QuicheStmt::StructDef(s) => {
                assert_eq!(s.name, "Point");
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("Expected StructDef"),
        }
    }

    #[test]
    fn test_parse_if() {
        let source = "if x:\n    y = 1\nelse:\n    y = 2\n";
        let module = parse_new(source).unwrap();
        match &module.body[0] {
            QuicheStmt::If(if_stmt) => {
                assert_eq!(if_stmt.body.len(), 1);
                assert_eq!(if_stmt.orelse.len(), 1);
            }
            _ => panic!("Expected If"),
        }
    }

    #[test]
    fn test_parse_for() {
        let source = "for i in range(10):\n    print(i)\n";
        let module = parse_new(source).unwrap();
        match &module.body[0] {
            QuicheStmt::For(_) => {}
            _ => panic!("Expected For"),
        }
    }

    #[test]
    fn test_parse_import() {
        let module = parse_new("import os.path").unwrap();
        match &module.body[0] {
            QuicheStmt::Import(i) => {
                assert_eq!(i.names[0].name, "os.path");
            }
            _ => panic!("Expected Import"),
        }
    }

    #[test]
    fn test_parse_from_import() {
        let module = parse_new("from os import path as p").unwrap();
        match &module.body[0] {
            QuicheStmt::ImportFrom(i) => {
                assert_eq!(i.module, Some("os".to_string()));
                assert_eq!(i.names[0].name, "path");
                assert_eq!(i.names[0].asname, Some("p".to_string()));
            }
            _ => panic!("Expected ImportFrom"),
        }
    }

    #[test]
    fn test_parse_rust_block() {
        let source = r#"rust("println!(\"hello\")")"#;
        let module = parse_new(source).unwrap();
        match &module.body[0] {
            QuicheStmt::RustBlock(code) => {
                assert!(code.contains("println!"));
            }
            _ => panic!("Expected RustBlock"),
        }
    }

    #[test]
    fn test_parse_lambda() {
        let module = parse_new("f = lambda x, y: x + y").unwrap();
        match &module.body[0] {
            QuicheStmt::Assign(a) => match a.value.as_ref() {
                QuicheExpr::Lambda { args, .. } => {
                    assert_eq!(args.len(), 2);
                }
                _ => panic!("Expected Lambda"),
            },
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_parse_operators() {
        let module = parse_new("a and b or not c").unwrap();
        match &module.body[0] {
            QuicheStmt::Expr(e) => match e.as_ref() {
                QuicheExpr::BoolOp {
                    op: BoolOperator::Or,
                    ..
                } => {}
                _ => panic!("Expected Or BoolOp"),
            },
            _ => panic!("Expected Expr"),
        }
    }

    #[test]
    fn test_parse_multiline_docstring_then_if() {
        // This is a regression test for a bug where multi-line docstrings
        // caused the following `if` statement to be parsed as a ternary expression
        let source = r#"def test(self):
    """Multi
    line
    doc"""
    if x > 0:
        pass
"#;
        let module = parse_new(source).expect("Should parse successfully");
        assert_eq!(module.body.len(), 1);
        match &module.body[0] {
            QuicheStmt::FunctionDef(f) => {
                assert_eq!(f.name, "test");
                // Should have docstring and if statement in body
                assert!(
                    f.body.len() >= 2,
                    "Function body should have at least 2 statements"
                );
                // First statement should be docstring (expression)
                match &f.body[0] {
                    QuicheStmt::Expr(_) => {}
                    _ => panic!(
                        "First statement should be docstring expression, got {:?}",
                        f.body[0]
                    ),
                }
                // Second statement should be if
                match &f.body[1] {
                    QuicheStmt::If(_) => {}
                    _ => panic!("Second statement should be If, got {:?}", f.body[1]),
                }
            }
            _ => panic!("Expected FunctionDef"),
        }
    }

    #[test]
    fn test_parse_multiline_function_call() {
        // Test parsing multi-line function calls
        let source = r#"result = foo(
    1,
    2,
    3
)
"#;
        let module = parse_new(source).expect("Should parse multiline function call");
        assert_eq!(module.body.len(), 1);
        match &module.body[0] {
            QuicheStmt::Assign(a) => match a.value.as_ref() {
                QuicheExpr::Call { args, .. } => {
                    assert_eq!(args.len(), 3);
                }
                _ => panic!("Expected Call"),
            },
            _ => panic!("Expected Assign"),
        }
    }

    #[test]
    fn test_parse_single_line_match_case() {
        // Test single-line match cases like "case Ok(v): return v"
        let source = r#"match result:
    case Ok(v): return v
    case Err(e):
        print(e)
"#;
        let module = parse_new(source).expect("Should parse single-line match cases");
        match &module.body[0] {
            QuicheStmt::Match(m) => {
                assert_eq!(m.cases.len(), 2);
                // First case should have a return statement
                match &m.cases[0].body[0] {
                    QuicheStmt::Return(_) => {}
                    _ => panic!(
                        "First case should have Return, got {:?}",
                        m.cases[0].body[0]
                    ),
                }
            }
            _ => panic!("Expected Match"),
        }
    }

    #[test]
    fn test_parse_if_elif_else() {
        // Regression test for elif chain bug - elif branches must not be
        // overwritten by else
        let source = r#"if a > 0:
    x = 1
elif b > 0:
    x = 2
elif c > 0:
    x = 3
else:
    x = 4
"#;
        let module = parse_new(source).expect("Should parse if/elif/else");
        assert_eq!(module.body.len(), 1);

        // Check structure: if -> orelse contains elif -> orelse contains elif -> orelse contains else
        match &module.body[0] {
            QuicheStmt::If(if1) => {
                assert_eq!(if1.body.len(), 1, "if body should have 1 statement");
                assert_eq!(if1.orelse.len(), 1, "if orelse should have 1 elif");

                // First elif
                match &if1.orelse[0] {
                    QuicheStmt::If(elif1) => {
                        assert_eq!(elif1.body.len(), 1, "elif1 body should have 1 statement");
                        assert_eq!(elif1.orelse.len(), 1, "elif1 orelse should have 1 elif");

                        // Second elif
                        match &elif1.orelse[0] {
                            QuicheStmt::If(elif2) => {
                                assert_eq!(
                                    elif2.body.len(),
                                    1,
                                    "elif2 body should have 1 statement"
                                );
                                assert_eq!(
                                    elif2.orelse.len(),
                                    1,
                                    "elif2 orelse should have else body"
                                );

                                // Final else body (not wrapped in If)
                                match &elif2.orelse[0] {
                                    QuicheStmt::Assign(_) => {} // else: x = 4
                                    _ => panic!(
                                        "else body should be Assign, got {:?}",
                                        elif2.orelse[0]
                                    ),
                                }
                            }
                            _ => panic!(
                                "elif1.orelse should be If (elif2), got {:?}",
                                elif1.orelse[0]
                            ),
                        }
                    }
                    _ => panic!("if.orelse should be If (elif1), got {:?}", if1.orelse[0]),
                }
            }
            _ => panic!("Expected If"),
        }
    }
}
