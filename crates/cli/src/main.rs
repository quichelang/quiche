use quiche_compiler::compile;
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

    // Virtual Module System (Poor Man's Linker)
    // Scan for `from lib.test` or `import lib.test`
    // Hardcoded for lib.test support for now to enable test suite
    let mut dependencies = String::new();
    if source.contains("lib.test") {
        let lib_path = "lib/test.qrs";
        if let Ok(lib_source) = fs::read_to_string(lib_path) {
            let lib_source = lib_source.replace("struct ", "class ");
            if let Some(rust_code) = compile(&lib_source) {
                dependencies.push_str("pub mod lib {\n");
                dependencies.push_str("    pub mod test {\n");
                // Indent code
                for line in rust_code.lines() {
                    let pub_line = line.replace("fn ", "pub fn ");
                    dependencies.push_str("        ");
                    dependencies.push_str(&pub_line);
                    dependencies.push_str("\n");
                }
                dependencies.push_str("    }\n");
                dependencies.push_str("}\n");
            }
        }
    }

    if let Some(rust_code) = compile(&source) {
        // Prepare valid Rust with dependencies
        let mut full_code = String::new();
        // Add dependencies first
        full_code.push_str(&dependencies);
        full_code.push_str("\n");
        full_code.push_str(&rust_code);

        let wrapped_code = if !full_code.contains("fn main") {
            format!("fn main() {{\n{}}}\n", full_code)
        } else {
            full_code
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
            // Forward arguments to the compiled binary
            // args[0] is quiche-run, args[1] is filename, args[2..] are for the script
            let script_args = if args.len() > 2 { &args[2..] } else { &[] };

            let output = Command::new("./target/tmp_bin")
                .args(script_args)
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
