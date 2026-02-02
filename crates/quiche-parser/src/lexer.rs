//! Quiche Lexer - Tokenizes Python/Quiche source code
//!
//! Produces a stream of tokens with position information for the parser.
//! Handles Python's significant whitespace (INDENT/DEDENT tokens).
#![allow(clippy::unwrap_used)]
use regex::Regex;

// ─────────────────────────────────────────────────────────────────────────────
// Token Types
// ─────────────────────────────────────────────────────────────────────────────

/// Keywords in the Quiche language
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    // Definitions
    Def,
    Class,
    // Control flow
    If,
    Elif,
    Else,
    For,
    While,
    Match,
    Case,
    Return,
    Pass,
    Break,
    Continue,
    // Imports
    Import,
    From,
    As,
    // Logical operators (as keywords)
    And,
    Or,
    Not,
    In,
    Is,
    // Literals
    None,
    True,
    False,
    // Exception handling
    Try,
    Except,
    Finally,
    Raise,
    // Other
    With,
    Assert,
    Lambda,
    Yield,
    Global,
    Nonlocal,
    Del,
    Async,
    Await,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Keyword> {
        match s {
            "def" => Some(Keyword::Def),
            "class" => Some(Keyword::Class),
            "if" => Some(Keyword::If),
            "elif" => Some(Keyword::Elif),
            "else" => Some(Keyword::Else),
            "for" => Some(Keyword::For),
            "while" => Some(Keyword::While),
            "match" => Some(Keyword::Match),
            "case" => Some(Keyword::Case),
            "return" => Some(Keyword::Return),
            "pass" => Some(Keyword::Pass),
            "break" => Some(Keyword::Break),
            "continue" => Some(Keyword::Continue),
            "import" => Some(Keyword::Import),
            "from" => Some(Keyword::From),
            "as" => Some(Keyword::As),
            "and" => Some(Keyword::And),
            "or" => Some(Keyword::Or),
            "not" => Some(Keyword::Not),
            "in" => Some(Keyword::In),
            "is" => Some(Keyword::Is),
            "None" => Some(Keyword::None),
            "True" => Some(Keyword::True),
            "False" => Some(Keyword::False),
            "try" => Some(Keyword::Try),
            "except" => Some(Keyword::Except),
            "finally" => Some(Keyword::Finally),
            "raise" => Some(Keyword::Raise),
            "with" => Some(Keyword::With),
            "assert" => Some(Keyword::Assert),
            "lambda" => Some(Keyword::Lambda),
            "yield" => Some(Keyword::Yield),
            "global" => Some(Keyword::Global),
            "nonlocal" => Some(Keyword::Nonlocal),
            "del" => Some(Keyword::Del),
            "async" => Some(Keyword::Async),
            "await" => Some(Keyword::Await),
            _ => Option::None,
        }
    }
}

/// All possible token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Structural tokens
    Newline,
    Indent,
    Dedent,
    Eof,

    // Identifiers and keywords
    Ident(String),
    Keyword(Keyword),

    // Literals
    Int(i64),
    Float(f64),
    String(String),

    // Operators - Arithmetic
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    DoubleSlash, // //
    Percent,     // %
    DoubleStar,  // **
    At,          // @

    // Operators - Comparison
    EqEq,  // ==
    NotEq, // !=
    Lt,    // <
    LtEq,  // <=
    Gt,    // >
    GtEq,  // >=

    // Operators - Assignment
    Eq,            // =
    PlusEq,        // +=
    MinusEq,       // -=
    StarEq,        // *=
    SlashEq,       // /=
    PercentEq,     // %=
    DoubleStarEq,  // **=
    DoubleSlashEq, // //=
    PipeEq,        // |=
    AmpEq,         // &=
    CaretEq,       // ^=
    LShiftEq,      // <<=
    RShiftEq,      // >>=
    AtEq,          // @=
    ColonEq,       // :=

    // Operators - Bitwise
    Pipe,   // |
    Amp,    // &
    Caret,  // ^
    Tilde,  // ~
    LShift, // <<
    RShift, // >>

    // Delimiters
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }
    Colon,     // :
    Comma,     // ,
    Dot,       // .
    Semicolon, // ;
    Arrow,     // ->
    Ellipsis,  // ...

    // Special
    Comment(String),
}

