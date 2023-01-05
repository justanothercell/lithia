pub(crate) mod parser;
pub(crate) mod patterns;
pub(crate) mod code_printer;
pub(crate) mod create_patterns;

use std::collections::HashMap;
use std::fmt::Debug;
use crate::source::span::Span;
use crate::tokens::Literal;

#[derive(Debug, Clone)]
pub(crate) struct Ident(pub(crate) String, pub(crate) Span);

#[derive(Debug, Clone)]
pub(crate) struct Item(pub(crate) Vec<Ident>, pub(crate) Span);

#[derive(Debug, Clone)]
pub(crate) struct AstLiteral(pub(crate) Literal, pub(crate) Span);

#[derive(Debug, Clone)]
pub(crate) struct Expression(pub(crate) Expr, pub(crate) Span);
#[derive(Debug, Clone)]
pub(crate) enum Expr {
    Literal(AstLiteral),
    Variable(Ident),
    Block(Block),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>),
    VarCreate(Item, bool, Option<FullType>, Box<Expression>),
    VarAssign(Item, Option<Operator>, Box<Expression>)
}


#[derive(Debug, Clone)]
pub(crate) struct Operator(pub(crate) Op, pub(crate) Span);
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub(crate) struct Statement(pub(crate) Expression, pub(crate) bool, pub(crate) Span);

#[derive(Debug, Clone)]
pub(crate) struct Module{
    pub(crate) name: Ident,
    pub(crate) sub_modules: HashMap<String, Module>,
    pub(crate) functions: HashMap<String, Func>,
    pub(crate) loc: Span
}

#[derive(Debug, Clone)]
pub(crate) struct Block(pub(crate) Vec<Statement>, pub(crate) Span);

#[derive(Debug, Clone)]
pub(crate) struct Func {
    pub(crate) name: Ident,
    pub(crate) args: Vec<(Ident, FullType)>,
    pub(crate) signature: FullType,
    pub(crate) body: Option<Block>,
    pub(crate) loc: Span
}

#[derive(Debug, Clone)]
pub(crate) struct FullType(pub(crate) TypeT, pub(crate) Span);
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub(crate) struct Type {
    pub(crate) generics: Vec<FullType>,
    pub(crate) base_type: Item,
    pub(crate) loc: Span
}
