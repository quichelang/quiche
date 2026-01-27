use quiche_compiler::compile;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src");
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);

    // 1. Gather all .qrs files
    let mut qrs_files = Vec::new();
    let src_dir = Path::new("src");
    if src_dir.exists() {
        for entry in fs::read_dir(src_dir).expect("Failed to read src dir") {
            let entry = entry.expect("Failed to read entry");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("qrs") {
                qrs_files.push(path);
            }
        }
    }

    // 2. Identify root (main.qrs or lib.qrs) and modules
    let mut root_file = None;
    let mut modules = Vec::new();

    for path in &qrs_files {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        if stem == "main" || stem == "lib" {
            root_file = Some((path, stem));
        } else {
            modules.push(stem.to_string());
        }
    }

    // 3. Compile all files
    for path in &qrs_files {
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let source = fs::read_to_string(path).expect("Read source failed");

        // HACK: Removed obsolete struct-to-class replacement.
        let mut source = source.to_string();

        // If this is the root file, inject hint for the compiler
        if let Some((_, root_stem)) = root_file {
            if stem == root_stem {
                let link_hint = format!("\"quiche:link={}\"\n", modules.join(","));
                source = format!("{}\n{}", link_hint, source);
            }
        }

        if let Some(mut rust_code) = compile(&source) {
            // If this is the root file, also inject `mod` declarations for linking
            if let Some((_, root_stem)) = root_file {
                if stem == root_stem {
                    let mod_decls: String = modules
                        .iter()
                        .map(|m| format!("pub mod {};\n", m))
                        .collect();
                    rust_code = format!("{}\n{}", mod_decls, rust_code);
                }
            }

            let dest_name = format!("{}.rs", stem);
            let dest_path = out_path.join(dest_name);
            fs::write(&dest_path, rust_code).expect("Write output failed");
        } else {
            panic!("Compilation failed for {}", path.display());
        }
    }

    // Ensure main.rs exists if we only have lib.qrs?
    // Cargo expects main.rs for bin, lib.rs for lib.
    // If we have main.qrs -> main.rs.
}
