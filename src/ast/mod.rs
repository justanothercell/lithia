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
    VarCreate(Item, bool, Option<Type>, Box<Expression>),
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
    pub(crate) constants: HashMap<String, Const>,
    pub(crate) loc: Span
}

#[derive(Debug, Clone)]
pub(crate) struct Block(pub(crate) Vec<Statement>, pub(crate) Span);

#[derive(Debug, Clone)]
pub(crate) struct Func {
    pub(crate) name: Ident,
    pub(crate) args: Vec<(Ident, Type)>,
    pub(crate) ret: Type,
    pub(crate) body: Option<Block>,
    pub(crate) loc: Span
}

#[derive(Debug, Clone)]
pub(crate) struct Const {
    pub(crate) name: Ident,
    pub(crate) ty: Type,
    pub(crate) val: Expression
}

#[derive(Debug, Clone)]
pub(crate) struct Type(pub(crate) Ty, pub(crate) Span);
#[derive(Debug, Clone)]
pub(crate) enum Ty {
    Single{
        generics: Vec<Type>,
        base_type: Item,
        loc: Span
    },
    Pointer(Box<Type>),
    Array(Box<Type>),
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