/// A token with position information
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,  // Byte offset
    pub end: usize,    // Byte offset
    pub line: usize,   // 1-indexed
    pub column: usize, // 1-indexed
}

impl Token {
    pub fn new(kind: TokenKind, start: usize, end: usize, line: usize, column: usize) -> Self {
        Token {
            kind,
            start,
            end,
            line,
            column,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Lexer Errors
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub pos: usize,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Lex error at {}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for LexError {}

// ─────────────────────────────────────────────────────────────────────────────
// Lexer Implementation
// ─────────────────────────────────────────────────────────────────────────────

/// Compiled regex patterns for tokenization
struct LexerPatterns {
    ident: Regex,
    int_hex: Regex,
    int_oct: Regex,
    int_bin: Regex,
    int_dec: Regex,
    float_exp: Regex,
    float_simple: Regex,
}

impl LexerPatterns {
    fn new() -> Result<Self, String> {
        Ok(LexerPatterns {
            ident: Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*").map_err(|e| e.to_string())?,
            int_hex: Regex::new(r"^0[xX][0-9a-fA-F_]+").map_err(|e| e.to_string())?,
            int_oct: Regex::new(r"^0[oO][0-7_]+").map_err(|e| e.to_string())?,
            int_bin: Regex::new(r"^0[bB][01_]+").map_err(|e| e.to_string())?,
            int_dec: Regex::new(r"^[0-9][0-9_]*").map_err(|e| e.to_string())?,
            float_exp: Regex::new(r"^[0-9][0-9_]*\.?[0-9_]*[eE][+-]?[0-9_]+")
                .map_err(|e| e.to_string())?,
            float_simple: Regex::new(r"^[0-9][0-9_]*\.[0-9_]*").map_err(|e| e.to_string())?,
        })
    }
}

/// The lexer state
pub struct Lexer<'a> {
    source: &'a str,
    pos: usize,
    line: usize,
    column: usize,
    indent_stack: Vec<usize>,
    pending_dedents: usize,
    at_line_start: bool,
    patterns: LexerPatterns,
    /// Track depth of nested brackets - newlines inside () [] {} are ignored
    bracket_depth: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Result<Self, String> {
        Ok(Lexer {
            source,
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0],
            pending_dedents: 0,
            at_line_start: true,
            patterns: LexerPatterns::new()?,
            bracket_depth: 0,
        })
    }

    /// Get the remaining source from current position
    fn remaining(&self) -> &'a str {
        &self.source[self.pos..]
    }

    /// Peek at the current character
    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Peek at the next character
    fn peek_next(&self) -> Option<char> {
        self.remaining().chars().nth(1)
    }

