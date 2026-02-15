use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

// ─────────────────────────────────────────────────────────────────────────────
// Flag Definitions — single source of truth for CLI parsing AND help text
// ─────────────────────────────────────────────────────────────────────────────

#[allow(dead_code)]
struct FlagDef {
    flag: &'static str,
    description: &'static str,
    aliases: &'static [&'static str],
}

/// Experiment flags enabled by default in Quiche.
const DEFAULT_EXPERIMENTS: &[FlagDef] = &[
    FlagDef {
        flag: "--exp-move-mut-args",
        description: "Mutable argument ownership transfer",
        aliases: &["--exp-mov-mut-args"],
    },
    FlagDef {
        flag: "--exp-type-system",
        description: "Enhanced type system with inference",
        aliases: &[],
    },
];

/// Experiment flags that are opt-in (not enabled by default).
const OPTIN_EXPERIMENTS: &[FlagDef] = &[];

/// Non-experiment compiler options.
const COMPILER_OPTIONS: &[FlagDef] = &[FlagDef {
    flag: "--fail-on-hot-clone",
    description: "Error instead of warn on implicit clones",
    aliases: &[],
}];

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check for --help
    if args.iter().any(|a| a == "--help" || a == "-h") || args.len() < 2 {
        print_usage();
        process::exit(if args.len() < 2 { 1 } else { 0 });
    }

    // Subcommand dispatch
    if args[1] == "init" {
        run_init(&args[2..]);
        return;
    }
    if args[1] == "build" {
        run_build(&args[2..]);
        return;
    }

    let filename = &args[1];
    let emit_rust = has_flag(&args, "--emit-rust");
    let emit_elevate = has_flag(&args, "--emit-elevate");
    let dump_ast = has_flag(&args, "--emit-ast");

    // Start with defaults (core experiments enabled)
    let mut options = quiche::default_options();

    // Allow CLI overrides for individual experiment flags
    if has_any_flag(&args, "--exp-move-mut-args", &["--exp-mov-mut-args"]) {
        options.experiments.move_mut_args = false;
    }
    if has_flag(&args, "--exp-type-system") {
        options.experiments.type_system = true;
    }
    if has_flag(&args, "--fail-on-hot-clone") {
        options.fail_on_hot_clone = true;
    }

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to read '{}': {}", filename, e);
            process::exit(1);
        }
    };

    if dump_ast {
        // Debug dump of the parsed Elevate AST (includes metadata/spans)
        match quiche::parse(&source) {
            Ok(module) => println!("{:#?}", module),
            Err(e) => {
                eprintln!("Parse error:\n{}", e);
                process::exit(1);
            }
        }
        return;
    }

    if emit_elevate {
        // Emit valid Elevate source from the typed IR
        match quiche::emit_elevate(&source, &options) {
            Ok(elevate_src) => print!("{}", elevate_src),
            Err(e) => {
                eprintln!("Error:\n{}", e);
                process::exit(1);
            }
        }
        return;
    }

    // Show active experiments
    let active: Vec<&str> = [
        (options.experiments.move_mut_args, "move_mut_args"),
        (options.experiments.type_system, "type_system"),
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

    match quiche::compile_file(&source, filename, &options) {
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
    // Locate quiche-lib rlib for --extern
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir)
        .parent()
        .expect("workspace root");
    let rlib = workspace_root.join("target/debug/libquiche_lib.rlib");
    let deps_dir = workspace_root.join("target/debug/deps");

    let compile = Command::new("rustc")
        .arg(&rs_path)
        .arg("-o")
        .arg(&bin_path)
        .arg("--edition")
        .arg("2021")
        .arg("--extern")
        .arg(format!("quiche_lib={}", rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", deps_dir.display()))
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

fn has_any_flag(args: &[String], flag: &str, aliases: &[&str]) -> bool {
    args.iter()
        .any(|a| a == flag || aliases.iter().any(|alias| a == alias))
}

fn print_usage() {
    eprintln!(
        "quiche - Python-flavoured Rust compiler\n\
         \n\
         USAGE:\n\
         \x20   quiche <file.q> [OPTIONS]\n\
         \x20   quiche init <path> [cargo init flags]\n\
         \n\
         By default, quiche compiles and runs the script.\n\
         Core experiment flags are enabled by default.\n\
         \n\
         OPTIONS:\n\
         \x20   --emit-rust              Emit generated Rust code to stdout\n\
         \x20   --emit-elevate           Emit Elevate (.ers) source to stdout\n\
         \x20   --emit-ast               Dump raw AST with metadata (debug)\n\
         \x20   -h, --help               Show this help message"
    );

    if !DEFAULT_EXPERIMENTS.is_empty() {
        eprintln!("\nEXPERIMENT FLAGS (enabled by default):");
        for def in DEFAULT_EXPERIMENTS {
            eprintln!("    {:<35}{}", def.flag, def.description);
        }
    }

    if !OPTIN_EXPERIMENTS.is_empty() {
        eprintln!("\nEXPERIMENT FLAGS (opt-in):");
        for def in OPTIN_EXPERIMENTS {
            eprintln!("    {:<35}{}", def.flag, def.description);
        }
    }

    if !COMPILER_OPTIONS.is_empty() {
        eprintln!("\nCOMPILER OPTIONS:");
        for def in COMPILER_OPTIONS {
            eprintln!("    {:<35}{}", def.flag, def.description);
        }
    }

    eprintln!(
        "\nEXAMPLES:\n\
         \x20   quiche hello.q                          # compile + run\n\
         \x20   quiche hello.q --emit-rust              # show generated Rust\n\
         \x20   quiche hello.q --emit-elevate           # show parsed AST"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// quiche build — compile .q to .rs
// ─────────────────────────────────────────────────────────────────────────────

fn run_build(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: quiche build <file.q> [-o <output.rs>]");
        process::exit(2);
    }

    let filename = &args[0];
    let output_path = args
        .windows(2)
        .find(|w| w[0] == "-o")
        .map(|w| PathBuf::from(&w[1]));

    let source = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to read '{}': {}", filename, e);
            process::exit(1);
        }
    };

    let options = quiche::default_options();
    match quiche::compile_file(&source, filename, &options) {
        Ok(rust_code) => {
            if let Some(path) = output_path {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).unwrap_or_else(|e| {
                        eprintln!("Error: Failed to create dir {}: {}", parent.display(), e);
                        process::exit(1);
                    });
                }
                fs::write(&path, rust_code).unwrap_or_else(|e| {
                    eprintln!("Error: Failed to write '{}': {}", path.display(), e);
                    process::exit(1);
                });
            } else {
                print!("{}", rust_code);
            }
        }
        Err(e) => {
            eprintln!("Compile error:\n{}", e);
            process::exit(1);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// quiche init — scaffolds a Quiche crate (modeled on Elevate's init)
// ─────────────────────────────────────────────────────────────────────────────

fn run_init(args: &[String]) {
    if args.is_empty() {
        eprintln!("usage: quiche init <crate-root> [cargo init flags]");
        process::exit(2);
    }

    let crate_root = PathBuf::from(&args[0]);

    // Run cargo init with passthrough args
    let mut init = Command::new("cargo");
    init.arg("init");
    init.args(args);
    let status = init.status().unwrap_or_else(|error| {
        eprintln!("failed to run cargo init: {error}");
        process::exit(1);
    });
    if !status.success() {
        process::exit(status.code().unwrap_or(1));
    }

    apply_quiche_templates(&crate_root).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });

    println!("initialized quiche crate at {}", crate_root.display());
}

fn apply_quiche_templates(crate_root: &Path) -> Result<(), String> {
    let src_dir = crate_root.join("src");
    fs::create_dir_all(&src_dir)
        .map_err(|e| format!("failed to create {}: {e}", src_dir.display()))?;

    // Write build.rs
    fs::write(
        crate_root.join("build.rs"),
        include_str!("templates/init_build.rs").as_bytes(),
    )
    .map_err(|e| format!("failed to write build.rs: {e}"))?;

    // Write src/main.q
    let main_q = src_dir.join("main.q");
    if !main_q.exists() {
        fs::write(&main_q, include_str!("templates/init_main.q").as_bytes())
            .map_err(|e| format!("failed to write {}: {e}", main_q.display()))?;
    }

    // Overwrite src/main.rs to include the generated code
    fs::write(
        src_dir.join("main.rs"),
        "// Auto-generated by `quiche init` -- do not edit.\n\
           // Your source code lives in src/main.q\n\
           include!(concat!(env!(\"OUT_DIR\"), \"/main.rs\"));\n",
    )
    .map_err(|e| format!("failed to write main.rs: {e}"))?;

    // Add quiche-lib dependency to Cargo.toml
    let manifest_path = crate_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("failed to read {}: {e}", manifest_path.display()))?;
    if !manifest.contains("quiche-lib") {
        let updated = if manifest.contains("[dependencies]") {
            manifest.replace(
                "[dependencies]",
                "[dependencies]\nquiche-lib = { path = \"../lib\" }",
            )
        } else {
            format!("{manifest}\n[dependencies]\nquiche-lib = {{ path = \"../lib\" }}\n")
        };
        fs::write(&manifest_path, updated.as_bytes())
            .map_err(|e| format!("failed to write {}: {e}", manifest_path.display()))?;
    }

    Ok(())
}
