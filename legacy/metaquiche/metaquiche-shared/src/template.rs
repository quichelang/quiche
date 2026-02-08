//! Unified template system for Quiche compilers
//!
//! This module provides shared templates used by both metaquiche-host and
//! metaquiche-native to ensure identical code output (stage parity).
//!
//! Template files are stored in `templates/` directory with the format:
//! - `[namespace.strings]` sections contain simple `key = "value"` pairs
//! - `[namespace.block_name]` with `content = '''...'''` for multi-line content

use std::collections::HashMap;

/// Raw template content embedded at compile time
const CODEGEN_TOML: &str = include_str!("../templates/codegen.toml");
const PROJECT_TOML: &str = include_str!("../templates/project.toml");
const RUNTIME_TOML: &str = include_str!("../templates/runtime.toml");
const MESSAGES_TOML: &str = include_str!("../templates/messages.toml");

/// A parsed template with its content and metadata
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub content: String,
}

/// Template registry holding all parsed templates
pub struct Templates {
    templates: HashMap<String, Template>,
}

impl Templates {
    /// Parse and load all templates from the embedded TOML files
    pub fn load() -> Self {
        let mut templates = HashMap::new();

        // Load each template file
        Self::parse_toml(&mut templates, CODEGEN_TOML);
        Self::parse_toml(&mut templates, PROJECT_TOML);
        Self::parse_toml(&mut templates, RUNTIME_TOML);
        Self::parse_toml(&mut templates, MESSAGES_TOML);

        Templates { templates }
    }

    /// Check if a line is a key=value assignment (not a section header)
    /// Returns Some((key, value_part)) if it's an assignment, None otherwise
    fn parse_assignment(line: &str) -> Option<(&str, &str)> {
        // Must contain '=' and NOT start with '[' (section header)
        if line.trim_start().starts_with('[') {
            return None;
        }
        // Use split_once to split at FIRST '=' only (left to right)
        // This preserves any '=' characters in the value
        line.split_once('=').map(|(k, v)| (k.trim(), v.trim()))
    }

    /// Check if a line starts a multi-line content block
    /// Handles whitespace: "content = '''" or "content='''" etc.
    fn is_content_block_start(line: &str) -> bool {
        if let Some((key, value)) = Self::parse_assignment(line) {
            key == "content" && value == "'''"
        } else {
            false
        }
    }

