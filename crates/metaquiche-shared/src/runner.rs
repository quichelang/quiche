use crate::error_exit::UnwrapOrExit;
use crate::template::{get_and_render, templates};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_cargo_command(cmd: String, args: Vec<String>) -> i32 {
    let status = Command::new("cargo")
        .arg(cmd)
        .args(args.iter())
        .status()
        .unwrap_or_exit()
        .with_error("Failed to run cargo");
    if status.success() {
        0
    } else {
        status.code().unwrap_or(1)
    }
}

pub fn run_rust_code(
    user_code: String,
    script_args: Vec<String>,
    quiet: bool,
    suppress_output: bool,
    raw_output: bool,
    warn: bool,
    strict: bool,
    release: bool,
) -> i32 {
    let rust_code = user_code.replace("#[test]", "");

    let quiche_module = templates().get_content("runtime.quiche_module_run");

    let wrapped_user_code = if !rust_code.contains("fn main") {
        format!("fn main() {{\n{}\n}}\n", rust_code)
    } else {
        rust_code
    };

    let full_code = get_and_render(
        "runtime.run_wrapper",
        &[
            ("quiche_module", quiche_module),
            ("user_code", &wrapped_user_code),
        ],
    );

    if !Path::new("target").exists() {
        std::fs::create_dir("target").ok();
    }
    let tmp_rs = "target/tmp.rs";
    std::fs::write(tmp_rs, full_code)
        .unwrap_or_exit()
        .with_error("Failed to write temp Rust file");

    if !quiet {
        // Removed verbose "Compiling and Running" message for cleaner output
    }
    let mut rustc = Command::new("rustc");
    rustc
        .arg(tmp_rs)
        .arg("--edition")
        .arg("2024")
        .arg("-o")
        .arg("target/tmp_bin");

    if release {
        rustc.arg("-Copt-level=3");
    }

    if strict {
        rustc.arg("-D").arg("warnings");
    }
    if quiet && !warn && !strict {
        rustc
            .arg("-Awarnings")
            .stdout(Stdio::null())
            .stderr(Stdio::null());
    }

    let status = rustc
        .status()
        .unwrap_or_exit()
        .with_error("Failed to run rustc");
    if !status.success() {
        return status.code().unwrap_or(1);
    }

    if suppress_output {
        let status = Command::new("./target/tmp_bin")
            .args(script_args.iter())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap_or_exit()
            .with_error("Failed to run binary");
        if !status.success() {
            return status.code().unwrap_or(1);
        }
        return 0;
    }

    let output = Command::new("./target/tmp_bin")
        .args(script_args.iter())
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
        return output.status.code().unwrap_or(1);
    }
    0
}
