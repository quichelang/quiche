# Why Quiche?

**The Python you love, with safety you can trust—without fighting the borrow checker.**

---

## The Problem

Rust is brilliant. It gives you memory safety, zero-cost abstractions, and fearless concurrency. But it asks a lot in return: you must think about lifetimes, wrestle with the borrow checker, and annotate ownership everywhere.

For many programs—business logic, CLI tools, data pipelines—this is overkill. You don't need arbitrary lifetime gymnastics. You need simple, predictable rules that just work.

Python gets this right. It's readable, productive, and gets out of your way. But it's slow, and its runtime errors can bite you in production.

**What if you could have both?**

---

## The Vision

Quiche is a **stratified language** that gives you the best of both worlds:

```
┌────────────────────────────────────────────────────────────────┐
│  Quiche (Pure)                                                 │
│  Python syntax. Compiler-managed borrowing. No panic traps.    │
│  Write code like Python, run it like Rust.                     │
├────────────────────────────────────────────────────────────────┤
│  MetaQuiche (The Escape Hatch)                                 │
│  When you need full control—explicit refs, Rust interop,       │
│  performance-critical inner loops—drop down one layer.         │
├────────────────────────────────────────────────────────────────┤
│  Rust (The Foundation)                                         │
│  Everything compiles to safe, idiomatic Rust. You get the      │
│  ecosystem, the tooling, the performance. For free.            │
└────────────────────────────────────────────────────────────────┘
```

---

## Simplified Lifetimes

Forget `'a`, `'b`, and `where T: 'static + Send + Sync`. In Quiche, there are only a few lifetime concepts you need:

| Annotation | Meaning | When to use |
|------------|---------|-------------|
| `static` | Lives forever | Configuration, singletons |
| `current` | Lives until this function returns | Local computation |
| `owned` | Caller takes ownership | Return values, transfers |
| `borrowed` | Temporary view | Reading without copying |

The compiler infers these for you. Most of the time, you write nothing at all:

```python
def process_file(path: str) -> Result[Data, Error]:
    contents = read_file(path)?       # Borrowed automatically
    parsed = parse(contents)?         # Compiler knows this escapes
    return Ok(parsed)                 # Only `parsed` survives
    # Everything else is cleaned up—guaranteed
```

No lifetime annotations. No borrow checker errors. The compiler does the work.

---

## No Panics, No Surprises

Quiche doesn't panic. Ever.

Every fallible operation returns a `Result`. There's no hidden `unwrap()`, no surprise stack unwinding, no "thread 'main' panicked at..." in production.

This unlocks something powerful: **deterministic cleanup**.

When a function returns—successfully or with an error—the compiler knows exactly what to free. No runtime costs for exception handling. No leaked resources. No half-initialized state.

```python
def risky_operation() -> Result[Output, Error]:
    resource = acquire()?         # If this fails, nothing to clean up
    result = process(resource)?   # If this fails, `resource` is released
    return Ok(result)             # Success—`resource` cleaned up on exit
```

The compiler generates the cleanup code. You write the happy path.

---

## Two Languages, One Codebase

**Quiche** is for application developers. It's approachable, safe, and productive. You don't need to understand borrowing, lifetimes, or ownership. The compiler handles it.

**MetaQuiche** is for systems developers. It's the layer that implements Quiche itself—the parser, the compiler, the runtime. It has access to raw Rust semantics when needed. It's deliberately "impure" so that Quiche can stay pure.

This separation is powerful:

- **Library authors** can use MetaQuiche to write high-performance primitives
- **Application developers** use Quiche and never touch unsafe code
- **The compiler** is self-hosting—MetaQuiche compiles itself

---

## What We're Building

Quiche isn't a toy. It's a language that can compile itself.

- **Self-hosting compiler** written in MetaQuiche
- **Zero external dependencies** for parsing (no 100-crate supply chain)
- **Full Python syntax** support where it makes sense
- **Seamless Rust interop** when you need the ecosystem

The goal: a language that's **safer than Rust for everyday code**, backed by Rust's guarantees, with Python's ergonomics.

---

## Open Questions (We're Figuring These Out)

1. **Cycles**: Should Quiche forbid cyclic data structures in pure mode? (Many programs don't need them.)

2. **Closures**: How do closures capture `current` values? What happens when they outlive their frame?

3. **Escape hatches**: When pure Quiche can't express something, who gets to use MetaQuiche—just stdlib authors, or everyone?

---

## The Pitch

> "I want to write Python, but I need it to be fast and safe."

Quiche gives you that. Python's syntax. Rust's performance. A simpler memory model that doesn't require a PhD to understand.

Write code that looks like this:

```python
def main() -> Result[(), Error]:
    args = parse_args()?
    data = load(args.input)?
    result = transform(data)?
    save(args.output, result)?
    return Ok(())
```

Get a binary that runs like Rust, never panics, and cleans up after itself.

That's Quiche.
