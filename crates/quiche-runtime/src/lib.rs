#![allow(
    non_snake_case,
    unused_variables,
    unused_mut,
    named_arguments_used_positionally,
    unused_parens
)]

pub mod ast_transformer {
    include!(concat!(env!("OUT_DIR"), "/ast_transformer.rs"));
}
pub mod memory_analysis {
    include!(concat!(env!("OUT_DIR"), "/memory_analysis.rs"));
}
pub mod introspect {
    include!(concat!(env!("OUT_DIR"), "/introspect.rs"));
}
pub mod qtest {
    include!(concat!(env!("OUT_DIR"), "/qtest.rs"));
}
pub mod pathlib {
    include!(concat!(env!("OUT_DIR"), "/pathlib.rs"));
}
pub mod re;

// Re-export perceus-mem types for use in generated code
pub use perceus_mem::{Handle, Managed, Region, Store, Weak};

// ============================================================================
// Memory Analysis Support
// ============================================================================

/// Allocation strategy for a type/value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AllocationStrategy {
    Inline, // Stack-only, Copy types
    Region, // Arena allocation
    #[default]
    Managed, // FBIP copy-on-write (default)
    Store,  // Long-lived, shared
}

/// Configuration from @mem decorator
#[derive(Debug, Clone, Default)]
pub struct MemConfig {
    pub is_inline: bool,
    pub is_region: bool,
    pub is_raw: bool,
}

pub fn create_MemConfig() -> MemConfig {
    MemConfig::default()
}

/// Escape analysis result
#[derive(Debug, Clone, Default)]
pub struct EscapeInfo {
    pub escaping: Vec<String>,
    pub local_only: Vec<String>,
}

pub fn create_EscapeInfo() -> EscapeInfo {
    EscapeInfo::default()
}

/// Memory analyzer state
#[derive(Debug, Clone, Default)]
pub struct MemoryAnalyzer {
    pub type_strategies: std::collections::HashMap<String, AllocationStrategy>,
    pub func_configs: std::collections::HashMap<String, MemConfig>,

    pub current_escape: EscapeInfo,
    pub inline_types: Vec<String>,
    pub verbose: bool,
}

pub fn create_MemoryAnalyzer() -> MemoryAnalyzer {
    MemoryAnalyzer::default()
}

// ============================================================================
// Introspection Support
// ============================================================================

/// Metadata for a registered function
#[derive(Debug, Clone, Default)]
pub struct FunctionMeta {
    pub name: String,
    pub arity: usize,
    pub signature: String,
    pub docstring: Option<String>,
    pub is_test: bool,
}

/// Metadata for a registered module
#[derive(Debug, Clone, Default)]
pub struct ModuleInfo {
    pub name: String,
    pub functions: std::collections::HashMap<String, FunctionMeta>,
    pub constants: std::collections::HashMap<String, String>,
}

/// Central runtime state object holding module registry
#[derive(Debug, Clone, Default)]
pub struct QuicheRuntime {
    pub modules: std::collections::HashMap<String, ModuleInfo>,
    pub current_module: String,
}

pub fn introspect_create_FunctionMeta(
    name: String,
    arity: usize,
    signature: String,
    docstring: Option<String>,
    is_test: bool,
) -> FunctionMeta {
    FunctionMeta {
        name,
        arity,
        signature,
        docstring,
        is_test,
    }
}

pub fn introspect_create_ModuleInfo(name: String) -> ModuleInfo {
    ModuleInfo {
        name,
        functions: std::collections::HashMap::new(),
        constants: std::collections::HashMap::new(),
    }
}

pub fn introspect_create_QuicheRuntime() -> QuicheRuntime {
    QuicheRuntime::default()
}

// ============================================================================
// qtest Support
// ============================================================================

/// Exception raised when an assertion fails
#[derive(Debug, Clone, Default)]
pub struct AssertionError {
    pub message: String,
}

/// Test result enum
#[derive(Debug, Clone)]
pub enum TestResult {
    Passed,
    Failed(String),
    Skipped(String),
}

/// Summary of test run results
#[derive(Debug, Clone, Default)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub failures: Vec<String>,
}

/// Panic with a message - wrapper for panic! macro
pub fn qtest_panic(message: String) {
    panic!("{}", message);
}

pub fn qtest_create_AssertionError(message: String) -> AssertionError {
    AssertionError { message }
}

pub fn qtest_create_TestSummary() -> TestSummary {
    TestSummary::default()
}

// ============================================================================
// Diagnostic Emission
// ============================================================================

/// Emit a warning message (delegates to telemetry)
pub fn emit_warning(message: String) {
    eprintln!(
        "{}: {}",
        metaquiche_shared::i18n::tr("diagnostic.level.warning"),
        message
    );
}

/// Emit an error message (delegates to telemetry)
pub fn emit_error(message: String) {
    eprintln!(
        "{}: {}",
        metaquiche_shared::i18n::tr("diagnostic.level.error"),
        message
    );
}

