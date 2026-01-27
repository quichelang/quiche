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

// Macro to wrap calls
#[macro_export]
macro_rules! call {
    ($func:expr $(, $arg:expr)*) => {
        {
            // Import traits locally to ensure method resolution works
            use $crate::{QuicheResult, QuicheGeneric};
            $func( $( ($arg).quiche_handle() ),* )
        }
    };
}

// Also used for return values etc if needed, but call! handles args.
// Host compiler uses crate::check! for return values / exprs.
// We might need to export check! too if self-host uses it?
// self-host uses `quiche_runtime::call!` for calls.
// What about non-call expressions?
// Host uses `crate::quiche::check!(...)`.
// Self-host compiler `generate_expr` might emit `check!`?
// Step 1586 host logic emits `check!` for many things.
// Use grep to see if self-host emits `check!`.
