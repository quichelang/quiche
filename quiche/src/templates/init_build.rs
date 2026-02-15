//! Build script for Quiche crates — compiles .q files to Rust at build time.

use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = Path::new(&manifest_dir).join("src");

    // Find all .q files in src/
    let q_files: Vec<_> = fs::read_dir(&src_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "q"))
        .map(|e| e.path())
        .collect();

    if q_files.is_empty() {
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();

    for q_file in &q_files {
        let stem = q_file.file_stem().unwrap().to_string_lossy();
        let rs_file = Path::new(&out_dir).join(format!("{stem}.rs"));

        // Compile .q → .rs using quiche
        let output = Command::new("quiche")
            .arg("build")
            .arg(q_file)
            .arg("-o")
            .arg(&rs_file)
            .output()
            .unwrap_or_else(|e| {
                panic!("failed to run quiche: {e}\nIs `quiche` installed? Run: cargo install --path <quiche-repo>/quiche")
            });

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "quiche compilation failed for {}:\n{stderr}",
                q_file.display()
            );
        }

        println!("cargo::rerun-if-changed={}", q_file.display());
    }
}
