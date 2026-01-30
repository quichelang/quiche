// This crate captures high-level desugaring logic.
// Currently it holds the reference implementation for try/except lowering.

/*
pub fn emit_try(self: &mut Codegen, t: &ast::StmtTry) {
    self.emit("let _quiche_try_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {\n");
    self.enter_var_scope();
    for s in &t.body {
        self.generate_stmt(s);
    }
    self.exit_var_scope();
    self.emit("}));\n");

    self.emit("if let Err(_quiche_err) = _quiche_try_result {\n");
    for handler in &t.handlers {
        if let ast::ExceptHandler::ExceptHandler(inner) = handler {
            if let Some(name) = &inner.name {
                self.emit("let ");
                self.emit(name.as_str());
                self.emit(" = _quiche_err.downcast_ref::<String>().map(|s| s.clone()).or_else(|| _quiche_err.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_else(|| \"Unknown Error\".to_string());\n");
            }
            self.enter_var_scope();
            for stmt in &inner.body {
                self.generate_stmt(stmt);
            }
            self.exit_var_scope();
        }
    }
    self.emit("}\n");
}
*/
