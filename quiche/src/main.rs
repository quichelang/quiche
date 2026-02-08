use std::env;
use std::fs;
use std::process::{self, Command};

use quiche::{CompileOptions, ExperimentFlags};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for --help
    if args.iter().any(|a| a == "--help" || a == "-h") || args.len() < 2 {
        print_usage();
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    let filename = &args[1];
    let emit_rust = has_flag(&args, "--emit-rust");
    let emit_elevate = has_flag(&args, "--emit-elevate");

    // Parse experiment flags
    let experiments = ExperimentFlags {
        move_mut_args: has_flag(&args, "--exp-move-mut-args"),
        infer_local_bidi: has_flag(&args, "--exp-infer-local-bidi"),
        effect_rows_internal: has_flag(&args, "--exp-effect-rows"),
        infer_principal_fallback: has_flag(&args, "--exp-infer-principal-fallback"),
        numeric_coercion: has_flag(&args, "--exp-numeric-coercion"),
    };

    // Parse compile options
    let options = CompileOptions {
        experiments,
        fail_on_hot_clone: has_flag(&args, "--fail-on-hot-clone"),
        ..Default::default()
    };

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to read '{}': {}", filename, e);
            process::exit(1);
        }
    };

    if emit_elevate {
        // Show the parsed Elevate AST
        match quiche::parse(&source) {
            Ok(module) => println!("{:#?}", module),
            Err(e) => {
                eprintln!("Parse error:\n{}", e);
                process::exit(1);
            }
        }
        return;
    }

    // Show active experiments
    let active: Vec<&str> = [
        (options.experiments.move_mut_args, "move_mut_args"),
        (options.experiments.infer_local_bidi, "infer_local_bidi"),
        (options.experiments.effect_rows_internal, "effect_rows"),
        (
            options.experiments.infer_principal_fallback,
            "infer_principal_fallback",
        ),
        (options.experiments.numeric_coercion, "numeric_coercion"),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|(_, name)| *name)
    .collect();
    if !active.is_empty() {
        eprintln!("🧪 experiments: {}", active.join(", "));
    }
    if options.fail_on_hot_clone {
        eprintln!("🔒 fail-on-hot-clone enabled");
    }

    match quiche::compile_with_options(&source, &options) {
        Ok(rust_code) => {
            if emit_rust {
                print!("{}", rust_code);
            } else {
                // Default: compile and run
                run_rust_code(&rust_code);
            }
        }
        Err(e) => {
            eprintln!("Compile error:\n{}", e);
            process::exit(1);
        }
    }
}

fn run_rust_code(rust_code: &str) {
    let tmp_dir = env::temp_dir().join("quiche");
    fs::create_dir_all(&tmp_dir).unwrap_or_else(|e| {
        eprintln!("Error: Failed to create temp dir: {}", e);
        process::exit(1);
    });

    let rs_path = tmp_dir.join("__quiche_run.rs");
    let bin_path = tmp_dir.join("__quiche_run");

    fs::write(&rs_path, rust_code).unwrap_or_else(|e| {
        eprintln!("Error: Failed to write temp file: {}", e);
        process::exit(1);
    });

    // Compile with rustc
    let compile = Command::new("rustc")
        .arg(&rs_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("--edition")
        .arg("2021")
        .output();

    match compile {
        Ok(output) if output.status.success() => {
            // Run the compiled binary
            let run = Command::new(&bin_path).status();
            match run {
                Ok(status) => process::exit(status.code().unwrap_or(1)),
                Err(e) => {
                    eprintln!("Error: Failed to run binary: {}", e);
                    process::exit(1);
                }
            }
        }
        Ok(output) => {
            eprintln!(
                "rustc failed:\n{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: Failed to invoke rustc: {}", e);
            eprintln!("Make sure rustc is installed (https://rustup.rs)");
            process::exit(1);
        }
    }
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

fn print_usage() {
    eprintln!(
        "\
quiche - Python-flavoured Rust compiler

USAGE:
    quiche <file.q> [OPTIONS]

By default, quiche compiles and runs the script.

OPTIONS:
    --emit-rust              Emit generated Rust code to stdout
    --emit-elevate           Emit parsed Elevate AST to stdout
    -h, --help               Show this help message

EXPERIMENT FLAGS:
    --exp-move-mut-args           Mutable argument ownership transfer
    --exp-infer-local-bidi        Bidirectional local type inference
    --exp-effect-rows             Internal effect row types
    --exp-infer-principal-fallback  Principal type fallback inference
    --exp-numeric-coercion        Automatic numeric type coercion

COMPILER OPTIONS:
    --fail-on-hot-clone           Error instead of warn on implicit clones

EXAMPLES:
    quiche hello.q                          # compile + run
    quiche hello.q --emit-rust              # show generated Rust
    quiche hello.q --emit-elevate           # show parsed AST
    quiche hello.q --exp-infer-local-bidi   # run with type inference"
    );
}
