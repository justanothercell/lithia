pub(crate) mod parser;
pub(crate) mod patterns;
pub(crate) mod code_printer;
pub(crate) mod create_patterns;
pub(crate) mod types_impl;

use std::collections::HashMap;
use std::fmt::Debug;
use crate::error::LithiaError;
use crate::source::span::Span;
use crate::tokens::Literal;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Ident(pub(crate) String, pub(crate) Span);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Item(pub(crate) Vec<Ident>, pub(crate) Span);
impl Item{
    pub(crate) fn new(parts: &Vec<&str>, loc: Span) -> Self {
        Self(parts.iter().map(|p| Ident(p.to_string(), loc.clone())).collect(), loc.clone())
    }

    pub(crate) fn placeholder(loc: Span) -> Self{
        Self::new(&vec!["_"], loc)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Tag(pub(crate) Ident, pub(crate) Vec<TagValue>, pub(crate) Span);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TagValue {
    Lit(AstLiteral),
    Ident(Ident),
    Tag(Box<Tag>)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AstLiteral(pub(crate) Literal, pub(crate) Span);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Expression(pub(crate) HashMap<String, Tag>, pub(crate) Expr, pub(crate) Span);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Point(Box<Expression>),
    Deref(Box<Expression>),
    Cast(Box<Expression>, Type),
    Literal(AstLiteral),
    Variable(Ident),
    Block(Block),
    Expr(Box<Expression>),
    If(Box<Expression>, Block, Block),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>),
    VarCreate(Ident, bool, Option<Type>, Box<Expression>),
    VarAssign(Ident, Option<Operator>, Box<Expression>),
    Return(Option<Box<Expression>>),
}

impl Expr {
    pub(crate) fn is_block_like(&self) -> bool {
        match self {
            Expr::Point(_) => false,
            Expr::Deref(_) => false,
            Expr::Cast(_, _) => false,
            Expr::Literal(_) => false,
            Expr::Variable(_) => false,
                Expr::Block(_) => true,
            Expr::Expr(_) => false,
                Expr::If(_, _, _) => true,
            Expr::FuncCall(_, _) => false,
            Expr::BinaryOp(_, _, _) => false,
            Expr::UnaryOp(_, _) => false,
            Expr::VarCreate(_, _, _, _) => false,
            Expr::VarAssign(_, _, _) => false,
            Expr::Return(_) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Operator(pub(crate) Op, pub(crate) Span);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Op {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Not,
    BinOr,
    BinAnd,
    LShift,
    RShift,
    LT,
    LE,
    GT,
    GE,
    EQ,
    NE,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Statement(pub(crate) Expression, pub(crate) bool, pub(crate) Span);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Module{
    pub(crate) name: Ident,
    pub(crate) sub_modules: HashMap<String, Module>,
    pub(crate) functions: HashMap<String, Func>,
    pub(crate) constants: HashMap<String, Const>,
    pub(crate) loc: Span
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Block(pub(crate) Vec<Statement>, pub(crate) Span);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Func {
    pub(crate) tags: HashMap<String, Tag>,
    pub(crate) name: Ident,
    pub(crate) args: Vec<(Ident, Type)>,
    pub(crate) ret: Type,
    pub(crate) body: Option<Block>,
    pub(crate) loc: Span
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Const {
    pub(crate) name: Ident,
    pub(crate) ty: Type,
    pub(crate) val: Expression
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Type(pub(crate) Ty, pub(crate) Span);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Ty {
    Single(Vec<Type>, Item),
    RawPointer,
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Slice(Box<Type>),
    Tuple(Vec<Type>),
    Signature(Vec<Type>, Box<Type>, Self::unsafe_func, Self::vararg_func)
}

impl Ty {
    #[allow(non_camel_case_types)]
    type unsafe_func = bool;
    #[allow(non_camel_case_types)]
    type vararg_func = bool;
}

impl AstLiteral {
    pub(crate) fn get_type(&self) -> Result<Type, LithiaError>{
        Ok(match &self.0 {
            Literal::String(s) => Type(Ty::Array(Box::new(Type(Ty::Single(vec![], Item::new(&vec!["u8"], self.1.clone())), self.1.clone())), s.len() + 1), self.1.clone()),
            Literal::Char(_) => Type(Ty::Single(vec![], Item::new(&vec!["u8"], self.1.clone())), self.1.clone()),
            Literal::Number(_,  Some(ty)) => Type(Ty::Single(vec![], Item::new(&vec![&format!("{ty}")], self.1.clone())), self.1.clone()),
            Literal::Number(_,  None) => unimplemented!(),
            Literal::Bool(_) => Type(Ty::Single(vec![], Item::new(&vec!["bool"], self.1.clone())), self.1.clone()),
            Literal::Array(_, elem_ty, len) =>  Type(Ty::Array(Box::new(elem_ty.clone()), *len), self.1.clone())
        })
    }
}