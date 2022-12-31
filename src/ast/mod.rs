use std::collections::HashMap;
use crate::source::span::Span;

pub(crate) type NamedMap<T> = HashMap<String, T>;

struct Expression(pub(crate) Expr, pub(crate) Span);
enum Expr {
    Literal(AstLiteral),
    Variable(Ident),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>)
}

struct Operator(pub(crate) Op, pub(crate) Span);
enum Op {
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

struct Statement(pub(crate) Stmt, pub(crate) Span);
enum Stmt {
    Expression(Expression),
    VarCreate(Item, Self::mutable, Option<FullType>, Expression),
    VarAssign(Item, Option<Operator>, Expression)
}
impl Stmt {
    type mutable = bool;
}

struct Module{
    pub(crate) name: Ident,
    pub(crate) sub_modules: NamedMap<Module>,
    pub(crate) functions: NamedMap<Func>,
    pub(crate) loc: Span
}