use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src");
    let out_dir = env::var("OUT_DIR").unwrap();

    // Check for lib.qrs, lib.q, main.qrs, or main.q
    let is_lib_qrs = Path::new("src/lib.qrs").exists();
    let is_lib_q = Path::new("src/lib.q").exists();
    let is_lib = is_lib_qrs || is_lib_q;

    let source_path = if is_lib_qrs {
        "src/lib.qrs"
    } else if is_lib_q {
        "src/lib.q"
    } else if Path::new("src/main.qrs").exists() {
        "src/main.qrs"
    } else {
        "src/main.q"
    };
    let dest_name = if is_lib { "lib.rs" } else { "main.rs" };
    let dest_path = Path::new(&out_dir).join(dest_name);

    if Path::new(source_path).exists() {
        // Use QUICHE_COMPILER_BIN if set, otherwise try relative path to bin/quiche
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
            panic!("Compilation failed:\n{}", stderr);
        }
    } else {
        fs::write(&dest_path, "").unwrap();
    }
}
