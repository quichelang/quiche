use std::ops::Deref;

/// Trait shared by all Quiche newtypes (`Str`, `List<T>`, `Dict<K,V>`).
///
/// Provides two ways to access the wrapped value:
/// - `.view()` — borrow the inner value (via `Deref`)
/// - `.inner()` — consume the wrapper, returning the owned inner value
pub trait QuicheType: Deref {
    /// The owned inner type returned by `.inner()`.
    ///
    /// This may differ from `Deref::Target` — for example, `Str` derefs to
    /// `str` (unsized) but `.inner()` returns `Arc<str>` (owned, sized).
    type Inner;

    /// Borrow the inner value without consuming.
    ///
    /// ```ignore
    /// names: List[String] = ["Alice", "Bob"]
    /// v: &Vec[String] = names.view()
    /// ```
    fn view(&self) -> &<Self as Deref>::Target {
        self.deref()
    }

    /// Consume the wrapper, returning the owned inner value.
    ///
    /// ```ignore
    /// names: List[String] = ["Alice", "Bob"]
    /// v: Vec[String] = names.inner()
    /// ```
    fn inner(self) -> Self::Inner;
}

/// Implement `QuicheType` for a newtype wrapper whose inner value is in `.0`.
///
/// Usage:
/// ```ignore
/// impl_quiche_type!(Str, Arc<str>);
/// impl_quiche_type!(List<T>, Vec<T>);
/// impl_quiche_type!(Dict<K: Eq + Hash, V: PartialEq>, HashMap<K, V>);
/// ```
#[macro_export]
macro_rules! impl_quiche_type {
    // Non-generic: impl_quiche_type!(Str, Arc<str>)
    ($ty:ty, $inner:ty) => {
        impl $crate::QuicheType for $ty {
            type Inner = $inner;
            fn inner(self) -> $inner {
                self.0
            }
        }
    };
    // Generic: impl_quiche_type!(List<T>, Vec<T>)
    ($ty:ident < $($gen:ident $(: $($bound:path)+)?),+ >, $inner:ty) => {
        impl< $($gen $(: $($bound +)?)?),+ > $crate::QuicheType for $ty< $($gen),+ >
        where
            Self: std::ops::Deref,
        {
            type Inner = $inner;
            fn inner(self) -> $inner {
                self.0
            }
        }
    };
}
