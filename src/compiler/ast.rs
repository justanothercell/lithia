use std::fmt::{Debug, Display, Formatter};
use crate::compiler::compiler::Loc;
use crate::variable::{Ident, Type, Value};

#[derive(Debug, Clone)]
pub(crate) struct FuncCall {
    pub(crate) ident: Ident,
    pub(crate) args: Vec<Expr>
}

#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Call(FuncCall, Loc),
    Stmts(Vec<Stmt>, Option<Box<Expr>>, Type, Loc),
    Variable(Ident, Loc),
    Value(Value, Loc),
    LoopWhile(Box<Expr>, Box<Expr>, Loc),
    Empty(Loc)
}

impl Expr {
    pub(crate) fn loc(&self) -> &Loc {
        match self {
            Expr::Call(_, loc) => loc,
            Expr::Stmts(_, _, _, loc) => loc,
            Expr::Variable(_, loc) => loc,
            Expr::Value(_, loc) => loc,
            Expr::LoopWhile(_, _, loc) => loc,
            Expr::Empty(loc) => loc
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Create(Ident, Expr, Loc),
    Delete(Ident, Loc),
    Assign(Ident, Expr, Loc),
    Expr(Expr, Loc),
    Return(Expr, Loc)
}

impl Stmt {
    pub(crate) fn loc(&self) -> &Loc {
        match self {
            Stmt::Create(_, _, loc) => loc,
            Stmt::Delete(_, loc) => loc,
            Stmt::Assign(_, _, loc) => loc,
            Stmt::Expr(_, loc) => loc,
            Stmt::Return(_, loc) => loc
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Stmt::Create(ident, expr, _) => format!("let {} = {};", ident.0, expr),
            Stmt::Delete(ident, _) => format!("DEL {};", ident.0),
            Stmt::Assign(ident, expr, _) => format!("{} = {};", ident.0, expr),
            Stmt::Expr(expr, _) => format!("{};", expr),
            Stmt::Return(expr, _) => format!("return {};", expr)
        })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Expr::Call(func_call, _) => format!("{}({})", func_call.ident.0, func_call.args.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join(", ")),
            Expr::Stmts(stmts, expr, _type, _) => {
                if let Some(e) = expr {
                    format!("{{\n{}\n{}\n}}", stmts.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n"), e)
                }
                else{
                    format!("{{\n{}\n}}", stmts.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n"))
                }
            },
            Expr::Variable(var, _) => format!("{}", var.0),
            Expr::Value(val, _) => format!("{:?}", val),
            Expr::LoopWhile(cond, body, _) => format!("while {} {{\n{}\n}}", cond, textwrap::indent(&format!("{}", body), "    ")),
            Expr::Empty(_) => format!("")
        })
    }
}