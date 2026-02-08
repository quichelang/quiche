use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Compile a .qrs file using the external bootstrap compiler binary.
fn compile_qrs(source_name: &str, out_dir: &str, compiler: &str) {
    let source_path = format!("src/{}.qrs", source_name);
    let dest_path = Path::new(out_dir).join(format!("{}.rs", source_name));

    if Path::new(&source_path).exists() {
        let output = Command::new(compiler)
            .arg(&source_path)
            .arg("--emit-rust")
            .output()
            .unwrap_or_else(|_| panic!("Failed to run quiche compiler: {}", compiler));

        if output.status.success() {
            fs::write(&dest_path, &output.stdout).expect("Write output failed");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Compilation of {} failed:\n{}", source_path, stderr);
        }
    }
}

/// Compile a .q file using quiche-bridge (Elevate pipeline) directly as a library.
fn compile_q(source_name: &str, out_dir: &str) {
    let source_path = format!("src/{}.q", source_name);
    let dest_path = Path::new(out_dir).join(format!("{}.rs", source_name));

    if Path::new(&source_path).exists() {
        let source = fs::read_to_string(&source_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", source_path, e));

        match quiche_bridge::compile(&source) {
            Ok(rust_code) => {
                // Post-process: strip #![...] inner attributes — they're invalid
                // when the code is include!()'d inside a `pub mod` block.
                let processed: String = rust_code
                    .lines()
                    .filter(|line| !line.trim_start().starts_with("#!["))
                    .collect::<Vec<_>>()
                    .join("\n");
                fs::write(&dest_path, processed).expect("Write output failed");
            }
            Err(e) => {
                panic!("Elevate compilation of {} failed:\n{}", source_path, e);
            }
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=src/ast_transformer.qrs");
    println!("cargo:rerun-if-changed=src/memory_analysis.qrs");
    println!("cargo:rerun-if-changed=src/qtest.q");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");

    // Use QUICHE_COMPILER_BIN if set, otherwise try relative path
    let compiler =
        env::var("QUICHE_COMPILER_BIN").unwrap_or_else(|_| "../../bin/quiche".to_string());

    // Compile MetaQuiche (.qrs) modules via external compiler binary
    compile_qrs("ast_transformer", &out_dir, &compiler);
    compile_qrs("memory_analysis", &out_dir, &compiler);

    // Compile Quiche (.q) modules via Elevate pipeline (library call)
    compile_q("qtest", &out_dir);
}
