def main():
    v = [1, 2, 3]
    print("Vector length:", v.len())
    assert v.len() == 3

    s = "hello"
    print("String length:", s.len())
    assert s.len() == 5

    print("len() tests passed!")
