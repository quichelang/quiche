# Example tests using Quiche test framework

def test_arithmetic():
    result = 2 + 2
    assert result == 4

    result = 10 - 3
    assert result == 7

    result = 5 * 5
    assert result == 25

def test_list_operations():
    v = [1, 2, 3]
    assert v.len() == 3

    v.push(4)
    assert v.len() == 4

    assert v[3] == 4

def test_conditionals():
    x = 10
    assert x > 5 == True
    assert x == 10

def main():
    print("=== Running Quiche Tests ===")

    print("Test: Arithmetic")
    test_arithmetic()

    print("Test: List Operations")
    test_list_operations()

    print("Test: Conditionals")
    test_conditionals()

    print("=== Tests Complete ===")