    /// Advance by one character
    fn advance(&mut self) -> Option<char> {
        if let Some(ch) = self.peek() {
            self.pos += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
                self.at_line_start = true;
            } else {
                self.column += 1;
            }
            Some(ch)
        } else {
            Option::None
        }
    }

    /// Advance by N bytes
    fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            if self.advance().is_none() {
                break;
            }
        }
    }

    /// Skip whitespace (but not newlines)
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip a comment (from # to end of line)
    fn skip_comment(&mut self) -> Option<String> {
        if self.peek() == Some('#') {
            let start = self.pos;
            while let Some(ch) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.advance();
            }
            Some(self.source[start..self.pos].to_string())
        } else {
            Option::None
        }
    }

    /// Calculate indentation at current line start
    fn measure_indent(&mut self) -> usize {
        let mut indent = 0;
        while let Some(ch) = self.peek() {
            match ch {
                ' ' => {
                    indent += 1;
                    self.advance();
                }
                '\t' => {
                    // Tab = advance to next multiple of 8
                    indent = (indent / 8 + 1) * 8;
                    self.advance();
                }
                _ => break,
            }
        }
        indent
    }

    /// Handle indentation at line start, producing INDENT/DEDENT tokens
    fn handle_indent(&mut self) -> Result<Option<Token>, LexError> {
        // Return pending dedents first
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Ok(Some(Token::new(
                TokenKind::Dedent,
                self.pos,
                self.pos,
                self.line,
                self.column,
            )));
        }

        // Skip indentation handling when inside brackets (Python implicit line continuation)
        if self.bracket_depth > 0 {
            self.at_line_start = false;
            return Ok(Option::None);
        }

        if !self.at_line_start {
            return Ok(Option::None);
        }

        // Skip blank lines and comments
        loop {
            let indent = self.measure_indent();

            // Skip comment-only lines
            if self.peek() == Some('#') {
                self.skip_comment();
            }

            // Skip blank lines
            if self.peek() == Some('\n') {
                self.advance();
                continue;
            }

            // End of file
            if self.peek().is_none() {
                // Emit remaining dedents
                let current = *self.indent_stack.last().unwrap_or(&0);
                if current > 0 {
                    self.indent_stack.pop();
                    return Ok(Some(Token::new(
                        TokenKind::Dedent,
                        self.pos,
                        self.pos,
                        self.line,
                        self.column,
                    )));
                }
                return Ok(Option::None);
            }

            self.at_line_start = false;
            let current_indent = *self.indent_stack.last().unwrap_or(&0);

            if indent > current_indent {
                self.indent_stack.push(indent);
                return Ok(Some(Token::new(
                    TokenKind::Indent,
                    self.pos,
                    self.pos,
                    self.line,
                    self.column,
                )));
            } else if indent < current_indent {
                // Count how many dedents we need
                while let Some(&top) = self.indent_stack.last() {
                    if top <= indent {
                        break;
                    }
                    self.indent_stack.pop();
                    self.pending_dedents += 1;
                }

                // Check for inconsistent indentation
                let new_current = *self.indent_stack.last().unwrap_or(&0);
                if indent != new_current {
                    return Err(LexError {
                        message: format!(
                            "Inconsistent indentation: expected {} spaces, got {}",
                            new_current, indent
                        ),
                        line: self.line,
                        column: self.column,
                        pos: self.pos,
                    });
                }

                if self.pending_dedents > 0 {
                    self.pending_dedents -= 1;
                    return Ok(Some(Token::new(
                        TokenKind::Dedent,
                        self.pos,
                        self.pos,
                        self.line,
                        self.column,
                    )));
                }
            }

            return Ok(Option::None);
        }
    }

    /// Try to match a regex pattern at the current position
    fn try_match(&self, re: &Regex) -> Option<&'a str> {
        re.find(self.remaining()).map(|m| m.as_str())
    }

    /// Lex the next token
    pub fn next_token(&mut self) -> Result<Token, LexError> {
        // Handle pending dedents
        if self.pending_dedents > 0 {
            self.pending_dedents -= 1;
            return Ok(Token::new(
                TokenKind::Dedent,
                self.pos,
                self.pos,
                self.line,
                self.column,
            ));
        }

        // Handle indentation at line start
        if let Some(tok) = self.handle_indent()? {
            return Ok(tok);
        }

        // Skip whitespace
        self.skip_whitespace();

        // Check for EOF
        let Some(ch) = self.peek() else {
            // Emit remaining dedents
            if self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                return Ok(Token::new(
                    TokenKind::Dedent,
                    self.pos,
                    self.pos,
                    self.line,
                    self.column,
                ));
            }
            return Ok(Token::new(
                TokenKind::Eof,
                self.pos,
                self.pos,
                self.line,
                self.column,
            ));
        };

        let start = self.pos;
        let line = self.line;
        let column = self.column;

        // Newline - skip if inside brackets (Python implicit line continuation)
        if ch == '\n' {
            self.advance();
            if self.bracket_depth > 0 {
                // Inside brackets - skip newline, recurse to get next token
                return self.next_token();
            }
            return Ok(Token::new(
                TokenKind::Newline,
                start,
                self.pos,
                line,
                column,
            ));
        }

        // Comment
        if ch == '#' {
            let comment = self.skip_comment().unwrap_or_default();
            return Ok(Token::new(
                TokenKind::Comment(comment),
                start,
                self.pos,
                line,
                column,
            ));
        }

        // Multi-character operators (check longer patterns first)
        if let Some(kind) = self.try_multi_char_op() {
            return Ok(Token::new(kind, start, self.pos, line, column));
        }

        // Single-character operators and delimiters
        if let Some(kind) = self.try_single_char_op() {
            self.advance();
            // Track bracket depth for implicit line continuation
            match &kind {
                TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    self.bracket_depth += 1;
                }
                TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                }
                _ => {}
            }
            return Ok(Token::new(kind, start, self.pos, line, column));
        }

        // String literals (including raw and f-strings)
        if ch == '"'
            || ch == '\''
            || (ch == 'r' && matches!(self.peek_next(), Some('"' | '\'')))
            || (ch == 'f' && matches!(self.peek_next(), Some('"' | '\'')))
            || (ch == 'b' && matches!(self.peek_next(), Some('"' | '\'')))
        {
            return self.lex_string(start, line, column);
        }

        // Numbers
        if ch.is_ascii_digit() {
            return self.lex_number(start, line, column);
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.lex_identifier(start, line, column);
        }

        Err(LexError {
            message: format!("Unexpected character: {:?}", ch),
            line,
            column,
            pos: start,
        })
    }

    /// Try to match multi-character operators
    fn try_multi_char_op(&mut self) -> Option<TokenKind> {
        let remaining = self.remaining();

        // Three-character operators
        if remaining.starts_with("...") {
            self.advance_by(3);
            return Some(TokenKind::Ellipsis);
        }
        if remaining.starts_with("**=") {
            self.advance_by(3);
            return Some(TokenKind::DoubleStarEq);
        }
        if remaining.starts_with("//=") {
            self.advance_by(3);
            return Some(TokenKind::DoubleSlashEq);
        }
        if remaining.starts_with("<<=") {
            self.advance_by(3);
            return Some(TokenKind::LShiftEq);
        }
        if remaining.starts_with(">>=") {
            self.advance_by(3);
            return Some(TokenKind::RShiftEq);
        }

        // Two-character operators
        if remaining.starts_with("**") {
            self.advance_by(2);
            return Some(TokenKind::DoubleStar);
        }
        if remaining.starts_with("//") {
            self.advance_by(2);
            return Some(TokenKind::DoubleSlash);
        }
        if remaining.starts_with("==") {
            self.advance_by(2);
            return Some(TokenKind::EqEq);
        }
        if remaining.starts_with("!=") {
            self.advance_by(2);
            return Some(TokenKind::NotEq);
        }
        if remaining.starts_with("<=") {
            self.advance_by(2);
            return Some(TokenKind::LtEq);
        }
        if remaining.starts_with(">=") {
            self.advance_by(2);
            return Some(TokenKind::GtEq);
        }
        if remaining.starts_with("<<") {
            self.advance_by(2);
            return Some(TokenKind::LShift);
        }
        if remaining.starts_with(">>") {
            self.advance_by(2);
            return Some(TokenKind::RShift);
        }
        if remaining.starts_with("->") {
            self.advance_by(2);
            return Some(TokenKind::Arrow);
        }
        if remaining.starts_with("+=") {
            self.advance_by(2);
            return Some(TokenKind::PlusEq);
        }
        if remaining.starts_with("-=") {
            self.advance_by(2);
            return Some(TokenKind::MinusEq);
        }
        if remaining.starts_with("*=") {
            self.advance_by(2);
            return Some(TokenKind::StarEq);
        }
        if remaining.starts_with("/=") {
            self.advance_by(2);
            return Some(TokenKind::SlashEq);
        }
        if remaining.starts_with("%=") {
            self.advance_by(2);
            return Some(TokenKind::PercentEq);
        }
        if remaining.starts_with("|=") {
            self.advance_by(2);
            return Some(TokenKind::PipeEq);
        }
        if remaining.starts_with("&=") {
            self.advance_by(2);
            return Some(TokenKind::AmpEq);
        }
        if remaining.starts_with("^=") {
            self.advance_by(2);
            return Some(TokenKind::CaretEq);
        }
        if remaining.starts_with("@=") {
            self.advance_by(2);
            return Some(TokenKind::AtEq);
        }
        if remaining.starts_with(":=") {
            self.advance_by(2);
            return Some(TokenKind::ColonEq);
        }

        Option::None
    }

    /// Try to match single-character operators
    fn try_single_char_op(&self) -> Option<TokenKind> {
        match self.peek()? {
            '+' => Some(TokenKind::Plus),
            '-' => Some(TokenKind::Minus),
            '*' => Some(TokenKind::Star),
            '/' => Some(TokenKind::Slash),
            '%' => Some(TokenKind::Percent),
            '@' => Some(TokenKind::At),
            '=' => Some(TokenKind::Eq),
            '<' => Some(TokenKind::Lt),
            '>' => Some(TokenKind::Gt),
            '|' => Some(TokenKind::Pipe),
            '&' => Some(TokenKind::Amp),
            '^' => Some(TokenKind::Caret),
            '~' => Some(TokenKind::Tilde),
            '(' => Some(TokenKind::LParen),
            ')' => Some(TokenKind::RParen),
            '[' => Some(TokenKind::LBracket),
            ']' => Some(TokenKind::RBracket),
            '{' => Some(TokenKind::LBrace),
            '}' => Some(TokenKind::RBrace),
            ':' => Some(TokenKind::Colon),
            ',' => Some(TokenKind::Comma),
            '.' => Some(TokenKind::Dot),
            ';' => Some(TokenKind::Semicolon),
            _ => Option::None,
        }
    }

    /// Lex a string literal
    fn lex_string(&mut self, start: usize, line: usize, column: usize) -> Result<Token, LexError> {
        // Check for prefix (r, f, b, rf, fr, br, rb)
        let mut is_raw = false;
        let mut is_fstring = false;
        let mut is_bytes = false;

        while let Some(ch) = self.peek() {
            match ch {
                'r' | 'R' => {
                    is_raw = true;
                    self.advance();
                }
                'f' | 'F' => {
                    is_fstring = true;
                    self.advance();
                }
                'b' | 'B' => {
                    is_bytes = true;
                    self.advance();
                }
                '"' | '\'' => break,
                _ => break,
            }
        }

        let quote_char = self.peek().ok_or_else(|| LexError {
            message: "Unexpected end of input in string".to_string(),
            line,
            column,
            pos: self.pos,
        })?;

        // Check for triple-quoted
        let is_triple =
            self.remaining().starts_with("\"\"\"") || self.remaining().starts_with("'''");

        let end_pattern = if is_triple {
            if quote_char == '"' { "\"\"\"" } else { "'''" }
        } else {
            if quote_char == '"' { "\"" } else { "'" }
        };

        // Skip opening quote(s)
        let quote_len = if is_triple { 3 } else { 1 };
        self.advance_by(quote_len);

        let mut content = String::new();

        // Scan for end of string
        loop {
            if self.remaining().starts_with(end_pattern) {
                break;
            }

            match self.peek() {
                Option::None => {
                    return Err(LexError {
                        message: "Unterminated string literal".to_string(),
                        line: self.line,
                        column: self.column,
                        pos: self.pos,
                    });
                }
                Some('\n') if !is_triple => {
                    return Err(LexError {
                        message: "Newline in single-quoted string".to_string(),
                        line: self.line,
                        column: self.column,
                        pos: self.pos,
                    });
                }
                Some('\\') if !is_raw => {
                    self.advance(); // Skip backslash
                    if let Some(escaped) = self.peek() {
                        self.advance();
                        let ch = match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            '\\' => '\\',
                            '"' => '"',
                            '\'' => '\'',
                            '0' => '\0',
                            _ => escaped,
                        };
                        content.push(ch);
                    }
                }
                Some(ch) => {
                    self.advance();
                    content.push(ch);
                }
            }
        }

        // Skip closing quote(s)
        self.advance_by(quote_len);

        // Reset at_line_start for multi-line strings - crossing newlines inside
        // strings shouldn't affect indentation tracking. We just finished lexing
        // a token that may have spanned multiple lines, so we're not at line start.
        if is_triple {
            self.at_line_start = false;
        }

        // TODO: Handle f-strings properly (parse interpolations)
        // For now, just return the raw content
        let _ = (is_fstring, is_bytes); // Suppress unused warnings

        Ok(Token::new(
            TokenKind::String(content),
            start,
            self.pos,
            line,
            column,
        ))
    }

    /// Lex a number literal
    fn lex_number(&mut self, start: usize, line: usize, column: usize) -> Result<Token, LexError> {
        // Try hex, octal, binary first
        if let Some(matched) = self.try_match(&self.patterns.int_hex) {
            self.advance_by(matched.len());
            let cleaned = matched[2..].replace('_', "");
            let value = i64::from_str_radix(&cleaned, 16).unwrap_or(0);
            return Ok(Token::new(
                TokenKind::Int(value),
                start,
                self.pos,
                line,
                column,
            ));
        }
        if let Some(matched) = self.try_match(&self.patterns.int_oct) {
            self.advance_by(matched.len());
            let cleaned = matched[2..].replace('_', "");
            let value = i64::from_str_radix(&cleaned, 8).unwrap_or(0);
            return Ok(Token::new(
                TokenKind::Int(value),
                start,
                self.pos,
                line,
                column,
            ));
        }
        if let Some(matched) = self.try_match(&self.patterns.int_bin) {
            self.advance_by(matched.len());
            let cleaned = matched[2..].replace('_', "");
            let value = i64::from_str_radix(&cleaned, 2).unwrap_or(0);
            return Ok(Token::new(
                TokenKind::Int(value),
                start,
                self.pos,
                line,
                column,
            ));
        }

        // Try floats (before integers to match longer patterns)
        if let Some(matched) = self.try_match(&self.patterns.float_exp) {
            self.advance_by(matched.len());
            let cleaned = matched.replace('_', "");
            let value = cleaned.parse().unwrap_or(0.0);
            return Ok(Token::new(
                TokenKind::Float(value),
                start,
                self.pos,
                line,
                column,
            ));
        }
        if let Some(matched) = self.try_match(&self.patterns.float_simple) {
            self.advance_by(matched.len());
            let cleaned = matched.replace('_', "");
            let value = cleaned.parse().unwrap_or(0.0);
            return Ok(Token::new(
                TokenKind::Float(value),
                start,
                self.pos,
                line,
                column,
            ));
        }

        // Regular integer
        if let Some(matched) = self.try_match(&self.patterns.int_dec) {
            self.advance_by(matched.len());
            let cleaned = matched.replace('_', "");
            let value = cleaned.parse().unwrap_or(0);
            return Ok(Token::new(
                TokenKind::Int(value),
                start,
                self.pos,
                line,
                column,
            ));
        }

        Err(LexError {
            message: "Invalid number literal".to_string(),
            line,
            column,
            pos: start,
        })
    }

    /// Lex an identifier or keyword
    fn lex_identifier(
        &mut self,
        start: usize,
        line: usize,
        column: usize,
    ) -> Result<Token, LexError> {
        if let Some(matched) = self.try_match(&self.patterns.ident) {
            self.advance_by(matched.len());
            let kind = if let Some(kw) = Keyword::from_str(matched) {
                TokenKind::Keyword(kw)
            } else {
                TokenKind::Ident(matched.to_string())
            };
            return Ok(Token::new(kind, start, self.pos, line, column));
        }

        Err(LexError {
            message: "Invalid identifier".to_string(),
            line,
            column,
            pos: start,
        })
    }

    /// Tokenize entire source, returning all tokens
    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}

