//! Python `re`-like regex API for Quiche
//!
//! Provides a familiar API wrapping Rust's `regex` crate:
//! - `re_compile(pattern)` - Compile pattern into reusable Regex
//! - `re_search(pattern, text)` - Find first match anywhere
//! - `re_match(pattern, text)` - Match at beginning only
//! - `re_findall(pattern, text)` - Return all matches
//! - `re_sub(pattern, repl, text)` - Replace all occurrences
//! - `re_split(pattern, text)` - Split by pattern

use regex::Regex;

/// A compiled regular expression (wrapper for regex::Regex)
#[derive(Clone)]
pub struct QuicheRegex(Regex);

impl std::fmt::Debug for QuicheRegex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuicheRegex({})", self.0.as_str())
    }
}

/// Match result containing start, end offsets and matched text
#[derive(Debug, Clone)]
pub struct QuicheMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Compile a regex pattern into a reusable QuicheRegex object
///
/// # Example (MetaQuiche)
/// ```python
/// pattern = re.compile(r"\d+")
/// ```
pub fn re_compile(pattern: &str) -> Result<QuicheRegex, String> {
    Regex::new(pattern)
        .map(QuicheRegex)
        .map_err(|e| e.to_string())
}

/// Test if pattern matches anywhere in text
///
/// # Example (MetaQuiche)
/// ```python
/// if re.is_match(r"\d+", text):
///     print("Contains digits")
/// ```
pub fn re_is_match(pattern: &str, text: &str) -> Result<bool, String> {
    let re = re_compile(pattern)?;
    Ok(re.0.is_match(text))
}

/// Test if compiled regex matches anywhere in text
pub fn re_is_match_compiled(re: &QuicheRegex, text: &str) -> bool {
    re.0.is_match(text)
}

/// Find first match of pattern anywhere in text
///
/// # Example (MetaQuiche)  
/// ```python
/// m = re.search(r"\d+", "abc123def")
/// if m is not None:
///     print(m.text)  # "123"
/// ```
pub fn re_search(pattern: &str, text: &str) -> Result<Option<QuicheMatch>, String> {
    let re = re_compile(pattern)?;
    Ok(re_search_compiled(&re, text))
}

/// Find first match using compiled regex
pub fn re_search_compiled(re: &QuicheRegex, text: &str) -> Option<QuicheMatch> {
    re.0.find(text).map(|m| QuicheMatch {
        start: m.start(),
        end: m.end(),
        text: m.as_str().to_string(),
    })
}

/// Match pattern at the beginning of text only
///
/// # Example (MetaQuiche)
/// ```python
/// m = re.match(r"\d+", "123abc")  # Some(Match)
/// m = re.match(r"\d+", "abc123")  # None
/// ```
pub fn re_match(pattern: &str, text: &str) -> Result<Option<QuicheMatch>, String> {
    // Anchor pattern to start
    let anchored = format!("^(?:{})", pattern);
    re_search(&anchored, text)
}

/// Match using compiled regex at beginning of text
pub fn re_match_compiled(re: &QuicheRegex, text: &str) -> Option<QuicheMatch> {
    // For compiled regex, we check if match starts at 0
    re.0.find(text).and_then(|m| {
        if m.start() == 0 {
            Some(QuicheMatch {
                start: m.start(),
                end: m.end(),
                text: m.as_str().to_string(),
            })
        } else {
            None
        }
    })
}

/// Find all non-overlapping matches of pattern in text
///
/// # Example (MetaQuiche)
/// ```python
/// matches = re.findall(r"\d+", "a1b22c333")
/// # ["1", "22", "333"]
/// ```
pub fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let re = re_compile(pattern)?;
    Ok(re_findall_compiled(&re, text))
}

