// Legacy host compiler CLI - will be deprecated in favor of metaquiche-native

use metaquiche_host::compile;
use metaquiche_shared::error_exit::UnwrapOrExit;
use metaquiche_shared::i18n::{tr, tr1};
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use metaquiche_shared::template as templates;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", tr("cli.error.no_command"));
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "new" => {
            if args.len() < 3 {
                eprintln!("{}", tr("cli.error.missing_project_name"));
                println!("{}", tr("cli.usage.new"));
                std::process::exit(1);
            }
            if args[2] == "--lib" {
                if args.len() < 4 {
                    eprintln!("{}", tr("cli.error.missing_project_name"));
                    println!("{}", tr("cli.usage.new_lib"));
                    std::process::exit(1);
                }
                create_new_project(&args[3], true);
            } else {
                create_new_project(&args[2], false);
            }
        }
        "build" => {
            let (_warn, _strict, _warn_all, _warn_quiche, _emit_rust, release, rest) =
                parse_flags(&args[2..]);
            let mut cmd_args = rest;
            if release {
                cmd_args.push("--release".to_string());
            }
            run_cargo_command("build", &cmd_args);
        }
        "run" => {
            let (_warn, _strict, _warn_all, _warn_quiche, _emit_rust, release, rest) =
                parse_flags(&args[2..]);
            if Path::new("Cargo.toml").exists() {
                let mut cmd_args = rest;
                if release {
                    cmd_args.push("--release".to_string());
                }
                run_cargo_command("run", &cmd_args);
            } else {
                eprintln!("{}", tr("cli.error.no_cargo_toml"));
                eprintln!("{}", tr("cli.hint.single_script"));
                std::process::exit(1);
            }
        }
        "test" => {
            let (warn, strict, warn_all, warn_quiche, emit_rust, release, rest) =
                parse_flags(&args[2..]);
            if Path::new("tests/runner.qrs").exists() {
                if let Ok(exe) = env::current_exe() {
                    if let Some(exe_str) = exe.to_str() {
                        env::set_var("QUICHE_TEST_BIN", exe_str);
                    }
                }
                if warn_all {
                    env::set_var("QUICHE_WARN_ALL", "1");
                }
                if warn_quiche {
                    env::set_var("QUICHE_WARN_QUICHE", "1");
                }
                run_single_file_with_options(
                    "tests/runner.qrs",
                    &rest,
                    true,
                    false,
                    true,
                    warn,
                    strict,
                    emit_rust,
                    release,
                );
            } else if Path::new("Cargo.toml").exists() {
                let mut cmd_args = rest;
                if release {
                    cmd_args.push("--release".to_string());
                }
                run_cargo_command("test", &cmd_args);
            } else {
                eprintln!("{}", tr("cli.error.no_tests_found"));
                std::process::exit(1);
            }
        }
        arg => {
            if arg.ends_with(".qrs") {
                if !Path::new(arg).exists() {
                    eprintln!("{}", tr1("cli.error.file_not_found", "file", arg));
                    std::process::exit(1);
                }
                let (warn, strict, warn_all, warn_quiche, emit_rust, release, rest) =
                    parse_flags(&args[2..]);
                if warn_all {
                    env::set_var("QUICHE_WARN_ALL", "1");
                }
                if warn_quiche {
                    env::set_var("QUICHE_WARN_QUICHE", "1");
                }
                run_single_file_with_options(
                    arg, &rest, false, false, false, warn, strict, emit_rust, release,
                );
            } else {
                eprintln!("{}", tr1("cli.error.unrecognized_command", "cmd", arg));
                if Path::new(arg).exists() {
                    eprintln!("{}", tr("cli.note.qrs_extension"));
                } else {
                    // Simple suggestions
                    let cmds = ["new", "build", "run", "test"];
                    for cmd in cmds {
                        // Check for common typo (1 char off) - manually or just prefix
                        if cmd.starts_with(arg) || arg.starts_with(cmd) {
                            eprintln!("{}", tr1("cli.hint.did_you_mean", "suggestion", cmd));
                            break;
                        }
                    }
                }
                println!();
                print_usage();
                std::process::exit(1);
            }
        }
    }
}

