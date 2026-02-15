# Dict (Dictionary) Examples
# Demonstrates Quiche's chainable dictionary type.

def test_construction():
    # Dict literal
    scores = {"Alice": 95, "Bob": 87, "Carol": 92}
    assert scores.len() == 3
    print(scores)

def main():
    print("=== Dict Examples ===")

    print("  Construction...")
    test_construction()

    print("=== Dict Examples Complete ===")
