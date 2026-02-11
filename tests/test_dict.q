

def main():
    print("Testing Dict...")

    # Test dictionary literal creation
    scores = {"Alice": 100, "Bob": 85}
    print(scores)

    # Test subscript access (returns Option<i64> via .get(&key).copied())
    val = scores["Alice"]
    print(val)

    print("Dict tests passed!")
