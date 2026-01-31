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
#![allow(unused_macros, unused_imports)]
// Allow in handwritten Rust helpers (generated Quiche code should not use these)
#![allow(clippy::unwrap_used, clippy::expect_used)]

// Export create helpers so they are available without prefix if imported *
// (Actually crate::quiche::... works if we use it)

pub mod compiler;
pub mod quiche {
    pub use quiche_runtime::{QuicheGeneric, QuicheResult, check, deref, mutref, qref};

    pub fn as_str_helper<T: AsRef<str> + ?Sized>(s: &T) -> String {
        s.as_ref().to_string()
    }

    pub fn module_path_for_file(_root: impl AsRef<str>, path: impl AsRef<str>) -> String {
        path.as_ref()
            .replace("/", "::")
            .replace(".rs", "")
            .replace(".qrs", "")
    }

    pub fn module_parent(path: impl AsRef<str>, _level: u32) -> String {
        let p = std::path::Path::new(path.as_ref());
        p.parent().unwrap_or(p).to_string_lossy().to_string()
    }

    pub fn dedup_shadowed_let_mut(s: impl AsRef<str>) -> String {
        s.as_ref().to_string()
    }

    pub fn module_join(a: impl AsRef<str>, b: impl AsRef<str>) -> String {
        format!("{}::{}", a.as_ref(), b.as_ref())
    }

    pub fn path_exists(path: impl AsRef<str>) -> bool {
        std::path::Path::new(path.as_ref()).exists()
    }

    pub fn create_dir_all(path: impl AsRef<str>) -> std::io::Result<()> {
        std::fs::create_dir_all(path.as_ref())
    }

    pub fn write_string(path: impl AsRef<str>, content: impl AsRef<str>) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path.as_ref())?;
        use std::io::Write;
        file.write_all(content.as_ref().as_bytes())?;
        Ok(())
    }

    pub fn set_env_var(k: impl AsRef<str>, v: impl AsRef<str>) {
        unsafe {
            std::env::set_var(k.as_ref(), v.as_ref());
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct ImportMaps {
        pub paths: std::collections::HashMap<String, String>,
        pub kinds: std::collections::HashMap<String, String>,
    }

    pub fn create_ImportMaps(
        paths: std::collections::HashMap<String, String>,
        kinds: std::collections::HashMap<String, String>,
    ) -> ImportMaps {
        ImportMaps { paths, kinds }
    }

    impl ImportMaps {
        pub fn new(
            paths: std::collections::HashMap<String, String>,
            kinds: std::collections::HashMap<String, String>,
        ) -> Self {
            ImportMaps { paths, kinds }
        }
    }

    pub fn current_exe_path() -> String {
        std::env::current_exe()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    pub fn run_cargo_command(cmd: impl AsRef<str>, args: Vec<String>) -> bool {
        true
    }

    pub fn vec_to_list<T: Clone>(v: Vec<T>) -> Vec<T> {
        v
    }

    pub fn push_str_wrapper(s: &mut String, other: impl AsRef<str>) {
        s.push_str(other.as_ref());
    }

    pub fn escape_rust_string(s: impl AsRef<str>) -> String {
        let s = s.as_ref();
        let mut result = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\\' => result.push_str("\\\\"),
                '"' => result.push_str("\\\""),
                '\n' => result.push_str("\\n"),
                '\r' => result.push_str("\\r"),
                '\t' => result.push_str("\\t"),
                c => result.push(c),
            }
        }
        result
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
        crate::run_rust_code(
            user_code,
            script_args,
            quiet,
            suppress_output,
            raw_output,
            warn,
            strict,
        )
    }

    pub fn normalize_module_path(path: impl AsRef<str>) -> String {
        path.as_ref().to_string()
    }

    pub fn compiler_path_for_new() -> String {
        "target/debug/metaquiche".to_string()
    }

    pub fn print_stdout(msg: impl AsRef<str>) {
        println!("{}", msg.as_ref());
    }

    pub fn path_dirname(path: impl AsRef<str>) -> String {
        let p = std::path::Path::new(path.as_ref());
        p.parent().unwrap_or(p).to_string_lossy().to_string()
    }

    pub fn build_module_index(root: impl AsRef<str>) -> std::collections::HashMap<String, String> {
        let root_str = root.as_ref().to_string();
        crate::build_module_index(root_str)
    }

    pub fn env_args_helper() -> Vec<String> {
        std::env::args().collect()
    }
}
pub use compiler::extern_defs;

