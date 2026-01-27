pub trait BorrowIfString<'a> {
    type Output;
    fn borrow_if_string(&'a self) -> Self::Output;
}

// If it's a String, lend it as &str
impl<'a> BorrowIfString<'a> for String {
    type Output = &'a str;
    fn borrow_if_string(&'a self) -> &'a str {
        self.as_str()
    }
}

// If it's already &str, just pass it through
impl<'a> BorrowIfString<'a> for str {
    type Output = &'a str;
    fn borrow_if_string(&'a self) -> &'a str {
        self
    }
}

// For primitives (like i32), we treat them as pass-through.
// Note: Since method resolution takes &self, we get &i32.
// If the target function takes i32 (by value), passing &i32 might fail if we don't deref.
// However, the macro generates `(&arg).borrow_if_string()`.
// If we return `i32` here, we are returning a value.
// Rust allows temporary lifetime extension.
impl<'a> BorrowIfString<'a> for i32 {
    type Output = i32;
    fn borrow_if_string(&'a self) -> i32 {
        *self
    }
}

impl<'a> BorrowIfString<'a> for bool {
    type Output = bool;
    fn borrow_if_string(&'a self) -> bool {
        *self
    }
}

// Generic fallback for other types?
// Use a macro rule for other common types or implement for T via specialization (unstable).
// For now, let's add common types used in Quiche AST/Compiler.

#[macro_export]
macro_rules! call {
    ($func:path, $($arg:expr),*) => {
        {
            use $crate::BorrowIfString;
            $func( $( (&$arg).borrow_if_string() ),* )
        }
    };
}
