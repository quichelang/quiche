use std::process::Command;
use tempfile::TempDir;

#[path = "../../../tests/runner_utils.rs"]
mod runner_utils;

#[test]
fn integration_tests() {
    runner_utils::run_integration_tests("quiche_cli", "quiche");
}

#[test]
fn test_project_lifecycle() {
    let binary_path = runner_utils::compile_binary("quiche_cli", "quiche");

    // 1. Create a temp directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let root = temp_dir.path();

    // 2. Run `quiche new test_proj`
    let status = Command::new(&binary_path)
        .arg("new")
        .arg("test_proj")
        .current_dir(root)
        .status()
        .expect("Failed to run quiche new");

    assert!(status.success(), "quiche new failed");

    let proj_dir = root.join("test_proj");
    assert!(proj_dir.exists());
    assert!(proj_dir.join("Cargo.toml").exists());
    assert!(proj_dir.join("src/main.qrs").exists());

    // 3. Run `quiche run` inside the project
    // This tests the delegation to cargo
    let output = Command::new(&binary_path)
        .arg("run")
        .current_dir(&proj_dir)
        .output()
        .expect("Failed to run quiche run");

    assert!(
        output.status.success(),
        "quiche run failed:\nStdout: {}\nStderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Hello, Quiche!"),
        "Output did not contain 'Hello, Quiche!', got:\n{}",
        stdout
    );
}
