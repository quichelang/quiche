# Qtest Runner Compilation Grievance

## Issue: Scripts Cannot Import Runtime Modules

**Date:** 2026-02-02
**Component:** qtest_runner.qrs, qtest.qrs, runtime imports
**Severity:** Blocking

### Description

When running a standalone `.qrs` script (like `tests/qtest_runner.qrs`), the generated Rust code tries to import from `crate::qtest` and `crate::quiche`. These imports fail because:

1. The script is compiled as an isolated program
2. The `qtest` module only exists inside `quiche-runtime`
3. The codegen generates paths like `crate::qtest::*` which work inside a crate but not for standalone scripts

### Error Output

```
error[E0432]: unresolved import `crate::qtest`
  --> target/tmp.rs:60:12
   |
60 | ...e::qtest::TestR...
   |       ^^^^^ could not find `qtest` in the crate root

error[E0432]: unresolved import `crate::quiche::env_args_helper`
  --> target/tmp.rs:68:9
   |
68 | ...se crate::quiche::env_args_helper as env_args;
   |       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `env_args_helper` in `quiche`
```

### Root Cause

The `from qtest import ...` statement in `.qrs` files is translated to `use crate::qtest::...` which only works when the module is compiled as part of the same crate. Standalone scripts have no access to `quiche-runtime` modules.

### Required Solution

Two options:
1. **Bundled runtime**: Scripts should automatically link against or include `quiche-runtime` as a dependency
2. **External crate imports**: The codegen should emit `use quiche_runtime::qtest::...` for imports that reference runtime modules

### Impact

- Cannot create a working qtest runner as a standalone script
- Cannot use any `quiche-runtime` modules from scripts
- Limits utility of the scripting capability

### Workaround

Scripts must inline all needed functionality or use only `@extern` declarations to Rust stdlib.

### Files Affected

- `tests/qtest_runner.qrs`
- `quiche/quiche-runtime/src/qtest.qrs`
- Codegen import resolution logic
