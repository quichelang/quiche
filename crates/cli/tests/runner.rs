use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn run_integration_tests() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // crates/cli -> ../../tests
    let tests_dir = PathBuf::from(manifest_dir).join("../../tests");

    if !tests_dir.exists() {
        println!("Tests dir not found at {:?}", tests_dir);
        return;
    }

    let mut failed = false;
    let entries = fs::read_dir(tests_dir).expect("Failed to read tests dir");

    // Locate the quiche binary
    // Using cargo run logic or locating the binary in target/debug
    // Reliable way: cargo build first, then execute target/debug/quiche

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap_or_default() == "qrs" {
            let test_name = path.file_name().unwrap().to_string_lossy();
            println!("Running spec: {}", test_name);

            // Invoke quiche cli
            // We use standard cargo invocation to run the binary from the workspace
            let status = Command::new("cargo")
                .arg("run")
                .arg("-p")
                .arg("quiche_cli")
                .arg("--bin")
                .arg("quiche")
                .arg("--")
                .arg(&path)
                .status()
                .expect("Failed to execute quiche");

            if !status.success() {
                println!("FAILED: {}", test_name);
                failed = true;
            } else {
                println!("PASSED: {}", test_name);
            }
        }
    }

    if failed {
        panic!("Some integration tests failed");
    }
}
