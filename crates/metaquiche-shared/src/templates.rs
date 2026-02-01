//! Unified template system for Quiche compilers
//!
//! This module provides shared templates used by both metaquiche-host and
//! metaquiche-native to ensure identical code output (stage parity).

use std::collections::HashMap;

/// Raw template content embedded at compile time
const TEMPLATES_TOML: &str = include_str!("../templates.toml");

/// A parsed template with its content and metadata
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub content: String,
}

/// Template registry holding all parsed templates
pub struct Templates {
    templates: HashMap<String, Template>,
}

impl Templates {
    /// Parse and load all templates from the embedded TOML
    pub fn load() -> Self {
        let mut templates = HashMap::new();

        let mut current_name: Option<String> = None;
        let mut current_description = String::new();
        let mut in_content = false;
        let mut content_lines: Vec<&str> = Vec::new();

        for line in TEMPLATES_TOML.lines() {
            // Check for section header [name]
            if line.starts_with('[') && line.ends_with(']') && !line.contains('=') {
                // Save previous template if any
                if let Some(name) = current_name.take() {
                    let content = content_lines.join("\n");
                    templates.insert(
                        name.clone(),
                        Template {
                            name,
                            description: std::mem::take(&mut current_description),
                            content,
                        },
                    );
                    content_lines.clear();
                }

                let name = line
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
                current_name = Some(name);
                in_content = false;
                continue;
            }

            // Skip comments
            if line.starts_with('#') && !in_content {
                continue;
            }

            // Parse description
            if line.starts_with("description = ") && !in_content {
                current_description = line
                    .trim_start_matches("description = ")
                    .trim_matches('"')
                    .to_string();
                continue;
            }

            // Single-line content (content = "..." or content = '...')
            // Handle whitespace-aligned format like: content                          = " { "
            let trimmed = line.trim();
            let is_single_line_content = trimmed.starts_with("content")
                && trimmed.contains('=')
                && (trimmed.contains('"') || trimmed.contains('\''))
                && !trimmed.ends_with("'''")
                && !in_content;

            if is_single_line_content {
                // Find the quote char used (after the '=' sign)
                let eq_pos = trimmed.find('=').unwrap();
                let after_eq = &trimmed[eq_pos + 1..].trim_start();
                let quote_char = after_eq.chars().next().unwrap_or('"');
                if quote_char == '"' || quote_char == '\'' {
                    // Find content between quotes
                    if let Some(first_quote_idx) = line.find(quote_char) {
                        let content_start = first_quote_idx + 1;
                        if let Some(content_end) = line[content_start..].rfind(quote_char) {
                            let raw_content = &line[content_start..content_start + content_end];
                            // Unescape common escape sequences
                            let unescaped = raw_content
                                .replace("\\n", "\n")
                                .replace("\\\"", "\"")
                                .replace("\\'", "'");
                            content_lines.push(unescaped.leak());
                        }
                    }
                }
                continue;
            }

            // Start of multi-line content block
            if line.starts_with("content = '''") && !in_content {
                in_content = true;
                continue;
            }

            // End of multi-line content block
            if line == "'''" && in_content {
                in_content = false;
                continue;
            }

            // Accumulate content lines
            if in_content {
                content_lines.push(line);
            }
        }

        // Save last template
        if let Some(name) = current_name {
            let content = content_lines.join("\n");
            templates.insert(
                name.clone(),
                Template {
                    name,
                    description: current_description,
                    content,
                },
            );
        }

        Templates { templates }
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Get template content by name, panics if not found
    pub fn get_content(&self, name: &str) -> &str {
        self.templates
            .get(name)
            .map(|t| t.content.as_str())
            .unwrap_or_else(|| panic!("Template '{}' not found", name))
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

/// Get raw template content by key, or empty string if not found
/// This allow wrappers to decide how to handle missing keys (e.g. panic)
pub fn codegen_template(name: &str) -> Option<&'static str> {
    // Automatically prepend "codegen." namespace
    let full_name = format!("codegen.{}", name);
    templates().get(&full_name).map(|t| t.content.as_str())
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
        // Should have at least the quiche_module template
        assert!(templates.get("quiche_module").is_some());
    }

    #[test]
    fn test_quiche_module_content() {
        let templates = Templates::load();
        let module = templates.get_content("quiche_module");
        println!("Module content:\n{}", module);
        println!("\n\nModule length: {}", module.len());
        assert!(module.contains("mod quiche"));
        assert!(module.contains("QuicheResult"));
        assert!(module.contains("macro_rules!"));
    }
}
