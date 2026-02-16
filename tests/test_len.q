def main():
    v = [1, 2, 3]
    len1 = v |> List.len()
    len1b = v |> len()
    len1c = v.len()
    print("Vector length:", len1)
    assert len1 == 3
    assert len1b == 3
    assert len1c == 3

    s = "hello"
    len2 = s.len()
    len2b = s |> len()
    len2c = len(s)
    print("String length:", len2)
    assert len2 == 5
    assert len2b == 5
    assert len2c == 5

    print("len() tests passed!")