/// Emit a note message
pub fn emit_note(message: String) {
    eprintln!(
        "{}: {}",
        metaquiche_shared::i18n::tr("diagnostic.level.note"),
        message
    );
}

pub mod quiche {
    pub use crate::{
        QuicheBorrow, QuicheDeref, QuicheException, QuicheGeneric, QuicheIterable, QuicheResult,
    };
    pub use crate::{create_QuicheModule, create_Transformer};
    // Re-export macros
    pub use crate::{call, check, deref, mutref, qref, strcat};

    // Path struct for pathlib
    #[derive(Debug, Clone, Default)]
    pub struct Path {
        pub path: String,
    }

    pub fn create_Path(path: String) -> Path {
        Path { path }
    }
}

pub fn create_QuicheModule(
    body: Vec<quiche_parser::ast::Stmt>,
) -> quiche_parser::ast::QuicheModule {
    quiche_parser::ast::QuicheModule { body }
}

pub fn create_Transformer() -> ast_transformer::Transformer {
    ast_transformer::Transformer::default()
}

pub fn box_expr(e: quiche_parser::ast::QuicheExpr) -> Box<quiche_parser::ast::QuicheExpr> {
    Box::new(e)
}

pub fn create_name_expr(s: String) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::Name(s)
}

pub fn ast_get_func_name(f: &quiche_parser::ast::FunctionDef) -> String {
    f.name.clone()
}

pub fn ast_update_func_body(
    mut f: quiche_parser::ast::FunctionDef,
    body: Vec<quiche_parser::ast::Stmt>,
) -> quiche_parser::ast::FunctionDef {
    f.body = body;
    f
}

pub fn ast_update_arg_annotation(
    mut arg: quiche_parser::ast::Arg,
    ann: Option<Box<quiche_parser::ast::QuicheExpr>>,
) -> quiche_parser::ast::Arg {
    arg.annotation = ann;
    arg
}

pub fn ast_wrap_mutref_type(
    inner: Box<quiche_parser::ast::QuicheExpr>,
) -> Box<quiche_parser::ast::QuicheExpr> {
    // Subscript { value: Name("mutref"), slice: inner }
    Box::new(quiche_parser::ast::QuicheExpr::Subscript {
        value: Box::new(quiche_parser::ast::QuicheExpr::Name("mutref".to_string())),
        slice: inner,
    })
}

pub fn ast_wrap_mutref_call(
    inner: Box<quiche_parser::ast::QuicheExpr>,
) -> Box<quiche_parser::ast::QuicheExpr> {
    // Call { func: Name("mutref"), args: [inner], keywords: [] }
    Box::new(quiche_parser::ast::QuicheExpr::Call {
        func: Box::new(quiche_parser::ast::QuicheExpr::Name("mutref".to_string())),
        args: vec![*inner],
        keywords: vec![],
    })
}

pub fn ast_cast_usize(
    inner: Box<quiche_parser::ast::QuicheExpr>,
) -> Box<quiche_parser::ast::QuicheExpr> {
    // Cast { expr: inner, target_type: Name("usize") }
    Box::new(quiche_parser::ast::QuicheExpr::Cast {
        expr: inner,
        target_type: Box::new(quiche_parser::ast::QuicheExpr::Name("usize".to_string())),
    })
}

pub fn ast_is_name(e: &quiche_parser::ast::QuicheExpr) -> bool {
    matches!(e, quiche_parser::ast::QuicheExpr::Name(_))
}

pub fn ast_get_name(e: &quiche_parser::ast::QuicheExpr) -> String {
    match e {
        quiche_parser::ast::QuicheExpr::Name(n) => n.clone(),
        _ => String::new(),
    }
}

pub fn ast_call_func_is_name(c: &quiche_parser::ast::QuicheExpr, name: &str) -> bool {
    // c is Call. func is Box<Expr>.
    if let quiche_parser::ast::QuicheExpr::Call { func, .. } = c {
        if let quiche_parser::ast::QuicheExpr::Name(n) = &**func {
            return n == name;
        }
    }
    false
}

pub fn ast_set_func_args(
    mut f: quiche_parser::ast::FunctionDef,
    args: Vec<quiche_parser::ast::Arg>,
) -> quiche_parser::ast::FunctionDef {
    f.args = args;
    f
}

pub fn check_first_upper(s: String) -> bool {
    s.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
}

pub fn check_prefix(s: String, prefix: String) -> bool {
    s.starts_with(&prefix)
}

