use quiche_compiler::compile;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src");
    let out_dir = env::var("OUT_DIR").unwrap();

    // Check for lib.qrs or main.qrs
    let is_lib = Path::new("src/lib.qrs").exists();
    let source_path = if is_lib {
        "src/lib.qrs"
    } else {
        "src/main.qrs"
    };
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
