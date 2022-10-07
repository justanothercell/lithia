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
    Call(FuncCall, Option<Type>, Loc),
    Stmts(Vec<Stmt>, Option<Box<Expr>>, Option<Type>, Loc),
    Variable(Ident, Option<Type>, Loc),
    Value(Value, Option<Type>, Loc),
    While(Box<Expr>, Box<Expr>, Option<Type>, Loc),
    If(Box<Expr>, Box<Expr>, Box<Expr>, Option<Type>, Loc),
    Empty(Loc)
}

impl Expr {
    pub(crate) fn loc(&self) -> &Loc {
        match self {
            Expr::Call(_, _, loc) => loc,
            Expr::Stmts(_, _, _, loc) => loc,
            Expr::Variable(_, _, loc) => loc,
            Expr::Value(_, _, loc) => loc,
            Expr::While(_, _, _, loc) => loc,
            Expr::If(_, _, _, _, loc) => loc,
            Expr::Empty(loc) => loc,
        }
    }

    pub(crate) fn get_type(&self) -> &Option<Type>{
        match self {
            Expr::Call(_, t, _) => t,
            Expr::Stmts(_, _, t, _) => t,
            Expr::Variable(_, t, _) => t,
            Expr::Value(_, t, _) => t,
            Expr::While(_, _, t, _) => t,
            Expr::If(_, _, _, t, _) => t,
            Expr::Empty(_) => &Some(Type::Empty),
        }
    }

    pub(crate) fn set_type(&mut self, ty: Option<Type>){
        match self {
            Expr::Call(_, t, _) => *t = ty,
            Expr::Stmts(_, _, t, _) => *t = ty,
            Expr::Variable(_, t, _) => *t = ty,
            Expr::Value(_, t, _) => *t = ty,
            Expr::While(_, _, t, _) => *t = ty,
            Expr::If(_, _, _, t, _) => *t = ty,
            Expr::Empty(_) => (),
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
            Expr::Call(func_call, _, _) => format!("{}({})", func_call.ident.0, func_call.args.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join(", ")),
            Expr::Stmts(stmts, expr, _, _) => {
                if let Some(e) = expr {
                    format!("{{\n{}\n{}\n}}", stmts.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n"), e)
                }
                else{
                    format!("{{\n{}\n}}", stmts.iter().map(|s| format!("{}", s)).collect::<Vec<String>>().join("\n"))
                }
            },
            Expr::Variable(var, _, _) => format!("{}", var.0),
            Expr::Value(val, _, _) => format!("{:?}", val),
            Expr::While(cond, body, _, _) => format!("while {} {{\n{}\n}}", cond, textwrap::indent(&format!("{}", body), "    ")),
            Expr::If(cond, body_if, body_else, _, _) => format!("if {} {{\n{}\n}}\nelse {{\n{}\n}}", cond, textwrap::indent(&format!("{}", body_if), "    "), textwrap::indent(&format!("{}", body_else), "    ")),
            Expr::Empty(_) => format!("")
        })
    }
}