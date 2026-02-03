use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

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

fn main() {
    println!("cargo:rerun-if-changed=src/ast_transformer.qrs");
    println!("cargo:rerun-if-changed=src/memory_analysis.qrs");
    println!("cargo:rerun-if-changed=src/introspect.qrs");
    println!("cargo:rerun-if-changed=src/qtest.qrs");
    println!("cargo:rerun-if-changed=src/pathlib.qrs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");

    // Use QUICHE_COMPILER_BIN if set, otherwise try relative path
    let compiler =
        env::var("QUICHE_COMPILER_BIN").unwrap_or_else(|_| "../../bin/quiche".to_string());

    // Compile Quiche modules
    compile_qrs("ast_transformer", &out_dir, &compiler);
    compile_qrs("memory_analysis", &out_dir, &compiler);
    compile_qrs("introspect", &out_dir, &compiler);
    compile_qrs("qtest", &out_dir, &compiler);
    compile_qrs("pathlib", &out_dir, &compiler);
}