fn print_usage() {
    println!("{}", tr("cli.usage.header"));
    println!("{}", tr("cli.usage.new"));
    println!("{}", tr("cli.usage.build"));
    println!("{}", tr("cli.usage.run"));
    println!("{}", tr("cli.usage.test"));
    println!("{}", tr("cli.usage.script"));
    println!();
    println!("Flags:");
    println!("  --warn               Show compiler warnings");
    println!("  --strict             Treat warnings as errors");
    println!("  --warn-all           Show all warnings (Quiche + Rust)");
    println!("  --warn-quiche        Show only Quiche warnings");
    println!("  -m, --emit-rust      Emit generated Rust code instead of running");
    println!("  -r, --release        Build in release mode (optimized)");
}

fn create_new_project(name: &str, is_lib: bool) {
    let path = Path::new(name);
    if path.exists() {
        println!("{}", tr1("cli.error.dir_exists", "name", name));
        return;
    }

    fs::create_dir_all(path.join("src"))
        .unwrap_or_exit()
        .with_error("Failed to create src dir");

    fs::write(
        path.join("Quiche.toml"),
        templates::get_and_render("quiche_toml", &[("name", name)]),
    )
    .unwrap_or_exit()
    .with_error("Failed to write Quiche.toml");
    // Determine path to compiler crate (relative to CLI crate which is compiled)
    // CARGO_MANIFEST_DIR points to crates/cli
    let compiler_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_exit()
        .with_error("No parent directory")
        .join("compiler")
        .canonicalize()
        .unwrap_or_else(|_| Path::new("../crates/compiler").to_path_buf());

    let compiler_path_str = compiler_path.to_str().unwrap_or("").replace("\\", "/");
    // Escape backslashes for Windows path in string literal if needed, but Cargo handles / fine.

    // Generate Cargo.toml
    let mut cargo_toml = templates::get_and_render(
        "cargo_toml",
        &[("name", name), ("compiler_path", &compiler_path_str)],
    );
    if is_lib {
        cargo_toml.push_str(templates::templates().get_content("cargo_toml_lib_section"));
    } else {
        cargo_toml.push_str(&templates::get_and_render(
            "cargo_toml_bin_section",
            &[("name", name)],
        ));
    }

    fs::write(path.join("Cargo.toml"), cargo_toml)
        .unwrap_or_exit()
        .with_error("Failed to write Cargo.toml");
    fs::write(
        path.join("build.rs"),
        templates::templates().get_content("build_rs"),
    )
    .unwrap_or_exit()
    .with_error("Failed to write build.rs");

    let quiche_module = templates::templates().get_content("quiche_module");

    if is_lib {
        fs::write(
            path.join("src/lib.qrs"),
            templates::templates().get_content("lib_qrs"),
        )
        .unwrap_or_exit()
        .with_error("Failed to write lib.qrs");
        fs::write(
            path.join("src/lib.rs"),
            templates::get_and_render("lib_rs_wrapper", &[("quiche_module", quiche_module)]),
        )
        .unwrap_or_exit()
        .with_error("Failed to write lib.rs");
    } else {
        fs::write(
            path.join("src/main.qrs"),
            templates::templates().get_content("main_qrs"),
        )
        .unwrap_or_exit()
        .with_error("Failed to write main.qrs");
        fs::write(
            path.join("src/main.rs"),
            templates::get_and_render("main_rs_wrapper", &[("quiche_module", quiche_module)]),
        )
        .unwrap_or_exit()
        .with_error("Failed to write main.rs");
    }

    println!("{}", tr1("cli.success.project_created", "name", name));
}

