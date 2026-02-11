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

    # Math expression (evaluated before f-string)
    doubled = age * 2
    calc = f"Double: {doubled}"
    print(calc)
