
# Test for auto-cloning of function arguments (ownership-blindness)

class Box:
    value: String
    
    def get(self) -> String:
        return self.value

def take_ownership(b: Box):
    println("Took ownership of: " + b.get())

def main():
    b = Box(value="Secret")
    
    # First use: Takes ownership. Should be auto-cloned because 'b' is used later.
    take_ownership(b)
    
    # Second use: Takes ownership. Should be auto-cloned because 'b' is used later.
    take_ownership(b)
    
    # Last use: Takes ownership. Should be moved (no clone).
    take_ownership(b)
    
    println("Done")
