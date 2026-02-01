pub mod re;

// High Priority: Consumes Self (Result/Option)
pub trait QuicheResult {
    type Output;
    fn quiche_handle(self) -> Self::Output;
}

impl<T, E: std::fmt::Display> QuicheResult for Result<T, E> {
    type Output = T;
    fn quiche_handle(self) -> T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "{}",
                    metaquiche_shared::i18n::tr1("runtime.error.generic", "error", &e.to_string())
                );
                std::process::exit(1);
            }
        }
    }
}

// Low Priority: Takes &Self (Clone fallback)
pub trait QuicheGeneric {
    fn quiche_handle(&self) -> Self;
}

impl<T: Clone> QuicheGeneric for T {
    fn quiche_handle(&self) -> Self {
        self.clone()
    }
}

// Macro to wrap calls (handles multiple args by wrapping each)
#[macro_export]
macro_rules! call {
    ($func:expr $(, $arg:expr)*) => {
        {
            use $crate::{QuicheResult, QuicheGeneric};
            $func( $( ($arg).quiche_handle() ),* )
        }
    };
}

// Macro to wrap any expression for handle calling
#[macro_export]
macro_rules! check {
    ($val:expr) => {{
        use $crate::{QuicheGeneric, QuicheResult};
        ($val).quiche_handle()
    }};
}

/// String concatenation macro - efficient push_str pattern
///
/// Quiche code:
/// ```python
/// s = "hello" + name + "!"
/// ```
///
/// Generated Rust:
/// ```rust
/// use quiche_runtime::strcat;
/// let name = "world";
/// let s = strcat!("hello ", name, "!");
/// assert_eq!(s, "hello world!");
/// ```
#[macro_export]
macro_rules! strcat {
    // Single argument - just convert to String
    ($arg:expr) => {
        ($arg).to_string()
    };
    // Multiple arguments - use push_str pattern
    ($first:expr, $($rest:expr),+ $(,)?) => {{
        let mut __s = ($first).to_string();
        $(
            __s.push_str(&($rest).to_string());
        )+
        __s
    }};
}

#[derive(Debug, Clone)]
pub struct QuicheException(pub String);

pub trait QuicheBorrow<T> {
    fn try_borrow_q(&self) -> Result<std::cell::Ref<T>, QuicheException>;
    fn try_borrow_mut_q(&self) -> Result<std::cell::RefMut<T>, QuicheException>;
}

impl<T> QuicheBorrow<T> for std::cell::RefCell<T> {
    fn try_borrow_q(&self) -> Result<std::cell::Ref<T>, QuicheException> {
        self.try_borrow()
            .map_err(|e| QuicheException(e.to_string()))
    }
    fn try_borrow_mut_q(&self) -> Result<std::cell::RefMut<T>, QuicheException> {
        self.try_borrow_mut()
            .map_err(|e| QuicheException(e.to_string()))
    }
}

pub trait QuicheIterable {
    type Item;
    type Iter: Iterator<Item = Self::Item>;
    fn quiche_iter(self) -> Self::Iter;
}

impl<T: Clone> QuicheIterable for std::rc::Rc<Vec<T>> {
    type Item = T;
    type Iter = std::vec::IntoIter<T>;
    fn quiche_iter(self) -> Self::Iter {
        self.as_ref().clone().into_iter()
    }
}

impl<T> QuicheIterable for Vec<T> {
    type Item = T;
    type Iter = std::vec::IntoIter<T>;
    fn quiche_iter(self) -> Self::Iter {
        self.into_iter()
    }
}

impl<T> QuicheIterable for std::ops::Range<T>
where
    std::ops::Range<T>: Iterator<Item = T>,
{
    type Item = T;
    type Iter = std::ops::Range<T>;
    fn quiche_iter(self) -> Self::Iter {
        self
    }
}

impl<'a, I> QuicheIterable for &'a I
where
    I: QuicheIterable + Clone,
{
    type Item = I::Item;
    type Iter = I::Iter;
    fn quiche_iter(self) -> Self::Iter {
        self.clone().quiche_iter()
    }
}

impl<'a, I> QuicheIterable for &'a mut I
where
    I: QuicheIterable + Clone,
{
    type Item = I::Item;
    type Iter = I::Iter;
    fn quiche_iter(self) -> Self::Iter {
        (*self).clone().quiche_iter()
    }
}

impl<'a, K, V> QuicheIterable for std::collections::hash_map::Keys<'a, K, V> {
    type Item = &'a K;
    type Iter = std::collections::hash_map::Keys<'a, K, V>;
    fn quiche_iter(self) -> Self::Iter {
        self
    }
}

impl<'a, K, V> QuicheIterable for std::collections::hash_map::Values<'a, K, V> {
    type Item = &'a V;
    type Iter = std::collections::hash_map::Values<'a, K, V>;
    fn quiche_iter(self) -> Self::Iter {
        self
    }
}

impl<'a, K, V> QuicheIterable for std::collections::hash_map::Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    type Iter = std::collections::hash_map::Iter<'a, K, V>;
    fn quiche_iter(self) -> Self::Iter {
        self
    }
}

impl<T> QuicheIterable for Box<[T]> {
    type Item = T;
    type Iter = std::vec::IntoIter<T>;
    fn quiche_iter(self) -> Self::Iter {
        self.into_vec().into_iter()
    }
}

pub trait QuicheDeref {
    type Target;
    fn quiche_deref(&self) -> Self::Target;
}

impl<T: Clone> QuicheDeref for Box<T> {
    type Target = T;
    fn quiche_deref(&self) -> T {
        (**self).clone()
    }
}

impl<T: Clone> QuicheDeref for Option<Box<T>> {
    type Target = T;
    fn quiche_deref(&self) -> T {
        match self.as_ref() {
            Some(v) => v.as_ref().clone(),
            None => {
                eprintln!(
                    "{}",
                    metaquiche_shared::i18n::tr("runtime.error.deref_none")
                );
                std::process::exit(1);
            }
        }
    }
}

#[macro_export]
macro_rules! deref {
    ($e:expr) => {{
        use $crate::QuicheDeref;
        ($e).quiche_deref()
    }};
}

// qref! - immutable borrow (called as ref() in Quiche code)
#[macro_export]
macro_rules! qref {
    ($e:expr) => {
        &($e)
    };
}

// mutref! - mutable borrow
#[macro_export]
macro_rules! mutref {
    ($e:expr) => {
        &mut ($e)
    };
}
