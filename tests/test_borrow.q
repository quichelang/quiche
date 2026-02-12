type Box:
    value: i64

def get_val(b: Box) -> i64:
    return b.value

def take_ownership(b: Box):
    print(f"Took ownership of: {get_val(b)}")
    b.value += 1

def main():
    b = Box(value=0)

    take_ownership(b)
    assert b.value == 1
    take_ownership(b)
    assert b.value == 2
    take_ownership(b)
    assert b.value == 3

    print("Done")
