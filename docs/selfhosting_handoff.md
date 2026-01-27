
## Complete Plan

### Implementation Plan - Full Self-Hosting Parity
This plan outlines the steps to bridge the remaining gaps in the quiche-self transpiler, ensuring it can handle all constructs used by the host compiler and pass the entire integration suite.
User Review Required

#### IMPORTANT
Priority Shift: We will address Interop (Phase 1) and Architecture (Phase 2) before the complex try-except (Phase 3) to ensure a stable foundation.

### Proposed Changes

#### Phase 1: Interop & Decorators (Priority 3)
Focus on resolving 

test_interop.qrs and generalizing static attribute access.

[MODIFY] 

compiler.qrs
* Decorator Handling: Update ClassDef and FunctionDef to handle ast.Expr.Call decorators.
    * Extract path and no_generic keywords from @extern(...).
* Static Separator: Generalize the is_static check in Attribute handler.
    * Use :: if the target is an extern class or a recognized type name.

#### Phase 2: Architectural Improvements (Priority 2)
Moving away from heuristics to a structured SymbolTable.
[MODIFY] 

compiler.qrs
* Symbol Table Implementation:
    * Add a SymbolTable class tracking is_extern, is_class, and inferred types.
    * Update generate_stmt to populate the table (e.g., on ClassDef, FunctionDef, Assign).
* Refined Inference: Use table lookups to decide between . and :: for attribute access.

#### Phase 3: Advanced Language Constructs (Priority 1)
Implementing the remaining high-level Python/Quiche features.
[MODIFY] 

compiler.qrs
* F-Strings: Transpile ast.Expr.FString to Rust format!().
* Dict Literals: Transpile {k: v} to std::collections::HashMap::from(...).
* Try-Except:
    * Implement the Try statement handler.
    * Wrap the body in std::panic::catch_unwind(AssertUnwindSafe(|| { ... })).
    * Implement ExceptHandler to downcast and bind the error.
* Consistency: Ensure all calls and fallible expressions are wrapped in crate::quiche::check!.

### Verification Plan

#### Automated Tests
* Run cargo test -p quiche_self after each phase.
* Specifically verify test_interop.qrs (Phase 1).
* Verify test_types_suite.qrs (Phase 2 & 3).

#### Manual Verification
* Inspect the generated compiler.rs for correctly emitted try/except and format! logic.

#### Commit Strategy
* After successful verification of each phase, commit the changes:jj commit -m "feat(quiche-self): phase [N] - [Goal]"


## Task Breakdown

### Task: Achieve Full Self-Hosting Parity

#### Phase 1: Interop & Decorators (Priority 3)
* 		 Fix decorator argument parsing in compiler.qrs
    * 		 Support @extern(path="...") and no_generic
* 		 Generalize static attribute access (:: vs .)
* 		Verify test_interop.qrs passes
* 		 Commit Phase 1 changes
#### Phase 2: Architectural Improvements (Priority 2)
* 		 Build robust SymbolTable in quiche-self
* 		 Migrate is_static and type inference to use Symbol Table
* 		 Improve import resolution and crate:: prefixing logic
* 		 Commit Phase 2 changes
#### Phase 3: Advanced Language Constructs (Priority 1)
* 		 Implement F-String transpilation
* 		 Implement Dict literal transpilation (HashMap::from)
* 		 Standardize check! macro wrapping for all expressions
* 		 Implement try-except (Panic catching)
* 		 Commit Phase 3 changes

* 		 Initial improvements (Lambda, Neg Indexing, Shared Cache)

