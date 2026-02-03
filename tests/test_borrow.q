
# Test for auto-cloning of function arguments (ownership-blindness)

class Box:
    value: String
    
    @immut
    def get_val(self) -> String:
        return self.value.clone()

def take_ownership(b: Box):
    println("Took ownership of: " + b.get_val())

def main():
    b = Box(value="Secret")
    
    # First use: Takes ownership. Should be auto-cloned because 'b' is used later.
    take_ownership(b)
    
    # Second use: Takes ownership. Should be auto-cloned because 'b' is used later.
    take_ownership(b)
    
    # Last use: Takes ownership. Should be moved (no clone).
    take_ownership(b)
    
    println("Done")