pub fn as_str_helper<T: AsRef<str> + ?Sized>(s: &T) -> String {
    s.as_ref().to_string()
}

pub fn concat2(a: impl AsRef<str>, b: impl AsRef<str>) -> String {
    format!("{}{}", a.as_ref(), b.as_ref())
}

pub fn concat3(a: impl AsRef<str>, b: impl AsRef<str>, c: impl AsRef<str>) -> String {
    format!("{}{}{}", a.as_ref(), b.as_ref(), c.as_ref())
}

pub fn concat4(
    a: impl AsRef<str>,
    b: impl AsRef<str>,
    c: impl AsRef<str>,
    d: impl AsRef<str>,
) -> String {
    format!("{}{}{}{}", a.as_ref(), b.as_ref(), c.as_ref(), d.as_ref())
}

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;

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

pub fn vec_to_list<T>(v: Vec<T>) -> Vec<T> {
    v
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
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c => result.push(c),
        }
    }
    result
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

pub fn print_stdout(s: String) {
    print!("{}", s);
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
        .args(args.iter())
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
        ($val:expr) => {{
            use crate::quiche::*;
            ($val).quiche_handle()
        }};
    }
    pub(crate) use check;

    macro_rules! call {
        ($func:expr $(, $arg:expr)*) => {{
            use crate::quiche::*;
            $func( $( ($arg).quiche_handle() ),* )
        }};
    }
    pub(crate) use call;

    macro_rules! qref {
        ($e:expr) => { &($e) };
    }
    pub(crate) use qref;

    macro_rules! mutref {
        ($e:expr) => { &mut ($e) };
    }
    pub(crate) use mutref;

    macro_rules! deref {
        ($e:expr) => { *($e) };
    }
    pub(crate) use deref;

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
    full_code.push_str("use crate::quiche as quiche_runtime;\n");
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
        rustc
            .arg("-Awarnings")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
    }

    let status = rustc.status().expect("Failed to run rustc");
    if !status.success() {
        return status.code().unwrap_or(1);
    }

    if suppress_output {
        let status = Command::new("./target/tmp_bin")
            .args(script_args.iter())
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
        .args(script_args.iter())
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
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_string_lossy().into_owned(),
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
        .map(|p| {
            p.iter()
                .filter_map(|c| c.to_str().map(|s| s.to_string()))
                .collect()
        })
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

// Helper to create codegen using extern_defs types
impl crate::compiler::codegen::Codegen {
    pub fn create_codegen(
        output: String,
        tuple_vars: HashMap<String, bool>,
        defined_vars: Vec<HashMap<String, bool>>,
        import_paths: HashMap<String, String>,
        import_kinds: HashMap<String, String>,
        clone_names: bool,
        current_module_path: String,
        class_fields: HashMap<String, HashMap<String, String>>,
        current_class: String,
    ) -> crate::compiler::codegen::Codegen {
        crate::compiler::codegen::Codegen {
            output,
            tuple_vars,
            defined_vars,
            import_paths,
            import_kinds,
            clone_names,
            current_module_path,
            class_fields,
            current_class,
        }
    }
}

mod version_info {
    pub fn get_stage() -> &'static str {
        option_env!("QUICHE_STAGE").unwrap_or("unknown")
    }
    pub fn get_commit() -> &'static str {
        option_env!("QUICHE_COMMIT").unwrap_or("unknown")
    }
    pub fn get_date() -> &'static str {
        option_env!("QUICHE_DATE").unwrap_or("unknown")
    }
    pub fn get_build_kind() -> &'static str {
        option_env!("QUICHE_BUILD_KIND").unwrap_or("unknown")
    }
}

#[cfg(feature = "bootstrap")]
include!("main_gen.rs");

#[cfg(not(feature = "bootstrap"))]
pub mod generated_main {
    #![allow(unused_imports, dead_code, unused_variables)]
    use super::quiche;
    use super::quiche::*;
    use super::{concat2, concat3, concat4};
    use crate::compiler;
    use quiche_runtime::qref;

    // Re-export ast from quiche_parser so generated code can reference ast::
    pub use quiche_parser::ast;

    include!(concat!(env!("OUT_DIR"), "/main.rs"));

    pub fn create_WarnFlags(
        warn: bool,
        strict: bool,
        warn_all: bool,
        warn_quiche: bool,
    ) -> WarnFlags {
        WarnFlags {
            warn,
            strict,
            warn_all,
            warn_quiche,
        }
    }
}

// ast is now accessed via quiche_parser::ast directly

#[cfg(not(feature = "bootstrap"))]
fn main() {
    generated_main::main();
}
