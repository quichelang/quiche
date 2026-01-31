use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_qrs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_qrs_files(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("qrs") {
                out.push(path);
            }
        }
    }
}

fn module_path_from_rel(rel: &Path) -> (String, bool) {
    let file_name = rel.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let is_mod = file_name == "mod.qrs";
    let mut parts: Vec<String> = Vec::new();
    if let Some(parent) = rel.parent() {
        for comp in parent.iter() {
            if let Some(seg) = comp.to_str() {
                if !seg.is_empty() {
                    parts.push(seg.to_string());
                }
            }
        }
    }
    if !is_mod {
        if let Some(stem) = Path::new(file_name).file_stem().and_then(|s| s.to_str()) {
            parts.push(stem.to_string());
        }
    }
    (parts.join("."), is_mod)
}

fn output_rel_from_rel(rel: &Path, is_mod: bool) -> PathBuf {
    if is_mod {
        if let Some(parent) = rel.parent() {
            return parent.join("mod.rs");
        }
        PathBuf::from("mod.rs")
    } else {
        rel.with_extension("rs")
    }
}

fn clean_generated_rs(dir: &Path) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                clean_generated_rs(&path);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

fn build_error(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    std::process::exit(1);
}

fn main() {
    println!("cargo:rerun-if-changed=src");

    let stage = env::var("QUICHE_STAGE").unwrap_or_else(|_| "stage0".to_string());
    println!("cargo:rustc-env=QUICHE_STAGE={}", stage);

    let commit = std::process::Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=QUICHE_COMMIT={}", commit);

    let date = std::process::Command::new("date")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=QUICHE_DATE={}", date);

    let profile = env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=QUICHE_BUILD_KIND={}", profile);

    let out_dir = match env::var("OUT_DIR") {
        Ok(d) => d,
        Err(_) => build_error("OUT_DIR not set"),
    };
    let out_path = Path::new(&out_dir);
    clean_generated_rs(out_path);

    // 1. Gather all .qrs files (recursive)
    let mut qrs_files = Vec::new();
    let src_dir = Path::new("src");
    if src_dir.exists() {
        collect_qrs_files(src_dir, &mut qrs_files);
    }

    // 2. Identify root (main.qrs or lib.qrs), modules, and children
    let mut top_modules: Vec<String> = Vec::new();
    let mut module_children: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for path in &qrs_files {
        let rel = path.strip_prefix(src_dir).unwrap_or(path);
        let (module_path, is_mod) = module_path_from_rel(rel);
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };

        if stem == "main" || stem == "lib" {
            continue;
        }

        if !module_path.is_empty() {
            if !module_path.contains('.') {
                top_modules.push(module_path.clone());
            }
            if let Some((parent, _)) = module_path.rsplit_once('.') {
                module_children
                    .entry(parent.to_string())
                    .or_default()
                    .push(module_path.clone());
            }

            // If this is a mod.qrs, ensure it can list children later
            if is_mod {
                module_children.entry(module_path.clone()).or_default();
            }
        }
    }

    // 3. Compile all files (requires QUICHE_COMPILER_BIN)
    let bootstrap_bin = match env::var("QUICHE_COMPILER_BIN") {
        Ok(bin) => bin,
        Err(_) => {
            // When running cargo test directly without make, skip Quiche compilation
            // Create stub files with minimal type definitions so main.rs compiles
            let main_stub = r#"// Stub: QUICHE_COMPILER_BIN not set
#[derive(Clone, Debug, Default)]
pub struct WarnFlags {
    pub warn: bool,
    pub strict: bool,
    pub warn_all: bool,
    pub warn_quiche: bool,
}

pub fn main() {
    eprintln!("Stub binary: run 'make stage1' to build the real compiler");
    std::process::exit(1);
}
"#;
            let codegen_stub = r#"// Stub: QUICHE_COMPILER_BIN not set
#[derive(Clone, Debug, Default)]
pub struct Codegen {
    pub output: String,
    pub tuple_vars: std::collections::HashMap<String, bool>,
    pub defined_vars: Vec<std::collections::HashMap<String, bool>>,
    pub import_paths: std::collections::HashMap<String, String>,
    pub import_kinds: std::collections::HashMap<String, String>,
    pub clone_names: bool,
    pub current_module_path: String,
    pub class_fields: std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    pub current_class: String,
}
"#;
            let mod_stub = "// Stub\npub mod codegen;\npub mod type_utils;\npub mod extern_defs;\n";
            let extern_stub = "// Stub\n";
            let type_utils_stub = "// Stub\n";

            let _ = fs::write(out_path.join("main.rs"), main_stub);
            let _ = fs::create_dir_all(out_path.join("compiler"));
            let _ = fs::write(out_path.join("compiler/mod.rs"), mod_stub);
            let _ = fs::write(out_path.join("compiler/codegen.rs"), codegen_stub);
            let _ = fs::write(out_path.join("compiler/type_utils.rs"), type_utils_stub);
            let _ = fs::write(out_path.join("compiler/extern_defs.rs"), extern_stub);
            println!("cargo:warning=QUICHE_COMPILER_BIN not set, using stub types");
            println!("cargo:warning=Use 'make test' for full Quiche compilation");
            return;
        }
    };

    for path in &qrs_files {
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let rel = path.strip_prefix(src_dir).unwrap_or(path);
        let (module_path, is_mod) = module_path_from_rel(rel);
        let out_rel = output_rel_from_rel(rel, is_mod);

        let dest_path = out_path.join(out_rel);
        if let Some(parent) = dest_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                build_error(&format!("Failed to create output dir: {}", e));
            }
        }

        // --- External Binary Transpilation ---
        let output = match std::process::Command::new(&bootstrap_bin)
            .arg(path)
            .arg("--emit-rust")
            .output()
        {
            Ok(o) => o,
            Err(e) => build_error(&format!(
                "Failed to run bootstrap bin {}: {}",
                bootstrap_bin, e
            )),
        };

        if !output.status.success() {
            build_error(&format!(
                "Bootstrap compilation failed for {}:\nstdout: {}\nstderr: {}",
                path.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        let rust_code = match String::from_utf8(output.stdout) {
            Ok(s) => s,
            Err(_) => build_error("Bootstrap output not UTF-8"),
        };
        // -------------------------------------

        let mut final_code = rust_code;

        if is_mod {
            if let Some(children) = module_children.get(&module_path) {
                let mut child_mods: Vec<String> = children
                    .iter()
                    .filter_map(|child| child.rsplit_once('.').map(|(_, name)| name.to_string()))
                    .collect();
                child_mods.sort();
                child_mods.dedup();
                if !child_mods.is_empty() {
                    let mod_decls: String = child_mods
                        .iter()
                        .map(|m| format!("pub mod {};\n", m))
                        .collect();
                    final_code = format!("{}\n{}", mod_decls, final_code);
                }
            }
        }

        if let Err(e) = fs::write(&dest_path, final_code) {
            build_error(&format!("Write output failed: {}", e));
        }
    }
}
