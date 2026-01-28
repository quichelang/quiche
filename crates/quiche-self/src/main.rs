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

    pub fn env_args_helper() -> Vec<String> {
        std::env::args().collect()
    }

    pub fn push_str_wrapper(mut s: String, val: String) -> String {
        s.push_str(&val);
        s
    }
}

#[cfg(feature = "bootstrap")]
include!("main_gen.rs");

#[cfg(not(feature = "bootstrap"))]
include!(concat!(env!("OUT_DIR"), "/main.rs"));
