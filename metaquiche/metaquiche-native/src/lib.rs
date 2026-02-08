//! metaquiche-native library interface
//!
//! Provides the compile() function for use as a build dependency.
//! This allows Quiche projects to use the native compiler (with full features)
//! instead of the host compiler.

#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

use metaquiche_shared::telemetry::{CompileContext, Diagnostic, Emitter};
use std::collections::HashSet;

/// Compile Quiche source code to Rust
///
/// This is the main entry point for build.rs files.
pub fn compile(source: &str, filename: &str) -> Option<String> {
    let ctx = CompileContext::new(filename, source);

    // Parse the source using metaquiche_parser
    let parsed = match metaquiche_parser::parse(source) {
        Ok(module) => module,
        Err(e) => {
            Emitter::print_failed_header(filename);
            let err_str = e.to_string();
            let diag = Diagnostic::error(&err_str);
            eprintln!(
                "{}",
                metaquiche_shared::telemetry::format_diagnostic(&diag, Some(&ctx))
            );
            return None;
        }
    };

    // Generate Rust code using metaquiche-host's codegen
    let mut cg = quiche_host::Codegen::new();
    let rust_code = cg.generate_module(&parsed);

    Some(dedup_shadowed_let_mut(&rust_code))
}

fn dedup_shadowed_let_mut(code: &str) -> String {
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
    let mut out = String::new();

    for line in code.lines() {
        let mut line_out = line.to_string();
        let mut in_string = false;
        let mut escape = false;

        // Calculate scope POPS (closing braces)
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if !in_string && ch == '}' && scopes.len() > 1 {
                scopes.pop();
            }
        }

        // Search and replace "let mut " if shadowed
        let mut search_start = 0;
        loop {
            if let Some(idx) = line_out[search_start..].find("let mut ") {
                let abs_idx = search_start + idx;
                let prefix = &line_out[0..abs_idx];
                let mut p_in_string = false;
                let mut p_escape = false;
                for c in prefix.chars() {
                    if p_escape {
                        p_escape = false;
                        continue;
                    }
                    if c == '\\' {
                        p_escape = true;
                        continue;
                    }
                    if c == '"' {
                        p_in_string = !p_in_string;
                    }
                }

                if p_in_string {
                    search_start = abs_idx + 8;
                    continue;
                }

                let name_start = abs_idx + "let mut ".len();
                let mut name_end = name_start;
                for (i, c) in line_out[name_start..].char_indices() {
                    if c.is_alphanumeric() || c == '_' {
                        name_end = name_start + i + c.len_utf8();
                    } else {
                        break;
                    }
                }
                if name_end == name_start {
                    search_start = name_start;
                    continue;
                }

                let name = line_out[name_start..name_end].to_string();
                let shadowed = scopes
                    .iter()
                    .take(scopes.len().saturating_sub(1))
                    .any(|s| s.contains(&name));

                if shadowed {
                    line_out.replace_range(abs_idx..name_start, "");
                    search_start = abs_idx;
                } else if let Some(cur) = scopes.last_mut() {
                    cur.insert(name);
                    search_start = name_end;
                } else {
                    search_start = name_end;
                }
            } else {
                break;
            }
        }

        // Calculate scope PUSHES (opening braces)
        in_string = false;
        escape = false;
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = !in_string;
                continue;
            }
            if !in_string && ch == '{' {
                scopes.push(HashSet::new());
            }
        }

        out.push_str(&line_out);
        out.push('\n');
    }

    out
}
