# qtest - Pure Quiche Test Runner
#
# A lightweight test framework for Quiche. Tests are regular functions
# that return TestResult. The runner collects results and prints a summary.
#
# Usage:
#   quiche lib/qtest.q              # run self-tests
#   quiche test my_tests.q          # run user tests (future)

# ── Types ──────────────────────────────────────────────────────────────────

type TestResult = | Passed | Failed(String) | Skipped(String)

type TestSummary:
    total: usize
    passed: usize
    failed: usize
    skipped: usize
    failures: Vec[String]

# ── Constructors ───────────────────────────────────────────────────────────

def passed() -> TestResult:
    return TestResult.Passed

def failed(reason: String) -> TestResult:
    return TestResult.Failed(reason)

def skipped(reason: String) -> TestResult:
    return TestResult.Skipped(reason)

def new_summary() -> TestSummary:
    return TestSummary(
        total=0,
        passed=0,
        failed=0,
        skipped=0,
        failures=Vec.new()
    )

# ── Recording ──────────────────────────────────────────────────────────────

def record(summary: TestSummary, name: String, result: TestResult):
    summary.total = summary.total + 1
    match result:
        case TestResult.Passed:
            summary.passed = summary.passed + 1
            print(f"  [PASS] {name}")
        case TestResult.Failed(reason):
            summary.failed = summary.failed + 1
            summary.failures.push(f"{name}: {reason}")
            print(f"  [FAIL] {name}: {reason}")
        case TestResult.Skipped(reason):
            summary.skipped = summary.skipped + 1
            print(f"  [SKIP] {name}: {reason}")

def print_summary(summary: TestSummary):
    print("")
    print("========================================")
    if summary.failed == 0:
        print(f"All {summary.passed} tests passed")
    else:
        print(f"{summary.passed} passed, {summary.failed} failed")
        for failure in summary.failures:
            print(f"  ✗ {failure}")
    print("========================================")

# ── Assertions ─────────────────────────────────────────────────────────────

def assert_eq[T](actual: T, expected: T, msg: String) -> TestResult:
    if actual == expected:
        return passed()
    return failed(msg)

def assert_true(condition: bool, msg: String) -> TestResult:
    if condition:
        return passed()
    return failed(msg)

# ── Self-Tests ─────────────────────────────────────────────────────────────

def test_passed_works() -> TestResult:
    match passed():
        case TestResult.Passed:
            return passed()
        case _:
            return failed("passed() did not return Passed")

def test_failed_works() -> TestResult:
    match failed("test reason"):
        case TestResult.Failed(msg):
            if msg == "test reason":
                return passed()
            return failed("wrong message")
        case _:
            return failed("failed() did not return Failed")

def test_basic_math() -> TestResult:
    if 1 + 1 != 2:
        return failed("1 + 1 should equal 2")
    if 5 * 3 != 15:
        return failed("5 * 3 should equal 15")
    return passed()

def test_string_len() -> TestResult:
    s: String = "hello".to_string()
    if s.len() != 5:
        return failed("string length should be 5")
    return passed()

# ── Runner ─────────────────────────────────────────────────────────────────

def main():
    print("")
    print("========================================")
    print("           qtest runner")
    print("========================================")
    print("")

    summary = new_summary()

    print("Running self-tests...")
    record(summary, "test_passed_works", test_passed_works())
    record(summary, "test_failed_works", test_failed_works())
    record(summary, "test_basic_math", test_basic_math())
    record(summary, "test_string_len", test_string_len())

    print_summary(summary)

    if summary.failed > 0:
        rust():
            std::process::exit(1);
