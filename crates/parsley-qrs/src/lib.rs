#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

// Alias for compatibility with generated code that uses crate::quiche::*
mod quiche {
    pub use quiche_runtime::{QuicheGeneric, QuicheResult, call, check};
}

// Re-export everything from the transpiled module
include!(concat!(env!("OUT_DIR"), "/lib.rs"));
