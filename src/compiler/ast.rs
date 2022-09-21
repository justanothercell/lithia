use std::collections::HashMap;
use std::fmt::Debug;
use crate::variable::{Ident, Type, Value};

#[derive(Debug, Clone)]
pub(crate) struct FuncCall {
    pub(crate) name: Ident,
    pub(crate) args: Vec<Expr>
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Call(FuncCall),
    Stmts(Vec<Stmt>, Option<Box<Expr>>, Type),
    Variable(Ident),
    Value(Value),
    LoopWhile(Expr, Expr),
    Empty
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Create(Ident),
    Delete(Ident),
    Assign(Ident, Expr),
    Call(FuncCall),
    Return(Expr)
}