    /// Parse a single TOML file and add templates to the registry
    fn parse_toml(templates: &mut HashMap<String, Template>, content: &str) {
        let mut current_section: Option<String> = None;
        let mut current_block: Option<String> = None;
        let mut in_strings_section = false;
        let mut in_content = false;
        let mut content_lines: Vec<&str> = Vec::new();

        for line in content.lines() {
            // Check for section header [name] or [name.subsection]
            // Skip if we're inside a content block (content may contain [section] patterns)
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') && !in_content {
                // Save previous multi-line template if any
                if let (Some(section), Some(block)) = (&current_section, current_block.take()) {
                    if !content_lines.is_empty() {
                        let name = format!("{}.{}", section, block);
                        let content_str = content_lines.join("\n");
                        templates.insert(
                            name.clone(),
                            Template {
                                name,
                                content: content_str,
                            },
                        );
                        content_lines.clear();
                    }
                }

                let section_name = trimmed
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim()
                    .to_string();

                // Check if this is a .strings section
                if section_name.ends_with(".strings") {
                    in_strings_section = true;
                    current_section = Some(section_name.trim_end_matches(".strings").to_string());
                    current_block = None;
                } else if section_name.contains('.') {
                    // This is a [namespace.block_name] section
                    in_strings_section = false;
                    if let Some((ns, block)) = section_name.split_once('.') {
                        current_section = Some(ns.to_string());
                        current_block = Some(block.to_string());
                    }
                } else {
                    // Top-level section like [codegen]
                    in_strings_section = false;
                    current_section = Some(section_name);
                    current_block = None;
                }
                in_content = false;
                continue;
            }

            // Skip comments and empty lines (outside content blocks)
            if (trimmed.starts_with('#') || trimmed.is_empty()) && !in_content {
                continue;
            }

            // Skip description lines
            if let Some((key, _)) = Self::parse_assignment(line) {
                if key == "description" && !in_content {
                    continue;
                }
            }

            // In strings section: parse key = "value" pairs
            if in_strings_section && !in_content {
                if let Some((key, value_part)) = Self::parse_assignment(line) {
                    // Determine quote character and extract content
                    let quote_char = value_part.chars().next();
                    if let Some(q) = quote_char {
                        if q == '"' || q == '\'' {
                            // Find content between quotes
                            if let Some(start) = value_part.find(q) {
                                let after_first = &value_part[start + 1..];
                                if let Some(end) = after_first.rfind(q) {
                                    let raw = &after_first[..end];
                                    let unescaped = raw
                                        .replace("\\n", "\n")
                                        .replace("\\\"", "\"")
                                        .replace("\\'", "'");

                                    if let Some(section) = &current_section {
                                        let name = format!("{}.{}", section, key);
                                        templates.insert(
                                            name.clone(),
                                            Template {
                                                name,
                                                content: unescaped,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                continue;
            }

            // Start of multi-line content block
            if Self::is_content_block_start(line) && !in_content {
                in_content = true;
                continue;
            }

            // End of multi-line content block
            if trimmed == "'''" && in_content {
                in_content = false;
                // Save the template
                if let (Some(section), Some(block)) = (&current_section, &current_block) {
                    let name = format!("{}.{}", section, block);
                    let content_str = content_lines.join("\n");
                    templates.insert(
                        name.clone(),
                        Template {
                            name,
                            content: content_str,
                        },
                    );
                    content_lines.clear();
                }
                continue;
            }

            // Accumulate content lines
            if in_content {
                content_lines.push(line);
            }
        }

        // Save any remaining template
        if let (Some(section), Some(block)) = (&current_section, current_block) {
            if !content_lines.is_empty() {
                let name = format!("{}.{}", section, block);
                let content_str = content_lines.join("\n");
                templates.insert(
                    name.clone(),
                    Template {
                        name,
                        content: content_str,
                    },
                );
            }
        }
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Get template content by name, exits gracefully if not found
    pub fn get_content(&self, name: &str) -> &str {
        match self.templates.get(name) {
            Some(t) => t.content.as_str(),
            None => {
                eprintln!("warning: Template '{}' not found", name);
                std::process::exit(1);
            }
        }
    }

    /// List all template names
    pub fn names(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

/// Render a template by replacing {{key}} placeholders with values
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Render with a HashMap for convenience
pub fn render_map(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{{{}}}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Format Rust code using rustfmt
/// Returns the original code if rustfmt fails or is unavailable
pub fn format_rust_code(code: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = match Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return code.to_string(),
    };

    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(code.as_bytes()).is_err() {
            return code.to_string();
        }
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).unwrap_or_else(|_| code.to_string())
        }
        _ => code.to_string(),
    }
}

/// Global template instance (lazy initialization)
static TEMPLATES: std::sync::OnceLock<Templates> = std::sync::OnceLock::new();

/// Get the global template registry
pub fn templates() -> &'static Templates {
    TEMPLATES.get_or_init(Templates::load)
}

/// Convenience function to get a template and render it
pub fn get_and_render(name: &str, vars: &[(&str, &str)]) -> String {
    let content = templates().get_content(name);
    render(content, vars)
}

/// Get raw template content by key for codegen namespace
/// Automatically prepends "codegen." namespace
pub fn codegen_template(name: &str) -> Option<&'static str> {
    // If name already contains a '.', assume it's fully qualified
    let full_name = if name.contains('.') {
        name.to_string()
    } else {
        format!("codegen.{}", name)
    };
    templates().get(&full_name).map(|t| t.content.as_str())
}

// =============================================================================
// I18n Message Functions (replaces rust-i18n)
// =============================================================================

/// Get a message by key from the messages namespace
/// Returns the key itself if not found (for debugging)
pub fn message(key: &str) -> String {
    let full_name = format!("messages.{}", key);
    templates()
        .get(&full_name)
        .map(|t| t.content.clone())
        .unwrap_or_else(|| key.to_string())
}

/// Get a message and format it with variable substitution
/// Uses %{name} placeholder syntax (compatible with rust-i18n)
pub fn message_fmt(key: &str, vars: &[(&str, &str)]) -> String {
    let template = message(key);
    let mut result = template;
    for (name, value) in vars {
        let placeholder = format!("%{{{}}}", name);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple() {
        let template = "Hello, {{name}}!";
        let result = render(template, &[("name", "World")]);
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_multiple() {
        let template = "{{greeting}}, {{name}}!";
        let result = render(template, &[("greeting", "Hi"), ("name", "Alice")]);
        assert_eq!(result, "Hi, Alice!");
    }

    #[test]
    fn test_templates_load() {
        let templates = Templates::load();
        // Should have codegen templates
        assert!(templates.get("codegen.function_def_start").is_some());
        assert!(templates.get("codegen.space_brace_open").is_some());
    }

    #[test]
    fn test_codegen_template() {
        let templates = Templates::load();
        let fn_start = templates.get_content("codegen.function_def_start");
        assert_eq!(fn_start, "pub fn ");

        let brace = templates.get_content("codegen.space_brace_open");
        assert_eq!(brace, " { ");
    }

    #[test]
    fn test_runtime_module() {
        let templates = Templates::load();
        let module = templates.get_content("runtime.quiche_module");
        assert!(module.contains("mod quiche"));
        assert!(module.contains("macro_rules!"));
    }

    #[test]
    fn test_project_templates() {
        let templates = Templates::load();
        let cargo = templates.get_content("project.cargo_toml");
        assert!(cargo.contains("[package]"));
        assert!(cargo.contains("{{name}}"));
    }

    #[test]
    fn test_parse_assignment_with_equals_in_value() {
        // Verify split_once splits at first '=' only
        let line = "key = value = more = stuff";
        let result = Templates::parse_assignment(line);
        assert_eq!(result, Some(("key", "value = more = stuff")));
    }

    #[test]
    fn test_whitespace_tolerance() {
        // Various whitespace combinations
        assert_eq!(
            Templates::parse_assignment("key=value"),
            Some(("key", "value"))
        );
        assert_eq!(
            Templates::parse_assignment("key = value"),
            Some(("key", "value"))
        );
        assert_eq!(
            Templates::parse_assignment("key  =  value"),
            Some(("key", "value"))
        );
        assert_eq!(
            Templates::parse_assignment("  key  =  value  "),
            Some(("key", "value"))
        );
    }

    #[test]
    fn test_section_header_not_parsed_as_assignment() {
        // Section headers should not be parsed as assignments
        assert_eq!(Templates::parse_assignment("[section]"), None);
        assert_eq!(Templates::parse_assignment("  [section.sub]  "), None);
    }

    #[test]
    fn test_tabs_and_mixed_whitespace() {
        // Tabs and mixed whitespace should be handled
        assert_eq!(
            Templates::parse_assignment("\tkey\t=\tvalue\t"),
            Some(("key", "value"))
        );
        assert_eq!(
            Templates::parse_assignment("  \t  key  \t  =  \t  value  \t  "),
            Some(("key", "value"))
        );
    }

    #[test]
    fn test_empty_value() {
        // Empty values should parse correctly
        assert_eq!(Templates::parse_assignment("key = "), Some(("key", "")));
        assert_eq!(Templates::parse_assignment("key="), Some(("key", "")));
    }

    #[test]
    fn test_content_block_start_variations() {
        // Various valid ways to start a content block
        assert!(Templates::is_content_block_start("content = '''"));
        assert!(Templates::is_content_block_start("content='''"));
        assert!(Templates::is_content_block_start("content  =  '''"));
        assert!(Templates::is_content_block_start("  content = '''  "));
        assert!(Templates::is_content_block_start("\tcontent\t=\t'''"));

        // These should NOT be content block starts
        assert!(!Templates::is_content_block_start("content = 'single'"));
        assert!(!Templates::is_content_block_start("content = \"double\""));
        assert!(!Templates::is_content_block_start("[content]"));
        assert!(!Templates::is_content_block_start("# content = '''"));
    }

    #[test]
    fn test_value_with_special_chars() {
        // Values containing special characters
        assert_eq!(
            Templates::parse_assignment("key = value#with#hash"),
            Some(("key", "value#with#hash"))
        );
        assert_eq!(
            Templates::parse_assignment("key = [not a section]"),
            Some(("key", "[not a section]"))
        );
    }

    #[test]
    fn test_content_with_section_like_patterns() {
        // Verify content containing [section] patterns is preserved
        let templates = Templates::load();
        let cargo = templates.get_content("project.cargo_toml");
        // Should contain [package], [workspace], [build-dependencies], [dependencies]
        assert!(cargo.contains("[package]"));
        assert!(cargo.contains("[workspace]"));
        assert!(cargo.contains("[build-dependencies]"));
        assert!(cargo.contains("[dependencies]"));
    }
}
