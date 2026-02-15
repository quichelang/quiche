# Design Philosophy

## What Is Quiche?

Quiche is a language with **Python's syntax** and **Rust's performance**. It compiles `.q` source files to native binaries via Rust — no VM, no garbage collector.

## Pipeline

```
.q source → Parser → Elevate IR → Type Inference → Rust codegen → rustc → binary
```

The compiler uses the [Elevate](https://github.com/nicholasgasior/elevate) backend for type inference, ownership analysis, and Rust code generation. The parser and frontend are written in Rust; the language itself is purely `.q` files.

## Core Principles

1. **Python syntax, Rust semantics** — `def`, `type`, `match`, indentation. Static typing, move semantics, exhaustiveness.

2. **Less verbose Rust** — support 80% of use-cases with 20% less flexibility. Meaningful defaults (strings are `Str`, lists are `List[T]`).

3. **No holes** — no dynamic features (`getattr`, `eval`). Everything is compile-time checked.

4. **Auto-borrowing** — the compiler inserts borrows and clones automatically. No `&`, `&mut`, or lifetime annotations.

5. **Native compilation** — compiles to idiomatic Rust. Zero runtime overhead.

## Why Quiche?

Rust is brilliant but asks a lot: lifetimes, borrow checker, ownership annotations. For many programs — CLI tools, data pipelines, business logic — this is overkill.

Python gets ergonomics right but is slow and dynamically typed.

**Quiche gives you both**: Python's readability with Rust's safety and performance. The compiler handles the hard parts so you write the happy path.
