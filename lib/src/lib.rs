//! Quiche standard library — primitive newtypes for the Quiche language.
//!
//! Provides `Str`, `List<T>`, and `Dict<K,V>` as ergonomic wrappers
//! around Rust's standard types with chainable APIs.
//!
//! Also provides Elixir-style modules: `File`, `Path`, `System`, `Enum`.
//!
//! All newtypes implement the [`QuicheType`] trait, giving them
//! `.view()` (borrow inner) and `.inner()` (consume wrapper).

mod dict;
mod enum_module;
mod file_module;
mod list;
mod path_module;
mod quiche_type;
mod str_type;
mod system_module;

pub use dict::Dict;
pub use enum_module::Enum;
pub use file_module::File;
pub use list::List;
pub use path_module::Path;
pub use quiche_type::QuicheType;
pub use str_type::{Str, str};
pub use system_module::System;
