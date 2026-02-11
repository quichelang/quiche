# Structs & Matching Tests

type Shape = Circle(i64) | Rectangle(i64, i64) | Point

type User:
    id: i64
    name: String

def test_enums():
    print("Running test_enums...")
    s = Shape.Circle(10)

    match s:
        case Shape.Circle(r):
            print("PASS: enum match")
        case _:
            print("FAIL: Match enum")

def test_structs():
    print("Running test_structs...")
    u = User(id=1, name="Alice")
    assert_eq(u.id, 1)
    assert_eq(u.name, "Alice")

def main():
    print("=== Struct Suite ===")
    test_enums()
    test_structs()
    print("=== Done ===")
