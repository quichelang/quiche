# Design Philosophy

## Language Stack

```
Rust → MetaQuiche → Quiche
```

### MetaQuiche

A lower-level dialect for implementing core language features:

- Can interface with Rust code directly
- Supports most Rust primitives: Traits, Structs, Enums, Generics
- Compile-time safety with no reliance on runtime checks
- "Panics" are compile-time errors
- Python-compatible syntax
- Fast compilation

### Quiche

A higher-level dialect for implementing application features:

- Can interface with MetaQuiche code
- Supports a subset of MetaQuiche features
- Builds on top of MetaQuiche's safety guarantees
- Automatic memory management and borrowing rules
- Python compatibility layer (lists, dicts, stdlib functions, builtins, etc.)
- Ability to write Python libraries

## Why Quiche?

I wanted an expressive, embeddable language with native Rust interop—something with compile-time safety and speed comparable to Rust itself.

I tried templating languages, dynamically loaded Rust modules, and macro DSLs. Each had drawbacks. Macros came closest but add a debugging layer that's hard to work with.

What started as a macro project kept growing. Adding proper checks meant writing a linter. I missed Python's rapid prototyping. Then I remembered Ruff was written in Rust. Could I use its parser? That gave me freedom to create a language with Python syntax but native Rust types.

After many iterations, we've rewritten almost everything. The language no longer depends on Ruff. Currently 16 dependencies, easily reducible further.
