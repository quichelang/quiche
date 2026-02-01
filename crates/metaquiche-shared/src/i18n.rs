//! Internationalization (i18n) module for the Quiche compiler ecosystem
//!
//! This module provides centralized translation support using rust-i18n.
//! All user-facing strings should be defined in the locale files and
//! accessed through the `tr!` macro or the translation functions.
//!
//! Note: The `rust_i18n::i18n!` macro is invoked in lib.rs (crate root)
//! as required by the rust-i18n crate.

// Re-export utilities (not t! macro - it only works within this crate)
pub use rust_i18n::{locale, set_locale};

/// Initialize i18n with the default locale (en-US)
pub fn init() {
    set_locale("en-US");
}

/// Get the current locale
pub fn current_locale() -> String {
    locale().to_string()
}

// ============================================================================
// Translation wrapper functions for cross-crate usage
// These functions call the t!() macro internally, avoiding crate-root issues
// ============================================================================

/// Translate a simple key (no interpolation)
pub fn tr(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}

/// Translate with a single named argument
pub fn tr1(key: &str, name: &str, value: &str) -> String {
    // rust-i18n requires compile-time known keys for the named syntax,
    // so we do string replacement manually
    let template = rust_i18n::t!(key);
    template.replace(&format!("%{{{}}}", name), value)
}

/// Translate with two named arguments
pub fn tr2(key: &str, name1: &str, val1: &str, name2: &str, val2: &str) -> String {
    let template = rust_i18n::t!(key);
    template
        .replace(&format!("%{{{}}}", name1), val1)
        .replace(&format!("%{{{}}}", name2), val2)
}

// ============================================================================
// Convenience macro for use within metaquiche-shared
// ============================================================================

/// Translation macro for internal use within metaquiche-shared
/// Re-exports rust_i18n::t! for internal module use
macro_rules! t {
    ($key:expr) => {
        rust_i18n::t!($key)
    };
    ($key:expr, $($arg:tt)*) => {
        rust_i18n::t!($key, $($arg)*)
    };
}
pub(crate) use t;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_locale() {
        init();
        assert_eq!(current_locale(), "en-US");
    }

    #[test]
    fn test_basic_translation() {
        init();
        let msg = tr("cli.error.no_command");
        assert!(!msg.is_empty());
        assert!(msg.contains("Error"));
    }

    #[test]
    fn test_translation_with_interpolation() {
        init();
        let msg = tr1("cli.error.file_not_found", "file", "test.qrs");
        assert!(msg.contains("test.qrs"));
    }

    #[test]
    fn test_tr2() {
        init();
        let msg = tr2(
            "cli.error.read_file_failed",
            "file",
            "test.qrs",
            "error",
            "not found",
        );
        assert!(msg.contains("test.qrs"));
        assert!(msg.contains("not found"));
    }
}
