
# Test for auto-cloning of function arguments (ownership-blindness)

type Box:
    value: String

def get_val(b: Box) -> String:
    return b.value

def take_ownership(b: Box):
    print("Took ownership of: " + get_val(b))

def main():
    b = Box(value="Secret")

    # First use: auto-cloned because 'b' is used later.
    take_ownership(b)

    # Second use: auto-cloned because 'b' is used later.
    take_ownership(b)

    # Last use: moved (no clone).
    take_ownership(b)

    print("Done")
