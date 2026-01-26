// Project Scaffolding Templates

pub fn get_quiche_toml(name: &str) -> String {
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
"#,
        name
    )
}

pub fn get_cargo_toml(name: &str, is_lib: bool) -> String {
    let mut s = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

# Break out of any parent workspace
[workspace]

[build-dependencies]
quiche_compiler = {{ path = "../crates/compiler" }}

[dependencies]
quiche_runtime = {{ path = "../quiche_runtime" }}
"#,
        name
    );

    if is_lib {
        s.push_str("\n[lib]\npath = \"src/lib.rs\"\n");
    } else {
        s.push_str("\n[[bin]]\nname = \"");
        s.push_str(name);
        s.push_str("\"\npath = \"src/main.rs\"\n");
    }
    s
}

pub fn get_build_rs() -> &'static str {
    r#"
use std::env;
use std::fs;
use std::path::Path;
use quiche_compiler::compile;

fn main() {
    println!("cargo:rerun-if-changed=src");
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Check for lib.qrs or main.qrs
    let is_lib = Path::new("src/lib.qrs").exists();
    let source_path = if is_lib { "src/lib.qrs" } else { "src/main.qrs" };
    let dest_name = if is_lib { "lib.rs" } else { "main.rs" };
    let dest_path = Path::new(&out_dir).join(dest_name);

    if Path::new(source_path).exists() {
        let source = fs::read_to_string(source_path).expect("Read source failed");
        let source = source.replace("struct ", "class ");
        
        if let Some(rust_code) = compile(&source) {
            fs::write(&dest_path, rust_code).expect("Write output failed");
        } else {
            panic!("Compilation failed");
        }
    } else {
        fs::write(&dest_path, "").unwrap();
    }
}
"#
}

pub fn get_lib_qrs() -> &'static str {
    r#"
def hello():
    print("Hello from Lib!")
"#
}

pub fn get_lib_rs() -> &'static str {
    r#"
// Re-export everything from the transpiled module
include!(concat!(env!("OUT_DIR"), "/lib.rs"));
"#
}

pub fn get_main_qrs() -> &'static str {
    r#"
def main():
    print("Hello, Quiche!")
"#
}

pub fn get_main_rs() -> &'static str {
    r#"
// Alias quiche_runtime::test to lib::test for backward compatibility with existing tests
// In improved version, we'd map standard library imports in the compiler.
pub mod lib {
    pub use quiche_runtime as test;
}

include!(concat!(env!("OUT_DIR"), "/main.rs"));
"#
}
