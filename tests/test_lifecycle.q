
def test_ownership_transfer():
    s = "Hello"
    l = s.len()
    assert_eq(l, 5)
    print("Lifecycle tests passed")

def main():
    test_ownership_transfer()
