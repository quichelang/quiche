use std::path::PathBuf;
use std::process::Command;

fn package_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn lib_resolution_runs_example_with_default_and_explicit_paths() {
    let bin = env!("CARGO_BIN_EXE_quiche");
    let root = package_root();

    let default_output = Command::new(bin)
        .current_dir(&root)
        .arg("../examples/hello.q")
        .output()
        .expect("failed to execute quiche binary");

    assert!(
        default_output.status.success(),
        "quiche run failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&default_output.stdout),
        String::from_utf8_lossy(&default_output.stderr)
    );

    let default_stdout = String::from_utf8_lossy(&default_output.stdout);
    assert!(default_stdout.contains("first: 10"));
    assert!(default_stdout.contains("rest length: 4"));

    let workspace_root = root.parent().expect("workspace root must exist");

    let example = workspace_root
        .join("examples")
        .join("hello.q")
        .canonicalize()
        .expect("example path should exist");
    let lib_path = workspace_root
        .join("lib")
        .canonicalize()
        .expect("lib path should exist");

    let explicit_output = Command::new(bin)
        .current_dir(std::env::temp_dir())
        .arg(example)
        .arg("--lib")
        .arg(lib_path)
        .output()
        .expect("failed to execute quiche binary");

    assert!(
        explicit_output.status.success(),
        "quiche run with --lib failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&explicit_output.stdout),
        String::from_utf8_lossy(&explicit_output.stderr)
    );

    let explicit_stdout = String::from_utf8_lossy(&explicit_output.stdout);
    assert!(explicit_stdout.contains("first: 10"));
    assert!(explicit_stdout.contains("last: 5"));
}