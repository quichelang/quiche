#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

// Alias for compatibility with generated code that uses crate::quiche::*
mod quiche {
    pub use quiche_runtime::{QuicheGeneric, QuicheResult, call, check};
}

use quiche_runtime::as_ref;

// Re-export everything from the transpiled module
include!(concat!(env!("OUT_DIR"), "/lib.rs"));

pub fn impl_construct_flag_spec(
    name: String,
    aliases: Vec<String>,
    takes_value: bool,
    default_bool: bool,
    default_value: String,
    has_default: bool,
) -> FlagSpec {
    FlagSpec {
        name,
        aliases,
        takes_value,
        default_bool,
        default_value,
        has_default,
    }
}

pub fn impl_construct_parse_result(
    flags: std::collections::HashMap<String, bool>,
    options: std::collections::HashMap<String, String>,
    positionals: Vec<String>,
    errors: Vec<String>,
    command: String,
    subargs: Vec<String>,
) -> ParseResult {
    ParseResult {
        flags,
        options,
        positionals,
        errors,
        command,
        subargs,
    }
}

pub fn impl_construct_parsley(
    specs: Vec<FlagSpec>,
    by_name: std::collections::HashMap<String, FlagSpec>,
    alias_map: std::collections::HashMap<String, String>,
    commands: Vec<String>,
) -> Parsley {
    Parsley {
        specs,
        by_name,
        alias_map,
        commands,
    }
}
