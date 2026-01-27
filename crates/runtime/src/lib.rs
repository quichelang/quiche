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