pub fn ast_create_assign(
    targets: Vec<quiche_parser::ast::QuicheExpr>,
    value: Box<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::Assign {
    quiche_parser::ast::Assign { targets, value }
}

pub fn ast_create_if(
    test: Box<quiche_parser::ast::QuicheExpr>,
    body: Vec<quiche_parser::ast::Stmt>,
    orelse: Vec<quiche_parser::ast::Stmt>,
) -> quiche_parser::ast::IfStmt {
    quiche_parser::ast::IfStmt { test, body, orelse }
}

pub fn ast_create_for(
    target: Box<quiche_parser::ast::QuicheExpr>,
    iter: Box<quiche_parser::ast::QuicheExpr>,
    body: Vec<quiche_parser::ast::Stmt>,
    orelse: Vec<quiche_parser::ast::Stmt>,
) -> quiche_parser::ast::ForStmt {
    quiche_parser::ast::ForStmt {
        target,
        iter,
        body,
        orelse,
    }
}

pub fn ast_create_call(
    func: Box<quiche_parser::ast::QuicheExpr>,
    args: Vec<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::Call {
        func,
        args,
        keywords: vec![],
    }
}

pub fn ast_create_call_with_keywords(
    func: Box<quiche_parser::ast::QuicheExpr>,
    args: Vec<quiche_parser::ast::QuicheExpr>,
    keywords: Vec<quiche_parser::ast::Keyword>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::Call {
        func,
        args,
        keywords,
    }
}

pub fn ast_create_keyword(
    arg: Option<String>,
    value: Box<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::Keyword {
    quiche_parser::ast::Keyword { arg, value }
}

pub fn ast_create_subscript(
    value: Box<quiche_parser::ast::QuicheExpr>,
    slice: Box<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::Subscript { value, slice }
}

pub fn ast_create_binop(
    left: Box<quiche_parser::ast::QuicheExpr>,
    op: quiche_parser::ast::Operator,
    right: Box<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::BinOp { left, op, right }
}

pub fn ast_create_boolop(
    op: quiche_parser::ast::BoolOperator,
    values: Vec<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::BoolOp { op, values }
}

pub fn ast_create_unaryop(
    op: quiche_parser::ast::UnaryOperator,
    operand: Box<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::UnaryOp { op, operand }
}

pub fn ast_create_list(
    elts: Vec<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::List(elts)
}

pub fn ast_create_attribute(
    value: Box<quiche_parser::ast::QuicheExpr>,
    attr: String,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::Attribute { value, attr }
}

pub fn ast_create_tuple(
    elts: Vec<quiche_parser::ast::QuicheExpr>,
) -> quiche_parser::ast::QuicheExpr {
    quiche_parser::ast::QuicheExpr::Tuple(elts)
}

pub fn make_if_stmt(
    test: Box<quiche_parser::ast::QuicheExpr>,
    body: Vec<quiche_parser::ast::QuicheStmt>,
    orelse: Vec<quiche_parser::ast::QuicheStmt>,
) -> quiche_parser::ast::QuicheStmt {
    quiche_parser::ast::QuicheStmt::If(quiche_parser::ast::IfStmt { test, body, orelse })
}

pub fn make_while_stmt(
    test: Box<quiche_parser::ast::QuicheExpr>,
    body: Vec<quiche_parser::ast::QuicheStmt>,
    orelse: Vec<quiche_parser::ast::QuicheStmt>,
) -> quiche_parser::ast::QuicheStmt {
    quiche_parser::ast::QuicheStmt::While(quiche_parser::ast::WhileStmt { test, body, orelse })
}

pub fn make_for_stmt(
    target: Box<quiche_parser::ast::QuicheExpr>,
    iter: Box<quiche_parser::ast::QuicheExpr>,
    body: Vec<quiche_parser::ast::QuicheStmt>,
    orelse: Vec<quiche_parser::ast::QuicheStmt>,
) -> quiche_parser::ast::QuicheStmt {
    quiche_parser::ast::QuicheStmt::For(quiche_parser::ast::ForStmt {
        target,
        iter,
        body,
        orelse,
    })
}

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
    fn try_borrow_q<'a>(&'a self) -> Result<std::cell::Ref<'a, T>, QuicheException>;
    fn try_borrow_mut_q<'a>(&'a self) -> Result<std::cell::RefMut<'a, T>, QuicheException>;
}

impl<T> QuicheBorrow<T> for std::cell::RefCell<T> {
    fn try_borrow_q<'a>(&'a self) -> Result<std::cell::Ref<'a, T>, QuicheException> {
        self.try_borrow()
            .map_err(|e| QuicheException(e.to_string()))
    }
    fn try_borrow_mut_q<'a>(&'a self) -> Result<std::cell::RefMut<'a, T>, QuicheException> {
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

/// Blanket impl for mutable references - returns clone of inner value
impl<T: Clone> QuicheDeref for &mut T {
    type Target = T;
    fn quiche_deref(&self) -> T {
        (*self).clone()
    }
}

/// Blanket impl for immutable references - returns clone of inner value
impl<T: Clone> QuicheDeref for &T {
    type Target = T;
    fn quiche_deref(&self) -> T {
        (*self).clone()
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
