# Test cases for function parameter scope in match arms

type Context:
    value: i64

def new_context(val: i64) -> Context:
    return Context(value=val)

def helper_with_param(ctx: Context, choice: i64) -> i64:
    if choice == 1:
        return ctx.value + 10
    if choice == 2:
        return ctx.value * 2
    return ctx.value

def test_param_accessible_in_match():
    print("Running test_param_accessible_in_match...")
    ctx = new_context(5)

    r1 = helper_with_param(ctx, 1)
    assert_eq(r1, 15)

    r2 = helper_with_param(ctx, 2)
    assert_eq(r2, 10)

    r3 = helper_with_param(ctx, 99)
    assert_eq(r3, 5)

def recurse_with_ctx(ctx: Context, n: i64) -> i64:
    if n <= 0:
        return ctx.value
    return recurse_with_ctx(ctx, n - 1)

def test_recursive_param_passing():
    print("Running test_recursive_param_passing...")
    ctx = new_context(42)

    result = recurse_with_ctx(ctx, 5)
    assert_eq(result, 42)

def multi_param_match(a: i64, b: i64, choice: i64) -> i64:
    if choice == 1:
        return a + b
    if choice == 0:
        return a - b
    return a * b

def test_multi_params_in_match():
    print("Running test_multi_params_in_match...")

    r1 = multi_param_match(10, 3, 1)
    assert_eq(r1, 13)

    r2 = multi_param_match(10, 3, 0)
    assert_eq(r2, 7)

    r3 = multi_param_match(10, 3, 99)
    assert_eq(r3, 30)

def main():
    print("=== Parameter Scope Suite ===")
    test_param_accessible_in_match()
    test_recursive_param_passing()
    test_multi_params_in_match()
    print("=== All tests passed ===")
