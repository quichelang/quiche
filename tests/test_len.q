
def main():
    v = [1, 2, 3]
    print("Vector length: " + len(v).to_string())
    if len(v) != 3:
        print("FAIL: Vector length mismatch")
        exit(1)

    s = "hello"
    print("String length: " + len(s).to_string())
    if len(s) != 5:
        print("FAIL: String length mismatch")
        exit(1)

    print("len() tests passed!")
