#![allow(
    dead_code,
    unreachable_code,
    unreachable_patterns,
    unused_assignments,
    unused_imports,
    unused_mut,
    unused_parens,
    unused_variables
)]

mod quiche {
    #![allow(unused_macros, unused_imports)]

    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }

    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Exception")
        }
    }

    // Low Priority: Takes &Self (Clone fallback)
    pub trait QuicheGeneric {
        fn quiche_handle(&self) -> Self;
    }

    impl<T: Clone> QuicheGeneric for T {
        fn quiche_handle(&self) -> Self {
            self.clone()
        }
    }

    macro_rules! check {
        ($val:expr) => {{
            use crate::quiche::{QuicheGeneric, QuicheResult};
            ($val).quiche_handle()
        }};
    }
    pub(crate) use check;
    pub(crate) use check as call;

    pub fn env_args_helper() -> Vec<String> {
        std::env::args().collect()
    }

    pub fn push_str_wrapper(mut s: String, val: String) -> String {
        s.push_str(&val);
        s
    }

    pub fn escape_rust_string(s: String) -> String {
        s.replace('\\', "\\\\").replace('\"', "\\\"")
    }

    pub fn run_test_cmd(exe: String, test_path: String) -> bool {
        use std::process::Stdio;
        let mut cmd = std::process::Command::new(exe);
        cmd.arg(test_path);
        cmd.env("QUICHE_QUIET", "1");
        cmd.env("QUICHE_SUPPRESS_OUTPUT", "1");
        cmd.stdout(Stdio::null());
        cmd.stderr(Stdio::null());
        match cmd.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    pub fn list_test_files() -> Vec<String> {
        let mut tests = Vec::new();
        if let Ok(entries) = std::fs::read_dir("tests") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".qrs") || name == "runner.qrs" {
                    continue;
                }
                tests.push(name);
            }
        }
        tests.sort();
        tests
    }

    pub fn path_exists(path: String) -> bool {
        Path::new(&path).exists()
    }

    pub fn create_dir_all(path: String) {
        let _ = std::fs::create_dir_all(path);
    }

    pub fn write_string(path: String, contents: String) {
        std::fs::write(path, contents).expect("Failed to write file");
    }

    pub fn set_env_var(key: String, value: String) {
        unsafe {
            std::env::set_var(key, value);
        }
    }

    pub fn current_exe_path() -> String {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "".to_string())
    }

    pub fn compiler_path_for_new() -> String {
        let compiler_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("compiler")
            .canonicalize()
            .unwrap_or_else(|_| Path::new("../crates/compiler").to_path_buf());
        compiler_path.to_str().unwrap_or("").replace("\\", "/")
    }

    pub fn run_cargo_command(cmd: String, args: Vec<String>) -> i32 {
        let status = Command::new("cargo")
            .arg(cmd)
            .args(args)
            .status()
            .expect("Failed to run cargo");
        if status.success() {
            0
        } else {
            status.code().unwrap_or(1)
        }
    }

    pub fn run_rust_code(
        user_code: String,
        script_args: Vec<String>,
        quiet: bool,
        suppress_output: bool,
        raw_output: bool,
        warn: bool,
        strict: bool,
    ) -> i32 {
        let rust_code = user_code.replace("#[test]", "");

        let quiche_module = r#"
mod quiche {
    #![allow(unused_macros, unused_imports)]
    
    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }
    
    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Exception")
        }
    }
    

    
    // Low Priority: Takes &Self (Clone fallback)
    pub trait QuicheGeneric {
        fn quiche_handle(&self) -> Self;
    }
    
    impl<T: Clone> QuicheGeneric for T {
        fn quiche_handle(&self) -> Self {
            self.clone()
        }
    }
    
    macro_rules! check {
        ($val:expr) => {
            {
                use crate::quiche::{QuicheResult, QuicheGeneric};
                ($val).quiche_handle()
            }
        };
    }
    pub(crate) use check;
    pub(crate) use check as call;

    pub fn run_test_cmd(exe: String, test_path: String) -> bool {
        let mut cmd = std::process::Command::new(exe);
        cmd.arg(test_path);
        cmd.env("QUICHE_QUIET", "1");
        cmd.env("QUICHE_SUPPRESS_OUTPUT", "1");
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        match cmd.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    pub fn list_test_files() -> Vec<String> {
        let mut tests = Vec::new();
        if let Ok(entries) = std::fs::read_dir("tests") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".qrs") || name == "runner.qrs" {
                    continue;
                }
                tests.push(name);
            }
        }
        tests.sort();
        tests
    }
}
"#;

        let wrapped_user_code = if !rust_code.contains("fn main") {
            format!("fn main() {{\n{}\n}}\n", rust_code)
        } else {
            rust_code
        };

        let mut full_code = String::new();
        full_code.push_str(
            "#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]\n",
        );
        full_code.push_str(quiche_module);
        full_code.push_str("\n");
        full_code.push_str(&wrapped_user_code);

        if !Path::new("target").exists() {
            std::fs::create_dir("target").ok();
        }
        let tmp_rs = "target/tmp.rs";
        std::fs::write(tmp_rs, full_code).expect("Failed to write temp Rust file");

        if !quiet {
            println!("--- Compiling and Running ---");
        }
        let mut rustc = Command::new("rustc");
        rustc
            .arg(tmp_rs)
            .arg("--edition")
            .arg("2024")
            .arg("-o")
            .arg("target/tmp_bin");

        if strict {
            rustc.arg("-D").arg("warnings");
        }
        if quiet && !warn && !strict {
            rustc.arg("-Awarnings").stdout(Stdio::null()).stderr(Stdio::null());
        }

        let status = rustc.status().expect("Failed to run rustc");
        if !status.success() {
            return status.code().unwrap_or(1);
        }

        if suppress_output {
            let status = Command::new("./target/tmp_bin")
                .args(script_args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .expect("Failed to run binary");
            if !status.success() {
                return status.code().unwrap_or(1);
            }
            return 0;
        }

        let output = Command::new("./target/tmp_bin")
            .args(script_args)
            .output()
            .expect("Failed to run binary");

        if raw_output {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!("Errors:\n{}", String::from_utf8_lossy(&output.stderr));
            }
        }

        if !output.status.success() {
            return output.status.code().unwrap_or(1);
        }
        0
    }

    pub fn dedup_shadowed_let_mut(code: String) -> String {
        use std::collections::HashSet;
        let mut out = String::new();
        let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];

        for line in code.lines() {
            let mut line_out = line.to_string();

            for ch in line.chars() {
                if ch == '}' && scopes.len() > 1 {
                    scopes.pop();
                }
            }

            let mut search_start = 0;
            loop {
                if let Some(idx) = line_out[search_start..].find("let mut ") {
                    let abs_idx = search_start + idx;
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
                    } else if let Some(cur) = scopes.last_mut() {
                        cur.insert(name);
                    }
                    search_start = name_end;
                } else {
                    break;
                }
            }

            for ch in line.chars() {
                if ch == '{' {
                    scopes.push(HashSet::new());
                }
            }

            out.push_str(&line_out);
            out.push('\n');
        }

        out
    }

    pub fn path_dirname(path: String) -> String {
        let p = Path::new(&path);
        match p.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                parent.to_string_lossy().into_owned()
            }
            _ => ".".to_string(),
        }
    }

    pub fn module_parent(path: String, levels: u32) -> String {
        if path.is_empty() {
            return "".to_string();
        }
        let mut parts: Vec<&str> = path.split('.').collect();
        let mut remaining = levels;
        while remaining > 0 && !parts.is_empty() {
            parts.pop();
            remaining -= 1;
        }
        parts.join(".")
    }

    pub fn module_join(base: String, sub: String) -> String {
        if base.is_empty() {
            return sub;
        }
        if sub.is_empty() {
            return base;
        }
        format!("{base}.{sub}")
    }

    fn module_path_from_relative(rel: &Path) -> String {
        let file_name = match rel.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => return "".to_string(),
        };
        let mut parts: Vec<String> = rel
            .parent()
            .map(|p| p.iter().filter_map(|c| c.to_str().map(|s| s.to_string())).collect())
            .unwrap_or_else(Vec::new);

        if file_name == "mod.qrs" {
            if parts.is_empty() {
                return "".to_string();
            }
            return parts.join(".");
        }

        if let Some(stem) = Path::new(file_name).file_stem().and_then(|s| s.to_str()) {
            parts.push(stem.to_string());
        }
        parts.join(".")
    }

    fn collect_qrs_files(root: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_qrs_files(&path, out);
                } else if path.extension().and_then(|e| e.to_str()) == Some("qrs") {
                    out.push(path);
                }
            }
        }
    }

    pub fn build_module_index(root: String) -> HashMap<String, String> {
        let root_path = PathBuf::from(root);
        let mut files = Vec::new();
        collect_qrs_files(&root_path, &mut files);
        let mut index = HashMap::new();
        for file in files {
            let rel = match file.strip_prefix(&root_path) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let module_path = module_path_from_relative(rel);
            index.insert(module_path, file.to_string_lossy().into_owned());
        }
        index
    }

    pub fn module_path_for_file(root: String, filename: String) -> String {
        let root_path = PathBuf::from(root);
        let file_path = PathBuf::from(filename);
        let rel = match file_path.strip_prefix(&root_path) {
            Ok(r) => r,
            Err(_) => return "".to_string(),
        };
        module_path_from_relative(rel)
    }
}

#[cfg(feature = "bootstrap")]
include!("main_gen.rs");

#[cfg(not(feature = "bootstrap"))]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
