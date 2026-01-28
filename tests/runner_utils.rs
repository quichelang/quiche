use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[allow(dead_code)]
pub fn compile_binary(package: &str, binary: &str) -> PathBuf {
    // Compile the binary once (quiet, warnings suppressed)
    let output = Command::new("cargo")
        .args(&["build", "-q", "-p", package, "--bin", binary])
        .env("RUSTFLAGS", "-Awarnings")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("Failed to build binary");

    if !output.status.success() {
        panic!("Failed to compile binary {}", binary);
    }

    // Located in target/debug/binary
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // Start from the crate's manifest dir (e.g. crates/cli) and go to workspace root
    // But since this file is included via #[path], CARGO_MANIFEST_DIR will be the crate running the test
    // which is e.g. crates/cli or crates/quiche-self.
    // Both are 2 levels deep from workspace root usually?
    // Let's assume standard layout: crates/<crate>/Cargo.toml
    let workspace_root = PathBuf::from(manifest_dir).join("../..");
    let binary_path = workspace_root.join("target/debug").join(binary);

    assert!(
        binary_path.exists(),
        "Binary not found at {:?}",
        binary_path
    );
    binary_path
}

fn wait_with_timeout(child: &mut Child, timeout: Duration) -> Option<ExitStatus> {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().expect("Failed to wait on child") {
            return Some(status);
        }

        if start.elapsed() > timeout {
            let _ = child.kill();
            return None;
        }

        thread::sleep(Duration::from_millis(100));
    }
}

#[derive(Debug)]
pub(crate) enum TestOutcome {
    Pass,
    Fail(String),
    Error(String),
}

#[allow(dead_code)]
pub fn run_spec_test(binary: &Path, spec_path: &Path) -> TestOutcome {
    let mut child = Command::new(binary)
        .arg(spec_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute binary");

    match wait_with_timeout(&mut child, Duration::from_secs(15)) {
        Some(status) => {
            let output = child.wait_with_output().expect("Failed to read output");
            if !status.success() {
                let _ = output;
                return TestOutcome::Fail("execution failed".to_string());
            } else {
                return TestOutcome::Pass;
            }
        }
        None => {
            return TestOutcome::Error("timeout".to_string());
        }
    }
}

#[allow(dead_code)]
pub fn run_integration_tests(package: &str, binary: &str) {
    let binary_path = compile_binary(package, binary);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // Assumes crates/<crate_name>/tests/runner.rs context
    let tests_dir = PathBuf::from(manifest_dir).join("../../tests");

    if !tests_dir.exists() {
        println!("Tests dir not found at {:?}", tests_dir);
        return;
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut errored = 0;
    let mut failures: Vec<String> = Vec::new();
    let entries = fs::read_dir(tests_dir).expect("Failed to read tests dir");

    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();

    let total = paths
        .iter()
        .filter(|p| p.extension().unwrap_or_default() == "qrs")
        .count();
    let mut index = 0;

    for path in paths {
        if path.extension().unwrap_or_default() != "qrs" {
            continue;
        }
        if path.file_name().map(|n| n == "runner.qrs").unwrap_or(false) {
            continue;
        }
        index += 1;
        let test_name = path.file_name().unwrap().to_string_lossy();
        match run_spec_test(&binary_path, &path) {
            TestOutcome::Pass => {
                passed += 1;
                println!("{}: PASS ({}/{})", test_name, index, total);
            }
            TestOutcome::Fail(detail) => {
                failed += 1;
                println!("{}: FAIL ({}/{})", test_name, index, total);
                failures.push(format!("{}: FAIL - {}", test_name, detail));
            }
            TestOutcome::Error(detail) => {
                errored += 1;
                println!("{}: ERROR ({}/{})", test_name, index, total);
                failures.push(format!("{}: ERROR - {}", test_name, detail));
            }
        }
    }

    if !failures.is_empty() {
        println!("Failures:");
        for line in failures {
            println!("{}", line);
        }
    }
    println!(
        "Summary: {} passed, {} failed, {} errored",
        passed, failed, errored
    );
    if failed > 0 || errored > 0 {
        panic!("Some integration tests failed");
    }
}

#[allow(dead_code)]
pub fn get_quiche_shim() -> &'static str {
    r#"
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
            self.expect("Quiche Exception")
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
}
"#
}

