# Str Examples
# Quiche's immutable, reference-counted string type.

def test_construction():
    # Literals automatically become Str
    greeting = "Hello"
    print(greeting)

    # str() converts any printable value
    age = str(42)
    print(age)

    pi = str(3.14)
    print(pi)

def test_equality():
    a = "hello"
    b = "hello"
    assert a == b

    c = "world"
    assert a == c == False

    # str() from numbers
    n = str(100)
    m = str(100)
    assert n == m
    print("Equality checks passed")

def test_print_formatting():
    name = "Quiche"
    version = str(7)
    print(name)
    print(version)
    print("Formatting test passed")

def main():
    print("=== Str Examples ===")
    test_construction()
    test_equality()
    test_print_formatting()
    print("=== Str Examples Complete ===")
