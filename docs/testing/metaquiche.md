# Testing in MetaQuiche

MetaQuiche uses a simple result-based testing pattern. Tests are functions that return `TestResult`.

## Quick Start

```python
from qtest import TestResult, passed, failed, check_eq_i32

def test_addition() -> TestResult:
    if 1 + 1 != 2:
        return failed("addition broken")
    return passed()

def test_with_check() -> TestResult:
    return check_eq_i32(40 + 2, 42, "should equal 42")
```

## API Reference

### Result Constructors

| Function | Description |
|:---------|:------------|
| `passed()` | Return passing result |
| `failed(msg)` | Return failing result with reason |
| `skipped(msg)` | Return skipped result with reason |

### Check Functions

| Function | Description |
|:---------|:------------|
| `check_true(cond, msg)` | Check condition is true |
| `check_false(cond, msg)` | Check condition is false |
| `check_eq_i32(a, b, msg)` | Check i32 equality |
| `check_eq_usize(a, b, msg)` | Check usize equality |

### Test Summary

```python
from qtest import create_test_summary, record_result, print_summary

summary = create_test_summary()
record_result(mutref(summary), "test_addition", test_addition())
record_result(mutref(summary), "test_with_check", test_with_check())
print_summary(ref(summary))
```

## Running Tests

```bash
# Run via qtest runner
quiche test

# Or filter by pattern
quiche test test_math
```
