//! F-string parser for Quiche
//!
//! Parses f-string content (between quotes) into AST nodes.
//! Handles replacement fields {expr}, escapes, and format specifiers.

use crate::ast::FStringPart;
use crate::parser::{ParseError, Parser};

/// Parse f-string content into parts
pub fn parse_fstring_content(
    content: &str,
    _parser: &Parser, // Keep reference for error context
) -> Result<Vec<FStringPart>, ParseError> {
    let mut parts = Vec::new();
    let mut chars = content.chars().peekable();
    let mut literal = String::new();

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                // Check for escaped brace: {{
                if chars.peek() == Some(&'{') {
                    chars.next();
                    literal.push('{');
                } else {
                    // Flush literal if any
                    if !literal.is_empty() {
                        parts.push(FStringPart::Literal(std::mem::take(&mut literal)));
                    }
                    // Parse replacement field
                    let replacement = parse_replacement_field(&mut chars)?;
                    parts.push(replacement);
                }
            }
            '}' => {
                // Check for escaped brace: }}
                if chars.peek() == Some(&'}') {
                    chars.next();
                    literal.push('}');
                } else {
                    // Unmatched } is an error
                    return Err(ParseError {
                        message: "Single '}' is not allowed in f-string".to_string(),
                        line: 1,
                        column: 1,
                    });
                }
            }
            _ => {
                literal.push(ch);
            }
        }
    }

    // Flush remaining literal
    if !literal.is_empty() {
        parts.push(FStringPart::Literal(literal));
    }

    Ok(parts)
}

/// Parse a replacement field: {expr!conversion:format}
fn parse_replacement_field(
    chars: &mut std::iter::Peekable<std::str::Chars>,
) -> Result<FStringPart, ParseError> {
    let mut expr_str = String::new();
    let mut brace_depth = 1;
    let mut in_string = false;
    let mut string_char = '"';
    let mut debug = false;
    let mut conversion: Option<char> = None;
    let mut format_spec: Option<String> = None;

    // Scan until we find the matching }
    while let Some(&ch) = chars.peek() {
        if in_string {
            expr_str.push(chars.next().unwrap());
            if ch == string_char && !expr_str.ends_with('\\') {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
                expr_str.push(chars.next().unwrap());
            }
            '{' => {
                brace_depth += 1;
                expr_str.push(chars.next().unwrap());
            }
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    chars.next(); // consume the closing }
                    break;
                }
                expr_str.push(chars.next().unwrap());
            }
            '=' if brace_depth == 1 => {
                // Debug specifier - check if this is at the end
                chars.next();
                if chars.peek() == Some(&'}')
                    || chars.peek() == Some(&'!')
                    || chars.peek() == Some(&':')
                {
                    debug = true;
                } else {
                    // It's part of the expression (e.g., ==)
                    expr_str.push('=');
                }
            }
            '!' if brace_depth == 1 => {
                chars.next(); // skip !
                if let Some(&conv) = chars.peek() {
                    if conv == 's' || conv == 'r' || conv == 'a' {
                        conversion = Some(chars.next().unwrap());
                    }
                }
            }
            ':' if brace_depth == 1 => {
                chars.next(); // skip :
                // Collect format spec until }
                let mut spec = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '}' {
                        break;
                    }
                    spec.push(chars.next().unwrap());
                }
                format_spec = Some(spec);
            }
            _ => {
                expr_str.push(chars.next().unwrap());
            }
        }
    }

    // Parse the expression string using a new sub-parser
    let mut sub_parser = Parser::new(&expr_str)?;
    let expr = sub_parser.parse_expression()?;

    Ok(FStringPart::Replacement {
        value: Box::new(expr),
        debug,
        conversion,
        format_spec,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_simple_literal() {
        // Just test parsing - full integration tested elsewhere
    }
}
