use quiche_parser::ast::QuicheModule;
use quiche_parser::ast::Stmt;
use quiche_parser::ast::Expr;
use quiche_parser::ast::FunctionDef;
use quiche_parser::ast::Arg;
use quiche_parser::ast::Operator;
use quiche_parser::ast::BoolOperator;
use quiche_parser::ast::UnaryOperator;
use quiche_parser::ast::IfStmt;
use quiche_parser::ast::WhileStmt;
use quiche_parser::ast::ForStmt;
use std::collections::HashMap;
pub use crate::create_QuicheModule as create_QuicheModule;
pub use crate::box_expr as box_expr;
pub use crate::ast_wrap_mutref_type as ast_wrap_mutref_type;
pub use crate::ast_wrap_mutref_call as ast_wrap_mutref_call;
pub use crate::ast_cast_usize as ast_cast_usize;
pub use crate::ast_get_func_name as ast_get_func_name;
pub use crate::ast_update_func_body as ast_update_func_body;
pub use crate::ast_update_arg_annotation as ast_update_arg_annotation;
pub use crate::create_Transformer as create_Transformer;
#[derive(Clone, Debug, Default)]
pub struct Transformer  {
pub signatures: std::collections::HashMap<String, Vec<bool>>,
}

impl Transformer {
pub fn new() -> Transformer {
return create_Transformer();
}

pub fn transform_module(&mut self, module: QuicheModule) -> QuicheModule {
println!("{}", String::from("Transformer: Analyzing signatures..."));
self.signatures = std::collections::HashMap::new();
let mut pass1_body: Vec<Stmt> = vec![];
for __q in (module.body) {
let stmt = __q;
pass1_body.push(self.visit_def(stmt));
}
println!("{}", String::from("Transformer: Transforming calls..."));
let mut final_body: Vec<Stmt> = vec![];
for __q in (pass1_body) {
let stmt = __q;
final_body.push(self.transform_stmt(stmt));
}
return create_QuicheModule(final_body);
}

pub fn visit_def(&mut self, stmt: Stmt) -> Stmt {
match stmt {
Stmt::FunctionDef(f) => {
let mut new_args: Vec<Arg> = vec![];
let mut signature: Vec<bool> = vec![];
let mut func_name = ast_get_func_name(crate::quiche::qref!(f));
for __q in (f.args.clone()) {
let arg = __q;
let mut new_arg = arg.clone();
let mut is_complex = false;
match arg.annotation {
Some(ann) => {
let mut inner = crate::quiche::deref!(ann).clone();
if self.is_complex_type(inner.clone()) {
is_complex = true;
let mut wrapped = ast_wrap_mutref_type(box_expr(inner));
new_arg = ast_update_arg_annotation(new_arg, Some(wrapped));
}
}
None => {
}
}
signature.push(is_complex);
new_args.push(new_arg);
}
self.signatures.insert(func_name, signature);
let mut updated_f = self.update_args(f, new_args);
return Stmt::FunctionDef(updated_f);
}
_ => {
return stmt;
}
}
}

pub fn update_args(&mut self, f: FunctionDef, args: Vec<Arg>) -> FunctionDef {
return ast_set_func_args(f, args);
}

pub fn is_complex_type(&mut self, e: Expr) -> bool {
match e {
Expr::Name(n) => {
return ((n == String::from("Vec")) || (n == String::from("String"))) || ((n.len() > 0) && (self.is_uppercase(n)));
}
Expr::Subscript { value: val_box, slice: slice_box, .. } => {
return self.is_complex_type(crate::quiche::deref!(val_box).clone());
}
_ => {
return false;
}
}
}

pub fn is_uppercase(&mut self, s: String) -> bool {
return check_first_upper(s);
}

pub fn transform_stmt(&mut self, stmt: Stmt) -> Stmt {
match stmt {
Stmt::Expr(e) => {
let mut new_expr = self.transform_expr(crate::quiche::deref!(e).clone());
return Stmt::Expr(box_expr(new_expr));
}
Stmt::If(i) => {
let mut test = self.transform_expr(crate::quiche::deref!(i.test).clone());
let mut body: Vec<Stmt> = vec![];
for __q in (i.body) {
let s = __q;
body.push(self.transform_stmt(s));
}
let mut orelse: Vec<Stmt> = vec![];
for __q in (i.orelse) {
let s = __q;
orelse.push(self.transform_stmt(s));
}
return make_if_stmt(box_expr(test), body, orelse);
}
Stmt::While(w) => {
let mut test = self.transform_expr(crate::quiche::deref!(w.test).clone());
let mut body: Vec<Stmt> = vec![];
for __q in (w.body) {
let s = __q;
body.push(self.transform_stmt(s));
}
let mut orelse: Vec<Stmt> = vec![];
for __q in (w.orelse) {
let s = __q;
orelse.push(self.transform_stmt(s));
}
return make_while_stmt(box_expr(test), body, orelse);
}
Stmt::For(f) => {
let mut target = f.target.clone();
let mut iter = self.transform_expr(crate::quiche::deref!(f.iter).clone());
let mut body: Vec<Stmt> = vec![];
for __q in (f.body) {
let s = __q;
body.push(self.transform_stmt(s));
}
let mut orelse: Vec<Stmt> = vec![];
for __q in (f.orelse) {
let s = __q;
orelse.push(self.transform_stmt(s));
}
return make_for_stmt(target, box_expr(iter), body, orelse);
}
Stmt::FunctionDef(f) => {
let mut new_body: Vec<Stmt> = vec![];
for __q in (f.body.clone()) {
let s = __q;
new_body.push(self.transform_stmt(s));
}
return Stmt::FunctionDef(ast_update_func_body(f, new_body));
}
Stmt::Return(r) => {
match r {
Some(e) => {
let mut inner = crate::quiche::deref!(e).clone();
return Stmt::Return(Some(box_expr(self.transform_expr(inner))));
}
None => {
return Stmt::Return(None);
}
}
}
Stmt::Assign(a) => {
let mut new_targets: Vec<Expr> = vec![];
for __q in (a.targets) {
let t = __q;
new_targets.push(self.transform_expr(t));
}
let mut val_inner = crate::quiche::deref!(a.value).clone();
let mut new_val = self.transform_expr(val_inner);
return Stmt::Assign(ast_create_assign(new_targets, box_expr(new_val)));
}
Stmt::If(i) => {
let mut test = self.transform_expr(crate::quiche::deref!(i.test).clone());
let mut body: Vec<Stmt> = vec![];
for __q in (i.body) {
let s = __q;
body.push(self.transform_stmt(s));
}
let mut orelse: Vec<Stmt> = vec![];
for __q in (i.orelse) {
let s = __q;
orelse.push(self.transform_stmt(s));
}
return Stmt::If(ast_create_if(box_expr(test), body, orelse));
}
Stmt::For(f) => {
let mut target = self.transform_expr(crate::quiche::deref!(f.target).clone());
let mut iter = self.transform_expr(crate::quiche::deref!(f.iter).clone());
let mut body: Vec<Stmt> = vec![];
for __q in (f.body) {
let s = __q;
body.push(self.transform_stmt(s));
}
let mut orelse: Vec<Stmt> = vec![];
for __q in (f.orelse) {
let s = __q;
orelse.push(self.transform_stmt(s));
}
return Stmt::For(ast_create_for(box_expr(target), box_expr(iter), body, orelse));
}
_ => {
return stmt;
}
}
}

pub fn transform_expr(&mut self, expr: Expr) -> Expr {
match expr {
Expr::Call { func: func_box, args: c_args, keywords: ckw, .. } => {
let mut func_expr = crate::quiche::deref!(func_box).clone();
let mut func_name = String::from("");
let mut has_name = false;
match func_expr {
Expr::Name(n) => {
func_name = n.clone();
has_name = true;
func_expr = Expr::Name(n);
}
_ => {
has_name = false;
}
}
let mut new_args: Vec<Expr> = vec![];
let mut sig: Vec<bool> = vec![];
if (has_name) && (self.signatures.contains_key(crate::quiche::qref!(func_name))) {
sig = self.signatures.get(crate::quiche::qref!(func_name)).unwrap().clone();
}
let mut idx = 0;
for __q in (c_args) {
let arg = __q;
let mut new_arg = self.transform_expr(arg);
if idx < sig.len() {
let mut needs_borrow = sig[idx].clone();
if needs_borrow {
new_arg = crate::quiche::deref!(ast_wrap_mutref_call(box_expr(new_arg)));
}
}
new_args.push(new_arg);
idx = idx + 1;
}
return ast_create_call(box_expr(func_expr), new_args);
}
Expr::Subscript { value: val_box, slice: slice_box, .. } => {
let mut val = self.transform_expr(crate::quiche::deref!(val_box).clone());
let mut slice = self.transform_expr(crate::quiche::deref!(slice_box).clone());
let mut new_slice = crate::quiche::deref!(ast_cast_usize(box_expr(slice)));
return ast_create_subscript(box_expr(val), box_expr(new_slice));
}
Expr::BinOp { left: l, op: o, right: r, .. } => {
let mut new_l = self.transform_expr(crate::quiche::deref!(l).clone());
let mut new_r = self.transform_expr(crate::quiche::deref!(r).clone());
return ast_create_binop(box_expr(new_l), o, box_expr(new_r));
}
Expr::UnaryOp { op: o, operand: opd, .. } => {
let mut new_opd = self.transform_expr(crate::quiche::deref!(opd).clone());
return ast_create_unaryop(o, box_expr(new_opd));
}
Expr::BoolOp { op: o, values: vals, .. } => {
let mut new_vals: Vec<Expr> = vec![];
for __q in (vals) {
let v = __q;
new_vals.push(self.transform_expr(v));
}
return ast_create_boolop(o, new_vals);
}
Expr::List(elts) => {
let mut new_elts: Vec<Expr> = vec![];
for __q in (elts) {
let e = __q;
new_elts.push(self.transform_expr(e));
}
return ast_create_list(new_elts);
}
Expr::Name(n) => {
return Expr::Name(n);
}
Expr::Constant(c) => {
return Expr::Constant(c);
}
_ => {
return expr;
}
}
}

}

pub use crate::ast_create_binop as ast_create_binop;
pub use crate::ast_create_unaryop as ast_create_unaryop;
pub use crate::ast_create_boolop as ast_create_boolop;
pub use crate::ast_create_list as ast_create_list;
pub use crate::make_if_stmt as make_if_stmt;
pub use crate::make_while_stmt as make_while_stmt;
pub use crate::make_for_stmt as make_for_stmt;
pub use crate::ast_set_func_args as ast_set_func_args;
pub use crate::check_first_upper as check_first_upper;
pub use crate::ast_create_assign as ast_create_assign;
pub use crate::ast_create_if as ast_create_if;
pub use crate::ast_create_for as ast_create_for;
pub use crate::ast_create_call as ast_create_call;
pub use crate::ast_create_subscript as ast_create_subscript;
pub fn transform_module(module: QuicheModule) -> QuicheModule {
let mut t = create_Transformer();
return t.transform_module(module);
}