#[allow(dead_code)]
pub fn run_transpilation_test(binary: &Path, spec_path: &Path, project_root: &Path) -> TestOutcome {
    // 1. Run Transpiler
    let mut child = Command::new(binary)
        .arg(spec_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute transpiler");

    let output = match wait_with_timeout(&mut child, Duration::from_secs(15)) {
        Some(status) => {
            let out = child.wait_with_output().expect("Failed to read output");
            if !status.success() {
                let _ = out;
                return TestOutcome::Fail("transpilation failed".to_string());
            }
            out
        }
        None => {
            return TestOutcome::Error("transpilation timeout".to_string());
        }
    };

    let rust_code = String::from_utf8_lossy(&output.stdout);
    if rust_code.contains("Type Errors found") {
        return TestOutcome::Fail("type errors".to_string());
    }

    // 2. Wrap if needed
    let wrapped_user_code = if !rust_code.contains("fn main") {
        format!("fn main() {{\n{}\n}}\n", rust_code)
    } else {
        rust_code.to_string()
    };

    let mut full_code = String::new();
    full_code.push_str(
        "#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]\n",
    );
    full_code.push_str(&wrapped_user_code);

    // 3. Prepare sources
    let src_dir = project_root.join("src");
    if !src_dir.exists() {
        fs::create_dir(&src_dir).expect("Failed to create src dir");
    }

    // Resolve path to runtime relative to CWD (runner context)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // Assuming we are in crates/quiche-self/tests/runner.rs, so manifest is crates/quiche-self
    // Runtime is ../runtime.
    // Need absolute path for generated Cargo.toml to be safe
    let runtime_path = PathBuf::from(manifest_dir)
        .join("../runtime")
        .canonicalize()
        .unwrap();
    let runtime_path_str = runtime_path.to_str().unwrap().replace("\\", "/");

    let cargo_toml = format!(
        r#"
[package]
name = "test_bin"
version = "0.1.0"
edition = "2021"

[dependencies]
quiche_runtime = {{ path = "{}" }}
"#,
        runtime_path_str
    );

    fs::write(project_root.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");
    fs::write(src_dir.join("main.rs"), full_code).expect("Failed to write main.rs");

    // 4. Run `cargo run`
    // Use --quiet to avoid build output spam
    // CRITICAL: Set CARGO_TARGET_DIR to avoid locking the main workspace target dir
    // logic: project_root is a temp dir, so we can just use project_root/target
    let target_dir = project_root.join("target");

    let mut child = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .current_dir(project_root)
        .env("CARGO_TARGET_DIR", target_dir)
        .env("RUSTFLAGS", "-Awarnings")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to run cargo run");

    match wait_with_timeout(&mut child, Duration::from_secs(15)) {
        Some(status) => {
            let output = child.wait_with_output().expect("Failed to read output");
            if !status.success() {
                let _ = output;
                return TestOutcome::Fail("execution failed".to_string());
            } else {
                return TestOutcome::Pass;
            }
        }
        None => {
            return TestOutcome::Error("execution timeout".to_string());
        }
    }
}

#[allow(dead_code)]
pub fn run_self_hosted_tests(package: &str, binary: &str) {
    let binary_path = compile_binary(package, binary);
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tests_dir = PathBuf::from(manifest_dir).join("../../tests");

    if !tests_dir.exists() {
        println!("Tests dir not found at {:?}", tests_dir);
        return;
    }

    let mut passed = 0;
    let mut failed = 0;
    let mut errored = 0;
    let mut failures: Vec<String> = Vec::new();
    let entries = fs::read_dir(tests_dir).expect("Failed to read tests dir");

    // Use a single temp dir for all tests to reuse build cache
    let temp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");
    let proj_root = temp_dir.path();

    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();

    let total = paths
        .iter()
        .filter(|p| p.extension().unwrap_or_default() == "qrs")
        .count();
    let mut index = 0;

    for path in paths {
        if path.extension().unwrap_or_default() != "qrs" {
            continue;
        }
        if path.file_name().map(|n| n == "runner.qrs").unwrap_or(false) {
            continue;
        }
        index += 1;
        let test_name = path.file_name().unwrap().to_string_lossy();
        match run_transpilation_test(&binary_path, &path, proj_root) {
            TestOutcome::Pass => {
                passed += 1;
                println!("{}: PASS ({}/{})", test_name, index, total);
            }
            TestOutcome::Fail(detail) => {
                failed += 1;
                println!("{}: FAIL ({}/{})", test_name, index, total);
                failures.push(format!("{}: FAIL - {}", test_name, detail));
            }
            TestOutcome::Error(detail) => {
                errored += 1;
                println!("{}: ERROR ({}/{})", test_name, index, total);
                failures.push(format!("{}: ERROR - {}", test_name, detail));
            }
        }
    }

    if !failures.is_empty() {
        println!("Failures:");
        for line in failures {
            println!("{}", line);
        }
    }
    println!(
        "Summary: {} passed, {} failed, {} errored",
        passed, failed, errored
    );
    if failed > 0 || errored > 0 {
        panic!("Some integration tests failed");
    }
}
