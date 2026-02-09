# Test: struct definitions with methods
# Verifies:
#   1. Static method calls: Type.method() → Type::method()
#   2. Instance method calls: obj.method() → obj.method()
#   3. Struct field declarations
#   4. Multiple methods per struct
#   5. Impl blocks are properly emitted (not dropped)

class Counter(Struct):
    value: i64

    def new(v: i64) -> Counter:
        return Counter(v)

    def get(self) -> i64:
        return self.value

    def add(self, n: i64) -> i64:
        return self.value + n

def main():
    c: Counter = Counter.new(10)
    print("value:", c.get())
    print("add 5:", c.add(5))
