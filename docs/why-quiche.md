# Why Quiche?

**The Python you love, with safety you can trust — without fighting the borrow checker.**

---

## The Problem

Rust is brilliant. It gives you memory safety, zero-cost abstractions, and fearless concurrency. But it asks a lot in return: lifetimes, the borrow checker, and ownership annotations everywhere.

For many programs — business logic, CLI tools, data pipelines — this is overkill. You don't need arbitrary lifetime gymnastics. You need simple, predictable rules that just work.

Python gets this right. It's readable, productive, and gets out of your way. But it's slow, and its runtime errors can bite you in production.

**What if you could have both?**

---

## The Vision

```
┌──────────────────────────────────────────────────────────┐
│  Quiche                                                   │
│  Python syntax. Compiler-managed borrowing.                │
│  Write code like Python, run it like Rust.                │
├──────────────────────────────────────────────────────────┤
│  Rust (The Foundation)                                    │
│  Everything compiles to safe, idiomatic Rust. You get     │
│  the ecosystem, the tooling, the performance. For free.   │
└──────────────────────────────────────────────────────────┘
```

---

## Auto-Borrowing

Forget `&`, `&mut`, and `where T: 'static + Send + Sync`. Quiche handles borrowing for you:

```python
def process_data(items: List[i64]) -> List[i64]:
    result = [x * 2 for x in items]
    return result
```

No borrow annotations. No lifetime parameters. The compiler inserts borrows, clones, and mutations where needed.

---

## The Pitch

> "I want to write Python, but I need it to be fast and safe."

Quiche gives you that. Python's syntax. Rust's performance. A simpler memory model that doesn't require a PhD to understand.

Write code that looks like this:

```python
def main():
    names = ["Alice", "Bob", "Carol"]
    greeting = [f"Hello {name}" for name in names]
    for g in greeting:
        print(g)
```

Get a binary that runs like Rust and cleans up after itself.

That's Quiche.
