# Quiche Language Design: MetaQuiche

**Philosophy**: "Use Quiche to Build Quiche"
MetaQuiche enables powerful compile-time metaprogramming (macros) using standard Quiche syntax.

## Architecture

1.  **Macro Definition**:
    -   Macros are defined using the `@macro` decorator.
    -   They accept AST nodes as input and return transformed AST nodes.
    -   They are standard Quiche functions.

2.  **Native Compilation**:
    -   Quiche files containing macros (`@macro`) are transpiled to a Rust `proc-macro` crate.
    -   Cargo builds and loads these macros at compile time.
    -   There is no embedded interpreter; macros are compiled to native machine code.

## Syntax

```python
from quiche.ast import ClassDef, Stmt

@macro
def dataclass(cls: ClassDef) -> ClassDef:
    """
    A macro that inspects a class and automatically adds methods.
    """
    match cls:
        case ClassDef(name, body):
            # Transformation logic...
            pass
    return cls
```

## Usage

```python
from my_macros import dataclass

@dataclass
class User:
    id: i32
    name: String
```

## Dependencies
MetaQuiche relies on having an exposed AST interface (`quiche.ast`) that mirrors the compiler's internal AST, and support for `enum` and `match` to manipulate that AST.
