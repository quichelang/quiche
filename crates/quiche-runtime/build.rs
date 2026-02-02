use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/ast_transformer.qrs");
    let out_dir = env::var("OUT_DIR").unwrap();

    // Compile ast_transformer.qrs using the quiche compiler
    let source_path = "src/ast_transformer.qrs";
    let dest_path = Path::new(&out_dir).join("ast_transformer.rs");

    if Path::new(source_path).exists() {
        // Use QUICHE_COMPILER_BIN if set, otherwise try relative path
        let compiler =
            env::var("QUICHE_COMPILER_BIN").unwrap_or_else(|_| "../../bin/quiche".to_string());

        let output = Command::new(&compiler)
            .arg(source_path)
            .arg("--emit-rust")
            .output()
            .expect(&format!("Failed to run quiche compiler: {}", compiler));

        if output.status.success() {
            fs::write(&dest_path, &output.stdout).expect("Write output failed");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Compilation of ast_transformer.qrs failed:\n{}", stderr);
        }
    }
}
