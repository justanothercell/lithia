pub(crate) mod parser;
pub(crate) mod patterns;
pub(crate) mod code_printer;
pub(crate) mod create_patterns;

use std::collections::HashMap;
use std::fmt::Debug;
use crate::error::ParseError;
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
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AstLiteral(pub(crate) Literal, pub(crate) Span);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Expression(pub(crate) Expr, pub(crate) Span);
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Expr {
    Point(Box<Expression>),
    Literal(AstLiteral),
    Variable(Ident),
    Block(Block),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>),
    VarCreate(Item, bool, Option<Type>, Box<Expression>),
    VarAssign(Item, Option<Operator>, Box<Expression>),
    Return(Option<Box<Expression>>),
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
    LShift,
    RShift,
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
    Pointer(Box<Type>),
    Array(Box<Type>, usize),
    Tuple(Vec<Type>),
    Signature(Vec<Type>, Box<Type>)
}
impl Ty {
    pub(crate) fn empty() -> Self{
        Ty::Tuple(vec![])
    }
    pub(crate) fn is_empty(&self) -> bool {
        if let Ty::Tuple(ty) = self {
            ty.len() == 0
        } else {
            false
        }
    }
}

impl AstLiteral {
    pub(crate) fn get_type(&self) -> Result<Type, ParseError>{
        Ok(match &self.0 {
            Literal::String(s) => Type(Ty::Array(Box::new(Type(Ty::Single(vec![], Item::new(&vec!["u8"], self.1.clone())), self.1.clone())), s.len() + 1), self.1.clone()),
            Literal::Char(_) => Type(Ty::Single(vec![], Item::new(&vec!["u8"], self.1.clone())), self.1.clone()),
            Literal::Number(_, ty) => unimplemented!(),
            Literal::Bool(_) => Type(Ty::Single(vec![], Item::new(&vec!["bool"], self.1.clone())), self.1.clone()),
            Literal::Array(_, elem_ty, len) =>  Type(Ty::Array(Box::new(elem_ty.clone()), *len), self.1.clone())
        })
    }
}