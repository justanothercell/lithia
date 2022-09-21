use std::collections::HashMap;
use crate::compiler::ast::Expr;
use crate::variable::{Ident, Value};

struct Compiler{
    pub(crate) externs: HashMap<Ident, Value>,
}

impl Compiler {
    pub(crate) fn compile(&self, ast: Expr) -> Vec<u8> {
        match ast {
            Expr::Call(fun) => {}
            Expr::Stmts(_, _, _) => {}
            Expr::Variable(_) => {}
            Expr::Value(_) => {}
            Expr::Empty => {}
        }
    }
}