fn run_cargo_command(cmd: &str, args: &[String]) {
    let status = Command::new("cargo")
        .arg(cmd)
        .args(args)
        .status()
        .unwrap_or_exit()
        .with_error("Failed to run cargo");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_single_file_with_options(
    filename: &str,
    script_args: &[String],
    quiet: bool,
    suppress_output: bool,
    raw_output: bool,
    warn: bool,
    strict: bool,
    emit_rust: bool,
    release: bool,
) {
    let source_raw = match fs::read_to_string(filename) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to read file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };
    let source = source_raw;

    // Virtual Module System (Poor Man's Linker)
    // Removed in favor of native macros and Cargo dependencies.
    let dependencies = String::new();

    if let Some(rust_code) = compile(&source, filename) {
        if emit_rust {
            print!("{}", rust_code);
            return;
        }

        let rust_code: String = rust_code.replace("#[test]", "");

        let quiche_module = r#"
mod quiche {
    #![allow(unused_macros, unused_imports)]
    
    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }
    
    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Error")
        }
    }
    

    
    // Low Priority: Takes &Self (Clone fallback)
    pub trait QuicheGeneric {
        fn quiche_handle(&self) -> Self;
    }
    
    impl<T: Clone> QuicheGeneric for T {
        fn quiche_handle(&self) -> Self {
            self.clone()
        }
    }
    
    macro_rules! check {
        ($val:expr) => {
            {
                use crate::quiche::{QuicheResult, QuicheGeneric};
                ($val).quiche_handle()
            }
        };
    }
    pub(crate) use check;
    pub(crate) use check as call;

    macro_rules! qref {
        ($e:expr) => { &($e) };
    }
    pub(crate) use qref;

    macro_rules! mutref {
        ($e:expr) => { &mut ($e) };
    }
    pub(crate) use mutref;

    macro_rules! deref {
        ($e:expr) => { *($e) };
    }
    pub(crate) use deref;

    /// String concatenation macro - efficient push_str pattern
    macro_rules! strcat {
        // Single argument - just convert to String
        ($arg:expr) => {
            ($arg).to_string()
        };
        // Multiple arguments - use push_str pattern
        ($first:expr, $($rest:expr),+ $(,)?) => {{
            let mut __s = ($first).to_string();
            $(
                __s.push_str(&($rest).to_string());
            )+
            __s
        }};
    }
    pub(crate) use strcat;

    pub fn run_test_cmd(exe: String, test_path: String) -> bool {
        let mut cmd = std::process::Command::new(exe);
        cmd.arg(test_path);
        cmd.env("QUICHE_QUIET", "1");
        cmd.env("QUICHE_SUPPRESS_OUTPUT", "1");
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        match cmd.status() {
            Ok(status) => status.success(),
            Err(_) => false,
        }
    }

    pub fn list_test_files() -> Vec<String> {
        let mut tests = Vec::new();
        if let Ok(entries) = std::fs::read_dir("tests") {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.ends_with(".qrs") || name == "runner.qrs" {
                    continue;
                }
                tests.push(name);
            }
        }
        tests.sort();
        tests
    }
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
        fs::write(tmp_rs, full_code)
            .unwrap_or_exit()
            .with_error("Failed to write temp Rust file");

        if !quiet {
            println!("{}", tr("cli.info.compiling"));
        }
        let mut rustc = Command::new("rustc");
        rustc
            .arg(tmp_rs)
            .arg("--edition")
            .arg("2024")
            .arg("-o")
            .arg("target/tmp_bin");

        if strict {
            rustc.arg("-D").arg("warnings");
        }
        if quiet && !warn && !strict {
            rustc
                .arg("-Awarnings")
                .stdout(Stdio::null())
                .stderr(Stdio::null());
        }

        if release {
            rustc.arg("-Copt-level=3");
        }

        let status = rustc
            .status()
            .unwrap_or_exit()
            .with_error("Failed to run rustc");

        if !status.success() {
            println!("{}", tr1("cli.error.compilation_failed", "error", filename));
            std::process::exit(status.code().unwrap_or(1));
        }

        if suppress_output {
            let status = Command::new("./target/tmp_bin")
                .args(script_args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap_or_exit()
                .with_error("Failed to run binary");
            if !status.success() {
                std::process::exit(status.code().unwrap_or(1));
            }
            return;
        }

        let output = Command::new("./target/tmp_bin")
            .args(script_args)
            .output()
            .unwrap_or_exit()
            .with_error("Failed to run binary");

        if raw_output {
            print!("{}", String::from_utf8_lossy(&output.stdout));
        } else {
            println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
            if !output.stderr.is_empty() {
                println!("Errors:\n{}", String::from_utf8_lossy(&output.stderr));
            }
        }

        if !output.status.success() {
            std::process::exit(output.status.code().unwrap_or(1));
        }
    } else {
        std::process::exit(1);
    }
}

fn parse_flags(args: &[String]) -> (bool, bool, bool, bool, bool, bool, Vec<String>) {
    let mut warn = false;
    let mut strict = false;
    let mut warn_all = false;
    let mut warn_quiche = false;
    let mut emit_rust = false;
    let mut release = false;
    let mut rest = Vec::new();
    for a in args {
        match a.as_str() {
            "--warn" => warn = true,
            "--strict" => strict = true,
            "--warn-all" => warn_all = true,
            "--warn-quiche" => warn_quiche = true,
            "--emit-rust" | "-m" => emit_rust = true,
            "--release" | "-r" => release = true,
            _ => rest.push(a.clone()),
        }
    }
    if warn {
        warn_all = true;
    }
    if warn_all {
        warn = true;
    }
    (
        warn,
        strict,
        warn_all,
        warn_quiche,
        emit_rust,
        release,
        rest,
    )
}