/// Convenience function to tokenize source code
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source).map_err(|e| LexError {
        message: e,
        line: 1,
        column: 1,
        pos: 0,
    })?;
    lexer.tokenize_all()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn tok_kinds(source: &str) -> Vec<TokenKind> {
        tokenize(source)
            .unwrap()
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Comment(_)))
            .map(|t| t.kind)
            .collect()
    }

    #[test]
    fn test_simple_expression() {
        // No trailing newline in input = no Newline token
        let tokens = tok_kinds("x + 1");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Plus,
                TokenKind::Int(1),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords() {
        let tokens = tok_kinds("def if else");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Keyword(Keyword::Def),
                TokenKind::Keyword(Keyword::If),
                TokenKind::Keyword(Keyword::Else),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let tokens = tok_kinds(r#""hello""#);
        assert_eq!(
            tokens,
            vec![TokenKind::String("hello".into()), TokenKind::Eof,]
        );
    }

    #[test]
    fn test_string_with_escapes() {
        let tokens = tok_kinds(r#""hello\nworld""#);
        assert_eq!(
            tokens,
            vec![TokenKind::String("hello\nworld".into()), TokenKind::Eof,]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens = tok_kinds("42 3.14 0xff 0b101");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Int(42),
                TokenKind::Float(3.14),
                TokenKind::Int(255),
                TokenKind::Int(5),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = tok_kinds("+ - * ** // -> ==");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::DoubleStar,
                TokenKind::DoubleSlash,
                TokenKind::Arrow,
                TokenKind::EqEq,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_indentation() {
        let source = "if x:\n    y = 1\n    z = 2\n";
        let tokens = tok_kinds(source);
        assert!(tokens.contains(&TokenKind::Indent));
        assert!(tokens.contains(&TokenKind::Dedent));
    }

    #[test]
    fn test_function_def() {
        let source = "def foo(x):\n    return x + 1\n";
        let tokens = tok_kinds(source);
        assert_eq!(tokens[0], TokenKind::Keyword(Keyword::Def));
        assert_eq!(tokens[1], TokenKind::Ident("foo".into()));
        assert_eq!(tokens[2], TokenKind::LParen);
    }

    #[test]
    fn test_delimiters() {
        let tokens = tok_kinds("()[]{}:,.");
        assert_eq!(
            tokens,
            vec![
                TokenKind::LParen,
                TokenKind::RParen,
                TokenKind::LBracket,
                TokenKind::RBracket,
                TokenKind::LBrace,
                TokenKind::RBrace,
                TokenKind::Colon,
                TokenKind::Comma,
                TokenKind::Dot,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_ellipsis() {
        let tokens = tok_kinds("...");
        assert_eq!(tokens, vec![TokenKind::Ellipsis, TokenKind::Eof]);
    }

    #[test]
    fn test_with_trailing_newline() {
        // With trailing newline, we DO get a Newline token
        let tokens = tok_kinds("x + 1\n");
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("x".into()),
                TokenKind::Plus,
                TokenKind::Int(1),
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multiline_docstring() {
        // Test that multi-line docstrings don't break indentation
        let source =
            "def test():\n    \"\"\"Multi\n    line\n    doc\"\"\"\n    if x:\n        pass\n";
        let tokens = tok_kinds(source);
        // Should have: def, test, (, ), :, Newline, Indent, String, Newline, if, x, :, Newline, Indent, pass, Newline, Dedent, Dedent, Eof
        // Key: after the docstring, we should get a Newline then `if` keyword
        assert!(tokens.contains(&TokenKind::Keyword(Keyword::Def)));
        assert!(tokens.contains(&TokenKind::Keyword(Keyword::If)));
        // Verify we get Indent tokens (means indentation tracking worked)
        let indent_count = tokens
            .iter()
            .filter(|t| matches!(t, TokenKind::Indent))
            .count();
        assert!(
            indent_count >= 2,
            "Should have at least 2 Indent tokens, got {}",
            indent_count
        );
    }

    #[test]
    fn test_multiline_function_call() {
        // Test that newlines inside parens are skipped
        let source = "foo(\n    1,\n    2\n)\n";
        let tokens = tok_kinds(source);
        // Should NOT have any Newline tokens inside the parens
        // Tokens: foo, (, 1, ,, 2, ), Newline, Eof
        assert_eq!(
            tokens,
            vec![
                TokenKind::Ident("foo".into()),
                TokenKind::LParen,
                TokenKind::Int(1),
                TokenKind::Comma,
                TokenKind::Int(2),
                TokenKind::RParen,
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multiline_brackets() {
        // Test that newlines inside brackets are skipped
        let source = "x = [\n    1,\n    2\n]\n";
        let tokens = tok_kinds(source);
        // Should NOT have Newline or Indent tokens inside the brackets
        assert!(
            !tokens[1..tokens.len() - 2].iter().any(|t| matches!(
                t,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
            )),
            "Should not have Newline/Indent/Dedent inside brackets: {:?}",
            tokens
        );
    }
}
