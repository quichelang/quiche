//! Core macros for Quiche-generated code
//!
//! These macros are used by the generated Rust code from both
//! metaquiche-host and metaquiche-native compilers.

/// Reference macro - creates an immutable borrow
#[macro_export]
macro_rules! qref {
    ($e:expr) => {
        &($e)
    };
}

/// Mutable reference macro - creates a mutable borrow
#[macro_export]
macro_rules! mutref {
    ($e:expr) => {
        &mut ($e)
    };
}

/// Dereference macro - dereferences a pointer/box
#[macro_export]
macro_rules! deref {
    ($e:expr) => {
        *($e)
    };
}

/// String concatenation macro - efficiently concatenates multiple values into a String
#[macro_export]
macro_rules! strcat {
    ($arg:expr) => { ($arg).to_string() };
    ($first:expr, $($rest:expr),+ $(,)?) => {{
        let mut __s = ($first).to_string();
        $( __s.push_str(&($rest).to_string()); )+
        __s
    }};
}
