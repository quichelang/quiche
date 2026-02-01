/// Error formatting utilities for Quiche compiler errors
///
/// Transforms opaque byte-range errors into human-readable messages with:
/// - Line and column numbers
/// - Source code context
/// - Visual caret pointing to the error location

/// Convert byte offset to (line, column), both 1-indexed
pub fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;
    let mut pos = 0;

    for ch in source.chars() {
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

    (line, col)
}

/// Format a compiler error with source context
///
/// # Arguments
/// * `message` - The error message (may contain "byte range X..Y")
/// * `source` - The original source code
///
/// # Returns
/// A formatted error string with line/column and source context
pub fn format_error_with_context(message: &str, source: &str) -> String {
    // Try to extract byte range from error message
    if let Some(byte_offset) = extract_byte_offset(message) {
        let (line, col) = byte_to_line_col(source, byte_offset);
        let source_line = source.lines().nth(line.saturating_sub(1)).unwrap_or("");

        // Clean up the message by removing the byte range info
        let clean_msg = message.split(" at byte range").next().unwrap_or(message);

        format!(
            "{}\n  --> line {}:{}\n   |\n{:3} | {}\n   | {}^",
            clean_msg,
            line,
            col,
            line,
            source_line,
            " ".repeat(col.saturating_sub(1))
        )
    } else {
        message.to_string()
    }
}

/// Extract byte offset from error message containing "byte range X..Y"
fn extract_byte_offset(message: &str) -> Option<usize> {
    let range_start = message.find("byte range ")?;
    let rest = &message[range_start + 11..];
    let (start_str, _) = rest.split_once("..")?;
    start_str.parse().ok()
}

/// Format a ruff parse error with source context
pub fn format_ruff_error(error: &ruff_python_parser::ParseError, source: &str) -> String {
    format_error_with_context(&error.to_string(), source)
}

/// Format a complete compiler error with filename header (Rust-style)
///
/// # Arguments
/// * `filename` - The source file being compiled
/// * `message` - The error message
/// * `source` - The source code
///
/// # Returns
/// A fully formatted compile error ready for display
pub fn format_compile_error(filename: &str, message: &str, source: &str) -> String {
    // Try to extract byte range and format with context
    if let Some(byte_offset) = extract_byte_offset(message) {
        let (line, col) = byte_to_line_col(source, byte_offset);
        let source_line = source.lines().nth(line.saturating_sub(1)).unwrap_or("");

        // Clean up the message by removing the byte range info
        let clean_msg = message.split(" at byte range").next().unwrap_or(message);

        format!(
            "error: Failed to compile `{}`\n\nerror: {}\n  --> {}:{}:{}\n   |\n{:3} | {}\n   | {}^\n",
            filename,
            clean_msg,
            filename,
            line,
            col,
            line,
            source_line,
            " ".repeat(col.saturating_sub(1))
        )
    } else {
        format!(
            "error: Failed to compile `{}`\n\nerror: {}\n",
            filename, message
        )
    }
}

/// Print a compile error to stderr and exit
pub fn report_compile_error(filename: &str, message: &str, source: &str) -> ! {
    eprint!("{}", format_compile_error(filename, message, source));
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_to_line_col_first_line() {
        let source = "hello world";
        assert_eq!(byte_to_line_col(source, 0), (1, 1));
        assert_eq!(byte_to_line_col(source, 6), (1, 7));
    }

    #[test]
    fn test_byte_to_line_col_multiline() {
        let source = "line1\nline2\nline3";
        assert_eq!(byte_to_line_col(source, 0), (1, 1));
        assert_eq!(byte_to_line_col(source, 6), (2, 1));
        assert_eq!(byte_to_line_col(source, 12), (3, 1));
    }

    #[test]
    fn test_format_error_with_context() {
        let source = "def foo():\n    x = 1\n    y";
        // Byte 21 is the start of line 3
        let msg = "Expected something at byte range 21..22";
        let result = format_error_with_context(msg, source);
        assert!(result.contains("line 3"));
        assert!(result.contains("y"));
    }

    #[test]
    fn test_format_error_without_byte_range() {
        let msg = "Generic error message";
        let result = format_error_with_context(msg, "any source");
        assert_eq!(result, msg);
    }
}
