use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use crate::variable::{Ident, Type, Value};

#[derive(Debug, Clone)]
pub(crate) struct FuncCall {
    pub(crate) ident: Ident,
    pub(crate) args: Vec<Expr>
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Call(FuncCall),
    Stmts(Vec<Stmt>, Option<Box<Expr>>, Type),
    Variable(Ident),
    Value(Value),
    LoopWhile(Box<Expr>, Box<Expr>),
    Empty
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Create(Ident, Expr),
    Delete(Ident),
    Assign(Ident, Expr),
    Expr(Expr),
    Return(Expr)
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Stmt::Create(ident, expr) => format!("let {} = {};", ident.0, expr),
            Stmt::Delete(ident) => format!("DEL {};", ident.0),
            Stmt::Assign(ident, expr) => format!("{} = {};", ident.0, expr),
            Stmt::Expr(expr) => format!("{};", expr),
            Stmt::Return(expr) => format!("return {};", expr)
        })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Expr::Call(funcCall) => format!("{}({})", funcCall.ident.0, funcCall.args.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join(", ")),
            Expr::Stmts(stmts, expr, r_type) => {
                if let Some(e) = expr {
                    format!("{{\n{}\n{}\n}}", stmts.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n"), e)
                }
                else{
                    format!("{}", stmts.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n"))
                }
            },
            Expr::Variable(var) => format!("{}", var.0),
            Expr::Value(val) => format!("{:?}", val),
            Expr::LoopWhile(cond, body) => format!("while {} {{\n{}\n}}", cond, body),
            Expr::Empty => format!("")
        })
    }
}