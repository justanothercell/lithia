pub(crate) mod parser;

use std::collections::HashMap;
use crate::source::span::Span;
use crate::tokens::Literal;

pub(crate) type NamedMap<T> = HashMap<String, T>;

pub(crate) struct Ident(pub(crate) String, pub(crate) Span);
pub(crate) struct Item(pub(crate) Vec<Ident>, pub(crate) Span);

pub(crate) struct AstLiteral(pub(crate) Literal, pub(crate) Span);

pub(crate) struct Expression(pub(crate) Expr, pub(crate) Span);
pub(crate) enum Expr {
    Literal(AstLiteral),
    Variable(Ident),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>)
}

pub(crate) struct Operator(pub(crate) Op, pub(crate) Span);
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

pub(crate) struct Statement(pub(crate) Stmt, pub(crate) Span);
pub(crate) enum Stmt {
    Expression(Expression),
    VarCreate(Item, Self::mutable, Option<FullType>, Expression),
    VarAssign(Item, Option<Operator>, Expression)
}
impl Stmt {
    type mutable = bool;
}

pub(crate) struct Module{
    pub(crate) name: Ident,
    pub(crate) sub_modules: NamedMap<Module>,
    pub(crate) functions: NamedMap<Func>,
    pub(crate) loc: Span
}
pub(crate) struct Block(pub(crate) Vec<Statement>, pub(crate) Span);
pub(crate) struct Func {
    pub(crate) name: Ident,
    pub(crate) args: Vec<(Ident, FullType)>,
    pub(crate) ret: FullType,
    pub(crate) body: Block,
    pub(crate) loc: Span
}

pub(crate) struct FullType(pub(crate) TypeT, pub(crate) Span);
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
pub(crate) struct Type {
    pub(crate) generics: Vec<FullType>,
    pub(crate) base_type: Item,
    pub(crate) loc: Span
}