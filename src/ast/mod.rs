pub(crate) mod parser;
pub(crate) mod patterns;
pub(crate) mod code_printer;

use std::collections::HashMap;
use std::fmt::Debug;
use crate::ast::code_printer::CodePrinter;
use crate::source::span::Span;
use crate::tokens::Literal;

pub(crate) type NamedMap<T> = HashMap<String, T>;

#[derive(Debug)]
pub(crate) struct Ident(pub(crate) String, pub(crate) Span);

#[derive(Debug)]
pub(crate) struct Item(pub(crate) Vec<Ident>, pub(crate) Span);

#[derive(Debug)]
pub(crate) struct AstLiteral(pub(crate) Literal, pub(crate) Span);

#[derive(Debug)]
pub(crate) struct Expression(pub(crate) Expr, pub(crate) Span);
#[derive(Debug)]
pub(crate) enum Expr {
    Literal(AstLiteral),
    Variable(Ident),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>)
}


#[derive(Debug)]
pub(crate) struct Operator(pub(crate) Op, pub(crate) Span);
#[derive(Debug)]
pub(crate) enum Op {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Not,
    LShift,
    RShift,
}

#[derive(Debug)]
pub(crate) struct Statement(pub(crate) Stmt, pub(crate) Span);
#[derive(Debug)]
pub(crate) enum Stmt {
    Expression(Expression),
    VarCreate(Item, Self::mutable, Option<FullType>, Expression),
    VarAssign(Item, Option<Operator>, Expression)
}
impl Stmt {
    type mutable = bool;
}

#[derive(Debug)]
pub(crate) struct Module{
    pub(crate) name: Ident,
    pub(crate) sub_modules: NamedMap<Module>,
    pub(crate) functions: NamedMap<Func>,
    pub(crate) loc: Span
}

#[derive(Debug)]
pub(crate) struct Block(pub(crate) Vec<Statement>, pub(crate) Span);

#[derive(Debug)]
pub(crate) struct Func {
    pub(crate) name: Ident,
    pub(crate) args: Vec<(Ident, FullType)>,
    pub(crate) ret: FullType,
    pub(crate) body: Block,
    pub(crate) loc: Span
}

#[derive(Debug)]
pub(crate) struct FullType(pub(crate) TypeT, pub(crate) Span);
#[derive(Debug)]
pub(crate) enum TypeT {
    Single(Type),
    Tuple(Vec<FullType>),
    Signature(Vec<FullType>, Box<FullType>)
}
impl TypeT {
    pub(crate) fn empty() -> Self{
        TypeT::Tuple(vec![])
    }
    pub(crate) fn is_empty(&self) -> bool {
        if let TypeT::Tuple(ty) = self {
            ty.len() == 0
        } else {
            false
        }
    }
}
#[derive(Debug)]
pub(crate) struct Type {
    pub(crate) generics: Vec<FullType>,
    pub(crate) base_type: Item,
    pub(crate) loc: Span
}