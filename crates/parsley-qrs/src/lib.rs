#![allow(dead_code, unused_variables, unused_mut, unused_imports, unused_parens)]

mod quiche {
    #![allow(unused_macros, unused_imports)]

    // High Priority: Consumes Self (Result/Option)
    pub trait QuicheResult {
        type Output;
        fn quiche_handle(self) -> Self::Output;
    }

    impl<T, E: std::fmt::Debug> QuicheResult for Result<T, E> {
        type Output = T;
        fn quiche_handle(self) -> T {
            self.expect("Quiche Exception")
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

    macro_rules! check {
        ($val:expr) => {{
            use crate::quiche::{QuicheGeneric, QuicheResult};
            ($val).quiche_handle()
        }};
    }
    pub(crate) use check;
    pub(crate) use check as call;
}

// Re-export everything from the transpiled module
include!(concat!(env!("OUT_DIR"), "/lib.rs"));
