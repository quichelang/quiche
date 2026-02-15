# qtest - Pure Quiche Test Runner
#
# Discovers and runs all tests/*.q files, reporting pass/fail status.
#
# Usage:
#   quiche lib/qtest.q              # run all tests

def is_quiche_file(f: Str) -> bool:
    return f.ends_with(".q")

def find_quiche_bin() -> Str:
    bin = System.find_executable("quiche")
    if bin != "":
        return bin
    if File.exists("target/release/quiche"):
        return "target/release/quiche"
    if File.exists("target/debug/quiche"):
        return "target/debug/quiche"
    return "quiche"

def build_test_path(test_file: Str) -> Str:
    return "tests" |> Path.join(test_file)

def run_test(path: Str) -> Tuple[Str, i64]:
    bin = find_quiche_bin()
    return System.cmd(bin, [path])

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
        path = build_test_path(f)
        output, code = run_test(path)
        if code == 0:
            passed = passed + 1
        else:
            failed = failed + 1
            print("  [FAIL]", output)

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
