use quiche_compiler::compile;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

mod templates;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "new" => {
            if args.len() < 3 {
                println!("Usage: quiche new [--lib] <project_name>");
                return;
            }
            if args[2] == "--lib" {
                if args.len() < 4 {
                    println!("Usage: quiche new --lib <project_name>");
                    return;
                }
                create_new_project(&args[3], true);
            } else {
                create_new_project(&args[2], false);
            }
        }
        "build" => {
            run_cargo_command("build", &args[2..]);
        }
        "run" => {
            if Path::new("Cargo.toml").exists() {
                run_cargo_command("run", &args[2..]);
            } else {
                println!("No Cargo.toml found. Did you mean 'quiche <file.qrs>'?");
            }
        }
        "test" => {
            if Path::new("Cargo.toml").exists() {
                run_cargo_command("test", &args[2..]);
            } else {
                // If we are in the quiche repository root (dev mode), run cargo test
                // which triggers tests/runner.rs
                // Use --test runner to avoid running empty lib tests
                let mut test_args = vec!["--test".to_string(), "runner".to_string()];
                test_args.extend_from_slice(&args[2..]);
                run_cargo_command("test", &test_args);
            }
        }
        _ => {
            let filename = &args[1];
            if filename.ends_with(".qrs") {
                run_single_file(filename, &args[2..]);
            } else {
                print_usage();
            }
        }
    }
}

fn print_usage() {
    println!("Usage:");
    println!("  quiche new <name>    Create a new Quiche project");
    println!("  quiche build         Build the current project");
    println!("  quiche run           Run the current project");
    println!("  quiche test          Run project tests");
    println!("  quiche <file.qrs>    Run a single file script");
}

fn create_new_project(name: &str, is_lib: bool) {
    let path = Path::new(name);
    if path.exists() {
        println!("Error: Directory '{}' already exists", name);
        return;
    }

    fs::create_dir_all(path.join("src")).expect("Failed to create src dir");

    fs::write(path.join("Quiche.toml"), templates::get_quiche_toml(name))
        .expect("Failed to write Quiche.toml");
    fs::write(
        path.join("Cargo.toml"),
        templates::get_cargo_toml(name, is_lib),
    )
    .expect("Failed to write Cargo.toml");
    fs::write(path.join("build.rs"), templates::get_build_rs()).expect("Failed to write build.rs");

    if is_lib {
        fs::write(path.join("src/lib.qrs"), templates::get_lib_qrs())
            .expect("Failed to write lib.qrs");
        fs::write(path.join("src/lib.rs"), templates::get_lib_rs())
            .expect("Failed to write lib.rs");
    } else {
        fs::write(path.join("src/main.qrs"), templates::get_main_qrs())
            .expect("Failed to write main.qrs");
        fs::write(path.join("src/main.rs"), templates::get_main_rs())
            .expect("Failed to write main.rs");
    }

    println!("Created new project: {}", name);
}

fn run_cargo_command(cmd: &str, args: &[String]) {
    let status = Command::new("cargo")
        .arg(cmd)
        .args(args)
        .status()
        .expect("Failed to run cargo");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_single_file(filename: &str, script_args: &[String]) {
    let source_raw = fs::read_to_string(filename).expect("Failed to read file");
    let source = source_raw.replace("struct ", "class ");

    // Virtual Module System (Poor Man's Linker)
    // Removed in favor of native macros and Cargo dependencies.
    let dependencies = String::new();

    if let Some(rust_code) = compile(&source) {
        let rust_code = rust_code.replace("#[test]", "");

        let quiche_module = r#"
mod quiche {
    #![allow(unused_macros, unused_imports)]
    
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }
    
    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Exception")
        }
    }
    
    impl<T> QuicheResult for Option<T> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Exception: Unexpected None")
        }
    }
    
    pub trait QuicheGeneric {
        fn quiche_handle(&self) -> Self;
    }
    
    impl<T: Clone> QuicheGeneric for T {
        fn quiche_handle(&self) -> Self {
            self.clone()
        }
    }
    
    macro_rules! call {
        ($func:path, $($arg:expr),*) => {
            {
                use crate::quiche::{QuicheResult, QuicheGeneric};
                $func( $($arg),* ).quiche_handle()
            }
        };
    }
    pub(crate) use call;
}
"#;

        let wrapped_user_code = if !rust_code.contains("fn main") {
            format!("fn main() {{\n{}\n}}\n", rust_code)
        } else {
            rust_code
        };

        // Assemble final code
        let mut full_code = String::new();
        full_code.push_str(
            "#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]\n",
        );
        full_code.push_str(quiche_module);
        full_code.push_str(&dependencies);
        full_code.push_str("\n");
        full_code.push_str(&wrapped_user_code);

        if !Path::new("target").exists() {
            fs::create_dir("target").ok();
        }
        let tmp_rs = "target/tmp.rs";
        fs::write(tmp_rs, full_code).expect("Failed to write temp Rust file");

        println!("--- Compiling and Running ---");
        let status = Command::new("rustc")
            .arg(tmp_rs)
            .arg("--edition")
            .arg("2024")
            .arg("-o")
            .arg("target/tmp_bin")
            .status()
            .expect("Failed to run rustc");

        if status.success() {
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
