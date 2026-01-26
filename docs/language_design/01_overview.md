# Quiche Language Design: Overview

**Philosophy**: "Pure Quiche"
Quiche is a language that offers the **ergonomics of Python** with the **performance and safety of Rust**. It is not an interpreted language; it is a high-level frontend for Rust.

## Core Principles

1.  **Python Syntax, Rust Semantics**:
    -   We use Python's syntax (`def`, `class`, `if`, `match`) because it is readable and familiar.
    -   We enforce Rust's semantics (static typing, move semantics, exhaustiveness) because they are correct and performant.

2.  **Native Compilation**:
    -   Quiche compiles directly to idiomatic Rust.
    -   There is no runtime overhead (VM or GC).
    -   `match` compiles to `match`. `class` compiles to `struct`.

3.  **No "Holes"**:
    -   We do not attempt to emulate Python's dynamic features (like `getattr` on arbitrary objects or runtime type modification).
    -   If a feature cannot be statically analyzed or compiled to safe Rust, it is generally excluded or requires `unsafe` blocks.

## Syntax Mapping Legend

| Feature | Quiche Syntax | Rust Compilation |
| :--- | :--- | :--- |
| **Blocks** | Indentation (Whitespace) | Braces `{}` |
| **Variables** | `x: i32 = 10` | `let x: i32 = 10;` |
| **Mutation** | `x = 10` (Inferred) | `let mut x = 10;` |
| **Functions** | `def foo(x: i32) -> i32:` | `fn foo(x: i32) -> i32` |
| **Structs** | `class Point:` | `struct Point` |
| **Enums** | `enum Color:` | `enum Color` |
| **Matching** | `match x:` | `match x` |
| **Macros** | `@macro` | `#[proc_macro]` |
