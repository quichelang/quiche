# qtest - Pure Quiche Test Runner
#
# Discovers and runs all tests/*.q files, reporting pass/fail status.
#
# Usage:
#   quiche lib/qtest.q              # run all tests

def is_quiche_file(f: Str) -> bool:
    return f.ends_with(".q")

def find_quiche_bin() -> Str:
    # Prefer local builds (test current code, not stale install)
    if File.exists("target/debug/quiche"):
        return "target/debug/quiche"
    if File.exists("target/release/quiche"):
        return "target/release/quiche"
    # Fall back to installed
    bin = System.find_executable("quiche")
    if bin != "":
        return bin
    return "quiche"

def run_test(test_file: Str) -> Tuple[Str, Str, i64]:
    bin = find_quiche_bin()
    path = "tests" |> Path.join(test_file)
    output, code = System.cmd(bin, [path])
    return (test_file, output, code)

def main():
    print("")
    print("========================================")
    print("           qtest runner")
    print("========================================")
    print("")

    all_files = File.ls("tests")
    test_files = all_files |> Enum.filter(is_quiche_file) |> Enum.sort()

    print("Using:", find_quiche_bin())
    print("")

    passed: i64 = 0
    failed: i64 = 0

    for f in test_files:
        name, output, code = run_test(f)
        if code == 0:
            passed = passed + 1
            print("  [PASS]", name)
        else:
            failed = failed + 1
            print("  [FAIL]", name)
            print("   ", output)

    print("")
    print("========================================")
    total = passed + failed
    if failed == 0:
        print(" ", total, "tests passed")
    else:
        print(" ", passed, "passed,", failed, "FAILED")
    print("========================================")

    if failed > 0:
        System.halt(1)
