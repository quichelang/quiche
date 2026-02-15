
def test_ownership_transfer():
    s: str = "Hello"
    l = s.len()
    assert l == 5
    print("Lifecycle tests passed")

def main():
    test_ownership_transfer()
