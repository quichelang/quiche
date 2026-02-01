//! Compiler diagnostics and telemetry
//!
//! Provides Rust-style error and warning display for the Quiche compiler.
//!
//! # Design (inspired by rustc)
//!
//! - `Diagnostic` - A single error/warning with source location
//! - `DiagnosticLevel` - Error, Warning, Note, Help
//! - `CompileContext` - Tracks current file, source, module path
//! - `Emitter` - Collects and displays diagnostics

use std::fmt;

use crate::i18n::t;

/// Severity level of a diagnostic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

impl fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLevel::Error => write!(f, "{}", t!("diagnostic.level.error")),
            DiagnosticLevel::Warning => write!(f, "{}", t!("diagnostic.level.warning")),
            DiagnosticLevel::Note => write!(f, "{}", t!("diagnostic.level.note")),
            DiagnosticLevel::Help => write!(f, "{}", t!("diagnostic.level.help")),
        }
    }
}

/// A source location span
#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub length: usize, // For underlining multiple characters
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Self {
            line,
            column,
            length: 1,
        }
    }

    pub fn with_length(line: usize, column: usize, length: usize) -> Self {
        Self {
            line,
            column,
            length,
        }
    }
}

/// A single diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<Span>,
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message: message.into(),
            span: None,
            help: None,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message: message.into(),
            span: None,
            help: None,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

/// Compilation context - tracks current file and source
#[derive(Debug, Clone)]
pub struct CompileContext {
    pub filename: String,
    pub source: String,
    pub module_path: Option<String>,
}

impl CompileContext {
    pub fn new(filename: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            source: source.into(),
            module_path: None,
        }
    }

    pub fn with_module(mut self, module_path: impl Into<String>) -> Self {
        self.module_path = Some(module_path.into());
        self
    }

    /// Get source line by line number (1-indexed)
    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.source.lines().nth(line.saturating_sub(1))
    }

    /// Convert byte offset to Span
    pub fn byte_to_span(&self, byte_offset: usize) -> Span {
        let mut line = 1;
        let mut col = 1;
        let mut pos = 0;

        for ch in self.source.chars() {
            if pos >= byte_offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
            pos += ch.len_utf8();
        }

        Span::new(line, col)
    }
}

/// Diagnostic emitter - formats and outputs diagnostics
pub struct Emitter {
    diagnostics: Vec<(Diagnostic, Option<CompileContext>)>,
    has_errors: bool,
}

impl Emitter {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            has_errors: false,
        }
    }

    /// Emit a diagnostic with context
    pub fn emit(&mut self, diag: Diagnostic, ctx: Option<&CompileContext>) {
        if diag.level == DiagnosticLevel::Error {
            self.has_errors = true;
        }
        self.diagnostics.push((diag, ctx.cloned()));
    }

    /// Check if any errors were emitted
    pub fn has_errors(&self) -> bool {
        self.has_errors
    }

    /// Format and print all diagnostics
    pub fn flush(&self) {
        for (diag, ctx) in &self.diagnostics {
            eprintln!("{}", format_diagnostic(diag, ctx.as_ref()));
        }
    }

    /// Print header for compilation failure
    pub fn print_failed_header(filename: &str) {
        eprint!("{}", t!("diagnostic.compile_failed", file = filename));
    }
}

impl Default for Emitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Format a single diagnostic for display
pub fn format_diagnostic(diag: &Diagnostic, ctx: Option<&CompileContext>) -> String {
    let mut output = String::new();

    // Level and message
    output.push_str(&format!("{}: {}\n", diag.level, diag.message));

    // Source location if available
    if let (Some(span), Some(ctx)) = (&diag.span, ctx) {
        let source_line = ctx.get_line(span.line).unwrap_or("");

        output.push_str(&format!(
            "  --> {}:{}:{}\n   |\n{:3} | {}\n   | {}{}",
            ctx.filename,
            span.line,
            span.column,
            span.line,
            source_line,
            " ".repeat(span.column.saturating_sub(1)),
            "^".repeat(span.length.max(1))
        ));

        if diag.help.is_some() || true {
            output.push('\n');
        }
    }

    // Help message if available
    if let Some(help) = &diag.help {
        output.push_str(&t!("diagnostic.help_prefix", message = help));
    }

    output
}

/// Convenience function: report a compile error and exit
pub fn report_error(ctx: &CompileContext, message: &str, byte_offset: Option<usize>) -> ! {
    Emitter::print_failed_header(&ctx.filename);

    let mut diag = Diagnostic::error(message);
    if let Some(offset) = byte_offset {
        diag = diag.with_span(ctx.byte_to_span(offset));
    }

    eprintln!("{}", format_diagnostic(&diag, Some(ctx)));
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_span() {
        let ctx = CompileContext::new("test.qrs", "line1\nline2\nline3");

        assert_eq!(ctx.byte_to_span(0).line, 1);
        assert_eq!(ctx.byte_to_span(6).line, 2);
        assert_eq!(ctx.byte_to_span(12).line, 3);
    }

    #[test]
    fn test_format_diagnostic() {
        let ctx = CompileContext::new("test.qrs", "def foo():\n    pass");
        let diag = Diagnostic::error("unexpected token")
            .with_span(Span::new(1, 5))
            .with_help("did you mean 'def'?");

        let output = format_diagnostic(&diag, Some(&ctx));
        assert!(output.contains("error: unexpected token"));
        assert!(output.contains("test.qrs:1:5"));
        assert!(output.contains("help:"));
    }
}
