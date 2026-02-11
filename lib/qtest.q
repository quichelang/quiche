# qtest - Pure Quiche Test Runner
#
# Discovers and runs all tests/*.q files, reporting pass/fail status.
#
# Usage:
#   quiche lib/qtest.q              # run all tests

# ── Self-Tests ─────────────────────────────────────────────────────────────

def run_self_tests() -> i64:
    fails = 0

    # test_basic_math
    if 1 + 1 == 2:
        print("  [PASS] basic_math")
    else:
        print("  [FAIL] basic_math")
        fails = fails + 1

    # test_string_len
    s: String = "hello".to_string()
    if s.len() == 5:
        print("  [PASS] string_len")
    else:
        print("  [FAIL] string_len")
        fails = fails + 1

    # test_vec_ops
    v = [1, 2, 3]
    v.push(4)
    if v.len() == 4:
        print("  [PASS] vec_ops")
    else:
        print("  [FAIL] vec_ops")
        fails = fails + 1

    return fails

# ── Main ───────────────────────────────────────────────────────────────────

def main():
    print("")
    print("========================================")
    print("           qtest runner")
    print("========================================")
    print("")

    print("Self-tests:")
    self_fails = run_self_tests()

    print("")
    print("Test files:")
    rust("""
        use std::process::Command;
        use std::path::Path;

        let mut test_files: Vec<String> = Vec::new();
        let test_dir = Path::new("tests");
        if test_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(test_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "q" {
                            if let Some(name) = path.file_name() {
                                test_files.push(name.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        }
        test_files.sort();

        let quiche_bin = {
            // Prefer the installed version on PATH (always up-to-date)
            let which = Command::new("which").arg("quiche").output();
            if let Ok(out) = &which {
                if out.status.success() {
                    String::from_utf8_lossy(&out.stdout).trim().to_string()
                } else if Path::new("target/release/quiche").exists() {
                    "target/release/quiche".to_string()
                } else if Path::new("target/debug/quiche").exists() {
                    "target/debug/quiche".to_string()
                } else {
                    "quiche".to_string()
                }
            } else {
                "quiche".to_string()
            }
        };

        let mut passed: usize = 0;
        let mut failed: usize = 0;
        let mut failures: Vec<String> = Vec::new();

        for file in &test_files {
            let path = format!("tests/{file}");
            let name = file.trim_end_matches(".q");
            let output = Command::new(quiche_bin.as_str()).arg(&path).output();
            match output {
                Ok(out) if out.status.success() => {
                    passed += 1;
                    println!("  [PASS] {name}");
                }
                Ok(out) => {
                    failed += 1;
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    // Collect error lines from both streams
                    let mut err_lines: Vec<String> = Vec::new();
                    for line in stderr.lines().chain(stdout.lines()) {
                        let lt = line.trim();
                        if lt.contains("error") || lt.contains("Error")
                            || lt.contains("Compile") || lt.contains("Parse")
                            || lt.contains("Capability") || lt.contains("panicked")
                            || lt.starts_with("-->") || lt.starts_with("|") {
                            if err_lines.len() < 5 {
                                err_lines.push(lt.to_string());
                            }
                        }
                    }
                    // Build summary from collected lines
                    let mut summary = String::new();
                    for (i, line) in err_lines.iter().enumerate() {
                        if i > 0 { summary.push_str("\n    "); }
                        summary.push_str(line);
                    }
                    if summary.is_empty() { summary = "unknown error".to_string(); }
                    let first_line = err_lines.first()
                        .map(|s| s.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    failures.push(format!("{name}: {first_line}"));
                    println!("  [FAIL] {name}:\n    {summary}");
                }
                Err(e) => {
                    failed += 1;
                    failures.push(format!("{name}: spawn failed: {e}"));
                    println!("  [FAIL] {name}: spawn failed: {e}");
                }
            }
        }

        println!();
        println!("========================================");
        let self_fails = self_fails as usize;
        let total_failed = failed + self_fails;
        if total_failed == 0 {
            let total = passed + 3; // 3 self-tests
            println!("  All {total} tests passed");
        } else {
            let total_passed = passed + 3 - self_fails;
            println!("  {total_passed} passed, {total_failed} FAILED");
            println!();
            for f in &failures {
                println!("    {f}");
            }
        }
        println!("========================================");

        if total_failed > 0 {
            std::process::exit(1);
        }
    """)
