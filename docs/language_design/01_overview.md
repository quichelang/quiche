# Quiche Language Design: Overview

**Philosophy**: "Pure Quiche"
Quiche is a language that offers the **ergonomics of Python** with the **performance and safety of Rust**. It is not an interpreted language; it is a high-level frontend for Rust.

## Core Principles

1.  **Less Verbose Rust (The 80/20 Rule)**:
    -   We aim to support 80% of use-cases with 20% less flexibility if it means significantly less verbosity.
    -   We make meaningful assumptions (like "Strings are usually owned") to remove friction.
    -   Constraints are added to support performance without the cognitive load of full Rust.

2.  **Python Syntax, Rust Semantics**:
    -   We use Python's syntax (`def`, `class`, `if`, `match`) because it is readable and familiar.
    -   We enforce Rust's semantics (static typing, move semantics, exhaustiveness).

3.  **Native Compilation**:
    -   Quiche compiles directly to idiomatic Rust.
    -   There is no runtime overhead (VM or GC).

4.  **No "Holes"**:
    -   We do not attempt to emulate Python's dynamic features (like `getattr` on arbitrary objects).

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
