# Dict (Dictionary) Examples
# Demonstrates Quiche's chainable dictionary type.

def test_construction():
    # Dict literal
    scores = {"Alice": 95, "Bob": 87, "Carol": 92}
    assert scores.len() == 3
    print(scores)

    # Chainable builder
    config = Dict.new().set("host", "localhost").set("port", "8080")
    assert config.len() == 2

def test_access():
    scores = {"Alice": 95, "Bob": 87}

    # Direct subscript access
    val = scores["Alice"]
    print(val)

    # has() check
    assert scores.has("Alice") == True
    assert scores.has("Dave") == False

def test_mutation():
    # set() returns a new Dict (chainable)
    base = {"x": 1, "y": 2}
    extended = base.set("z", 3)
    assert extended.len() == 3
    assert extended.has("z") == True

    # remove_key() also returns new Dict
    trimmed = extended.remove_key("x")
    assert trimmed.len() == 2
    assert trimmed.has("x") == False
    print(trimmed)

def test_hashmap_methods():
    # All HashMap methods work through Deref
    scores = {"Alice": 95, "Bob": 87, "Carol": 92}
    assert scores.contains_key("Bob") == True
    assert scores.is_empty() == False
    assert scores.len() == 3

    # Iteration
    total: i64 = 0
    for entry in scores:
        total = total + entry.1
    print(total)

def main():
    print("=== Dict Examples ===")

    print("  Construction...")
    test_construction()

    print("  Access...")
    test_access()

    print("  Mutation...")
    test_mutation()

    print("  HashMap methods...")
    test_hashmap_methods()

    print("=== Dict Examples Complete ===")
