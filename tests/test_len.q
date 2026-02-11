
def main():
    v = [1, 2, 3]
    print("Vector length: " + v.len().to_string())
    assert_eq(v.len(), 3)

    s = "hello"
    print("String length: " + s.len().to_string())
    assert_eq(s.len(), 5)

    print("len() tests passed!")
