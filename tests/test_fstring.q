# Test f-strings

def main():
    name = "World"
    age = 42
    
    # Basic f-string
    greeting = f"Hello, {name}!"
    print(greeting)
    
    # Expression in f-string
    message = f"You are {age} years old"
    print(message)
    
    # Multiple expressions
    full = f"{name} is {age}"
    print(full)
    
    # Method call in expression
    upper = f"Upper: {name.to_uppercase()}"
    print(upper)
    
    # Math expression
    calc = f"Double: {age * 2}"
    print(calc)
    
    # Escaped braces
    braces = f"Curly: {{braces}}"
    print(braces)
