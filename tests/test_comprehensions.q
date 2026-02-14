# Test List Comprehensions and New Lambda Syntax
# These tests verify the new language features

def test_list_comp_basic():
    """Test basic list comprehension without filter"""
    nums = [1, 2, 3, 4, 5]
    doubled = [x * 2 for x in nums]
    assert doubled[0] == 2
    assert doubled[4] == 10

def test_lambda_single_line():
    """Test simple single-line lambda"""
    double = |x: i64| x * 2
    assert double(5) == 10
    assert double(3) == 6

def test_lambda_two_params():
    """Test lambda with two parameters"""
    add = |x: i64, y: i64| x + y
    assert add(2, 3) == 5
    assert add(10, 20) == 30

def main():
    test_list_comp_basic()
    test_lambda_single_line()
    test_lambda_two_params()
    print("All comprehension and lambda tests passed!")