/// Find all matches using compiled regex
pub fn re_findall_compiled(re: &QuicheRegex, text: &str) -> Vec<String> {
    re.0.find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Replace all occurrences of pattern with replacement string
///
/// # Example (MetaQuiche)
/// ```python
/// result = re.sub(r"\d+", "X", "a1b22c333")
/// # "aXbXcX"
/// ```
pub fn re_sub(pattern: &str, repl: &str, text: &str) -> Result<String, String> {
    let re = re_compile(pattern)?;
    Ok(re_sub_compiled(&re, repl, text))
}

/// Replace using compiled regex
pub fn re_sub_compiled(re: &QuicheRegex, repl: &str, text: &str) -> String {
    re.0.replace_all(text, repl).into_owned()
}

/// Split text by pattern matches
///
/// # Example (MetaQuiche)
/// ```python
/// parts = re.split(r"\s+", "hello   world")
/// # ["hello", "world"]
/// ```
pub fn re_split(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let re = re_compile(pattern)?;
    Ok(re_split_compiled(&re, text))
}

/// Split using compiled regex
pub fn re_split_compiled(re: &QuicheRegex, text: &str) -> Vec<String> {
    re.0.split(text).map(|s| s.to_string()).collect()
}

/// Get the pattern string from a compiled regex
pub fn re_pattern(re: &QuicheRegex) -> String {
    re.0.as_str().to_string()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_valid() {
        let re = re_compile(r"\d+");
        assert!(re.is_ok());
    }

    #[test]
    fn test_compile_invalid() {
        let re = re_compile(r"[invalid");
        assert!(re.is_err());
    }

    #[test]
    fn test_is_match() {
        assert!(re_is_match(r"\d+", "abc123").unwrap());
        assert!(!re_is_match(r"\d+", "abc").unwrap());
    }

    #[test]
    fn test_search() {
        let m = re_search(r"\d+", "abc123def").unwrap();
        assert!(m.is_some());
        let m = m.unwrap();
        assert_eq!(m.text, "123");
        assert_eq!(m.start, 3);
        assert_eq!(m.end, 6);
    }

    #[test]
    fn test_search_no_match() {
        let m = re_search(r"\d+", "abcdef").unwrap();
        assert!(m.is_none());
    }

    #[test]
    fn test_match_at_start() {
        let m = re_match(r"\d+", "123abc").unwrap();
        assert!(m.is_some());
        assert_eq!(m.unwrap().text, "123");
    }

    #[test]
    fn test_match_not_at_start() {
        let m = re_match(r"\d+", "abc123").unwrap();
        assert!(m.is_none());
    }

    #[test]
    fn test_findall() {
        let matches = re_findall(r"\d+", "a1b22c333").unwrap();
        assert_eq!(matches, vec!["1", "22", "333"]);
    }

    #[test]
    fn test_findall_no_matches() {
        let matches = re_findall(r"\d+", "abc").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_sub() {
        let result = re_sub(r"\d+", "X", "a1b22c333").unwrap();
        assert_eq!(result, "aXbXcX");
    }

    #[test]
    fn test_sub_no_matches() {
        let result = re_sub(r"\d+", "X", "abc").unwrap();
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_split() {
        let parts = re_split(r"\s+", "hello   world  test").unwrap();
        assert_eq!(parts, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_split_no_matches() {
        let parts = re_split(r"\s+", "hello").unwrap();
        assert_eq!(parts, vec!["hello"]);
    }

    #[test]
    fn test_compiled_regex() {
        let re = re_compile(r"\d+").unwrap();
        assert!(re_is_match_compiled(&re, "abc123"));
        assert_eq!(re_search_compiled(&re, "abc123").unwrap().text, "123");
        assert_eq!(re_findall_compiled(&re, "a1b2"), vec!["1", "2"]);
        assert_eq!(re_sub_compiled(&re, "X", "a1b2"), "aXbX");
        assert_eq!(re_split_compiled(&re, "a1b2c"), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_pattern_extraction() {
        let re = re_compile(r"\d+").unwrap();
        assert_eq!(re_pattern(&re), r"\d+");
    }
}
