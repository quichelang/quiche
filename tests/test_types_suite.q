# Types & Collections Tests

def test_lists():
    print("Running test_lists...")
    v = [1, 2, 3]
    assert_eq(v.len(), 3)

    v.push(4)
    assert_eq(v[3], 4)

def test_options():
    print("Running test_options...")
    x = Some(10)
    assert_eq(x.is_some(), True)

def test_fstrings():
    print("Running test_fstrings...")
    name = "Quiche"
    msg = f"Hello {name}"
    assert_eq(msg, "Hello Quiche")

def main():
    print("=== Types Suite ===")
    test_lists()
    test_options()
    test_fstrings()
    print("=== Done ===")
