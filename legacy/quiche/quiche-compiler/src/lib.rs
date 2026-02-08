//! Quiche compiler library.
//!
//! Provides the Elevate-based compilation pipeline for use by the CLI and build system.
//! Pipeline: Quiche source → parser → Quiche AST → desugar → Elevate IR → Rust code

#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

/// Re-export desugar from quiche-bridge (single source of truth)
pub use quiche_bridge::desugar;

use metaquiche_shared::telemetry::{CompileContext, Diagnostic, Emitter};

/// Compile Quiche source code to Rust via the Elevate pipeline.
///
/// Pipeline: Quiche source → metaquiche-parser → Quiche AST → desugar → Elevate AST
///           → Elevate type inference → ownership planner → Rust codegen
pub fn compile_via_elevate(source: &str, filename: &str) -> Option<String> {
    let ctx = CompileContext::new(filename, source);

    // Step 1: Parse with metaquiche-parser
    let parsed = match metaquiche_parser::parse(source) {
        Ok(module) => module,
        Err(e) => {
            Emitter::print_failed_header(filename);
            let err_str = e.to_string();
            let diag = Diagnostic::error(&err_str);
            eprintln!(
                "{}",
                metaquiche_shared::telemetry::format_diagnostic(&diag, Some(&ctx))
            );
            return None;
        }
    };

    // Step 2: Desugar Quiche AST → Elevate AST
    let elevate_ast = desugar::lower(&parsed);

    // Step 3: Type inference (Elevate pass)
    let tc_opts = elevate::passes::TypecheckOptions {
        numeric_coercion: true,
    };
    let typed = match elevate::passes::lower_to_typed_with_options(&elevate_ast, &tc_opts) {
        Ok(module) => module,
        Err(diagnostics) => {
            Emitter::print_failed_header(filename);
            for diag in &diagnostics {
                eprintln!("[elevate] {diag}");
            }
            return None;
        }
    };

    // Step 4: Ownership analysis + lowering (Elevate pass)
    let lowered = elevate::passes::lower_to_rust(&typed);

    // Step 5: Emit Rust code (Elevate codegen)
    let rust_code = elevate::codegen::emit_rust_module(&lowered);

    Some(rust_code)
}
