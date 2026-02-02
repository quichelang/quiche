//! Internationalization (i18n) module for the Quiche compiler ecosystem
//!
//! This module provides centralized translation support using the template system.
//! All user-facing strings are defined in `templates/messages.toml` and
//! accessed through the translation functions.

use crate::template;

/// Initialize i18n (no-op, kept for API compatibility)
pub fn init() {
    // Templates are lazily loaded, no initialization needed
}

/// Get the current locale (returns "en-US" for now)
pub fn current_locale() -> String {
    "en-US".to_string()
}

/// Set the locale (no-op, kept for API compatibility)
pub fn set_locale(_locale: &str) {
    // Single-locale for now, could be extended later
}

/// Get the current locale (alias for current_locale)
pub fn locale() -> String {
    current_locale()
}

// ============================================================================
// Translation wrapper functions for cross-crate usage
// ============================================================================

/// Translate a simple key (no interpolation)
pub fn tr(key: &str) -> String {
    template::message(key)
}

/// Translate with a single named argument
pub fn tr1(key: &str, name: &str, value: &str) -> String {
    template::message_fmt(key, &[(name, value)])
}

/// Translate with two named arguments
pub fn tr2(key: &str, name1: &str, val1: &str, name2: &str, val2: &str) -> String {
    template::message_fmt(key, &[(name1, val1), (name2, val2)])
}

// ============================================================================
// Convenience macro for use within metaquiche-shared
// ============================================================================

/// Translation macro for internal use within metaquiche-shared
/// Provides t!("key") and t!("key", name = value) syntax
macro_rules! t {
    ($key:expr) => {
        crate::template::message($key)
    };
    ($key:expr, $name:ident = $value:expr) => {
        crate::template::message_fmt($key, &[(stringify!($name), &$value.to_string())])
    };
    ($key:expr, $name1:ident = $value1:expr, $name2:ident = $value2:expr) => {
        crate::template::message_fmt(
            $key,
            &[
                (stringify!($name1), &$value1.to_string()),
                (stringify!($name2), &$value2.to_string()),
            ],
        )
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
