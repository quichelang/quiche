use quiche::compile;
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: quiche-run <file.qrs>");
        return;
    }

    let filename = &args[1];
    let source_raw = fs::read_to_string(filename).expect("Failed to read file");

    // Pre-process: map 'struct' keyword to 'class' for parser compatibility
    let source = source_raw.replace("struct ", "class ");

    if let Some(rust_code) = compile(&source) {
        // Prepare valid Rust with a main wrapper for global code if necessary
        // For now, we assume global code is handled or the user provided a main()
        let wrapped_code = if !rust_code.contains("fn main") {
            format!("fn main() {{\n{}}}\n", rust_code)
        } else {
            rust_code
        };

        let tmp_rs = "target/tmp.rs";
        fs::write(tmp_rs, wrapped_code).expect("Failed to write temp Rust file");

        println!("--- Compiling and Running ---");
        let status = Command::new("rustc")
            .arg(tmp_rs)
            .arg("-o")
            .arg("target/tmp_bin")
            .status()
            .expect("Failed to run rustc");

        if status.success() {
            let output = Command::new("./target/tmp_bin")
                .output()
                .expect("Failed to run binary");

            println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!("Errors:\n{}", String::from_utf8_lossy(&output.stderr));
            }
        } else {
            println!("Compilation failed.");
        }
    }
}
