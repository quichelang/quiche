use ruff_python_ast as ast;

#[derive(Debug, Clone)]
pub struct QuicheModule {
    pub body: Vec<QuicheStmt>,
}

#[derive(Debug, Clone)]
pub enum QuicheStmt {
    // Native Constructs
    StructDef(StructDef),
    EnumDef(EnumDef),
    TraitDef(TraitDef),
    ImplDef(ImplDef),

    // Standard Constructs (Wrapped/Lowered)
    FunctionDef(ast::StmtFunctionDef),
    ClassDef(ast::StmtClassDef), // Legacy or Python Class
    Stmt(ast::Stmt),             // Fallback for standard statements

    // Rust Injection
    RustBlock(String),
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
