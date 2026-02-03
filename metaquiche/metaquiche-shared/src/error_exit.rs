//! Compiler error handling with graceful exit
//!
//! Provides `UnwrapOrExit` trait for Result/Option with builder pattern
//! for rich error context integration with telemetry.

use crate::telemetry::{CompileContext, Diagnostic, Emitter, Span, format_diagnostic};

/// Context for error reporting with optional source location
pub struct ErrorContext<'a> {
    ctx: Option<&'a CompileContext>,
    span: Option<Span>,
    message: Option<String>,
}

impl<'a> ErrorContext<'a> {
    pub fn new() -> Self {
        Self {
            ctx: None,
            span: None,
            message: None,
        }
    }

    pub fn with_context(mut self, ctx: &'a CompileContext) -> Self {
        self.ctx = Some(ctx);
        self
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_error(mut self, msg: impl Into<String>) -> Self {
        self.message = Some(msg.into());
        self
    }

    /// Exit with the accumulated error context
    pub fn exit(self, error: Option<&dyn std::fmt::Display>) -> ! {
        let msg = self.message.unwrap_or_else(|| "Compilation failed".into());
        let full_msg = match error {
            Some(e) => format!("{}: {}", msg, e),
            None => msg,
        };

        if let Some(ctx) = self.ctx {
            Emitter::print_failed_header(&ctx.filename);
            let mut diag = Diagnostic::error(&full_msg);
            if let Some(s) = self.span {
                diag = diag.with_span(s);
            }
            eprintln!("{}", format_diagnostic(&diag, Some(ctx)));
        } else {
            eprintln!("error: {}", full_msg);
        }
        std::process::exit(1);
    }
}

impl Default for ErrorContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for Result/Option with graceful exit
pub trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self) -> ExitBuilder<T>;
}

/// Builder for configuring error exit behavior
pub struct ExitBuilder<T> {
    value: Result<T, String>,
}

impl<T> ExitBuilder<T> {
    /// Exit with custom error message (no source context)
    pub fn with_error(self, msg: &str) -> T {
        self.finish(ErrorContext::new().with_error(msg))
    }

    /// Add compilation context for rich error reporting
    pub fn with_context<'a>(self, ctx: &'a CompileContext) -> ContextBuilder<'a, T> {
        ContextBuilder {
            value: self.value,
            error_ctx: ErrorContext::new().with_context(ctx),
        }
    }

    fn finish(self, err_ctx: ErrorContext<'_>) -> T {
        match self.value {
            Ok(v) => v,
            Err(e) => err_ctx.exit(Some(&e as &dyn std::fmt::Display)),
        }
    }
}

/// Builder with compilation context attached
pub struct ContextBuilder<'a, T> {
    value: Result<T, String>,
    error_ctx: ErrorContext<'a>,
}

impl<'a, T> ContextBuilder<'a, T> {
    /// Add source span for error location
    pub fn with_span(mut self, span: Span) -> Self {
        self.error_ctx = self.error_ctx.with_span(span);
        self
    }

    /// Complete with error message and unwrap or exit
    pub fn with_error(mut self, msg: &str) -> T {
        self.error_ctx = self.error_ctx.with_error(msg);
        match self.value {
            Ok(v) => v,
            Err(e) => self.error_ctx.exit(Some(&e as &dyn std::fmt::Display)),
        }
    }
}

impl<T, E: std::fmt::Display> UnwrapOrExit<T> for Result<T, E> {
    fn unwrap_or_exit(self) -> ExitBuilder<T> {
        ExitBuilder {
            value: self.map_err(|e| e.to_string()),
        }
    }
}

impl<T> UnwrapOrExit<T> for Option<T> {
    fn unwrap_or_exit(self) -> ExitBuilder<T> {
        ExitBuilder {
            value: self.ok_or_else(|| "None".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_result_ok_unwraps() {
        let result: Result<i32, &str> = Ok(42);
        let value = result.unwrap_or_exit().with_error("should not fail");
        assert_eq!(value, 42);
    }

    #[test]
    fn test_option_some_unwraps() {
        let opt: Option<i32> = Some(42);
        let value = opt.unwrap_or_exit().with_error("should not fail");
        assert_eq!(value, 42);
    }

    #[test]
    fn test_error_context_builder() {
        let ctx = ErrorContext::new().with_error("test message");
        assert_eq!(ctx.message, Some("test message".to_string()));
    }
}
