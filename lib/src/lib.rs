//! Quiche standard library — primitive newtypes for the Quiche language.
//!
//! Provides `Str`, `List<T>`, and `Dict<K,V>` as ergonomic wrappers
//! around Rust's standard types with chainable APIs.
//!
//! All newtypes implement the [`QuicheType`] trait, giving them
//! `.view()` (borrow inner) and `.inner()` (consume wrapper).

mod dict;
mod list;
mod quiche_type;
mod str_type;

pub use dict::Dict;
pub use list::List;
pub use quiche_type::QuicheType;
pub use str_type::{Str, str};
