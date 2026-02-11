# Control Flow & Arithmetic Tests

def test_arithmetic():
    print("Running test_arithmetic...")
    x = 10
    y = 5
    assert_eq(x + y, 15)
    assert_eq(x - y, 5)
    assert_eq(x * y, 50)
    assert_eq(x / y, 2)

def test_factorial():
    print("Running test_factorial...")
    res = 1
    i = 1
    while i <= 5:
        res = res * i
        i = i + 1
    assert_eq(res, 120)

def test_loops():
    print("Running test_loops...")
    # While loop
    sum = 0
    k = 0
    while k < 10:
        k = k + 1
        sum = sum + k
    assert_eq(sum, 55)

    # For loop (Iterating Vec)
    items = [10, 20, 30]
    total = 0
    for item in items:
        total = total + item
    assert_eq(total, 60)

def test_ternary():
    print("Running test_ternary...")
    a = 5
    b = 10
    # Ternary not supported in .q, use if/else
    max_val = 0
    if a > b:
        max_val = a
    else:
        max_val = b
    assert_eq(max_val, 10)

    min_val = 0
    if a < b:
        min_val = a
    else:
        min_val = b
    assert_eq(min_val, 5)

def main():
    print("=== Control Flow Suite ===")
    test_arithmetic()
    test_factorial()
    test_loops()
    test_ternary()
    print("=== Done ===")
