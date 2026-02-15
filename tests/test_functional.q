# Functional & Lambda Tests

def test_basic_lambda():
    double = |x: i64| x * 2
    assert(double(5) == 10)
    assert(double(0) == 0)
    print("test_basic_lambda passed")

def test_lambda_assignment():
    negate = |x: i64| 0 - x
    result = negate(42)
    assert(result == -42)
    print("test_lambda_assignment passed")

def test_map_with_lambda():
    nums = [1, 2, 3, 4, 5]
    doubled = nums.into_iter().map(|x: i64| x * 2).collect()
    assert(doubled == [2, 4, 6, 8, 10])
    print("test_map_with_lambda passed")

def main():
    print("=== Functional Suite ===")
    test_basic_lambda()
    test_lambda_assignment()
    test_map_with_lambda()
    print("=== Done ===")
