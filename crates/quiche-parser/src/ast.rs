#[derive(Debug, Clone)]
pub struct QuicheModule {
    pub body: Vec<QuicheStmt>,
}

// Full AST Proxy Definitions to decouple from Ruff

#[derive(Debug, Clone)]
pub enum QuicheStmt {
    // Native Constructs
    StructDef(StructDef),
    EnumDef(EnumDef),
    TraitDef(TraitDef),
    ImplDef(ImplDef),
    RustBlock(String),

    // Standard Constructs (Proxied)
    FunctionDef(FunctionDef),
    ClassDef(ClassDef),
    Return(Option<Box<QuicheExpr>>),
    Assign(Assign),
    AnnAssign(AnnAssign),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Expr(Box<QuicheExpr>),
    Import(Import),
    ImportFrom(ImportFrom),
    Match(MatchStmt),
    Assert(AssertStmt),
    Pass,
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub enum QuicheExpr {
    BinOp {
        left: Box<QuicheExpr>,
        op: Operator,
        right: Box<QuicheExpr>,
    },
    BoolOp {
        op: BoolOperator,
        values: Vec<QuicheExpr>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<QuicheExpr>,
    },
    Compare {
        left: Box<QuicheExpr>,
        ops: Vec<CmpOperator>,
        comparators: Vec<QuicheExpr>,
    },
    Call {
        func: Box<QuicheExpr>,
        args: Vec<QuicheExpr>,
        keywords: Vec<Keyword>,
    },
    Attribute {
        value: Box<QuicheExpr>,
        attr: String,
    },
    Subscript {
        value: Box<QuicheExpr>,
        slice: Box<QuicheExpr>,
    },
    Name(String),
    Constant(Constant),
    List(Vec<QuicheExpr>),
    Tuple(Vec<QuicheExpr>),
    Lambda {
        args: Vec<String>,
        body: Box<QuicheExpr>,
    }, // Simplified args
    IfExp {
        test: Box<QuicheExpr>,
        body: Box<QuicheExpr>,
        orelse: Box<QuicheExpr>,
    },
    Cast {
        expr: Box<QuicheExpr>,
        target_type: Box<QuicheExpr>,
    },
}

// Support Types
#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Vec<QuicheStmt>,
    pub decorator_list: Vec<QuicheExpr>,
    pub returns: Option<Box<QuicheExpr>>,
    pub type_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub bases: Vec<QuicheExpr>,
    pub body: Vec<QuicheStmt>,
    pub decorator_list: Vec<QuicheExpr>,
    pub type_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Assign {
    pub targets: Vec<QuicheExpr>,
    pub value: Box<QuicheExpr>,
}

#[derive(Debug, Clone)]
pub struct AnnAssign {
    pub target: Box<QuicheExpr>,
    pub annotation: Box<QuicheExpr>,
    pub value: Option<Box<QuicheExpr>>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub test: Box<QuicheExpr>,
    pub body: Vec<QuicheStmt>,
    pub orelse: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub test: Box<QuicheExpr>,
    pub body: Vec<QuicheStmt>,
    pub orelse: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub target: Box<QuicheExpr>,
    pub iter: Box<QuicheExpr>,
    pub body: Vec<QuicheStmt>,
    pub orelse: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub arg: String,
    pub annotation: Option<Box<QuicheExpr>>,
}

#[derive(Debug, Clone)]
pub struct Keyword {
    pub arg: Option<String>,
    pub value: Box<QuicheExpr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Pow,
    LShift,
    RShift,
    BitOr,
    BitXor,
    BitAnd,
    FloorDiv,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BoolOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Invert,
    Not,
    UAdd,
    USub,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CmpOperator {
    Eq,
    NotEq,
    Lt,
    LtE,
    Gt,
    GtE,
    Is,
    IsNot,
    In,
    NotIn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constant {
    NoneVal,
    Bool(bool),
    Str(String),
    Int(i64),
    Float(f64),
    Ellipsis,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<VariantDef>,
}

#[derive(Debug, Clone)]
pub struct VariantDef {
    pub name: String,
    pub fields: Vec<String>, // Tuple-like variant types
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub name: String,
    pub body: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub struct ImplDef {
    pub trait_name: Option<String>,
    pub target_type: String,
    pub body: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub names: Vec<Alias>,
}

#[derive(Debug, Clone)]
pub struct ImportFrom {
    pub module: Option<String>,
    pub names: Vec<Alias>,
    pub level: u32,
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub asname: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub subject: Box<QuicheExpr>,
    pub cases: Vec<MatchCase>,
}

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub guard: Option<Box<QuicheExpr>>,
    pub body: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    MatchValue(Box<QuicheExpr>),
    MatchSingleton(Constant),
    MatchSequence(Vec<Pattern>),
    MatchMapping {
        keys: Vec<Box<QuicheExpr>>,
        patterns: Vec<Pattern>,
        rest: Option<String>,
    },
    MatchClass(MatchClassPattern), // Refactored to struct for cleaner enum
    MatchStar(Option<String>),
    MatchAs {
        pattern: Option<Box<Pattern>>,
        name: Option<String>,
    },
    MatchOr(Vec<Pattern>),
}

#[derive(Debug, Clone)]
pub struct MatchClassPattern {
    pub cls: Box<QuicheExpr>,
    pub patterns: Vec<Pattern>,
    pub kwd_attrs: Vec<String>,
    pub kwd_patterns: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct AssertStmt {
    pub test: Box<QuicheExpr>,
    pub msg: Option<Box<QuicheExpr>>,
}
pub type Expr = QuicheExpr;
pub type Stmt = QuicheStmt;
