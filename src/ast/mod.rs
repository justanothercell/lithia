use crate::source::span::Span;

enum Expr {
    Literal(AstLiteral),
    Variable(Ident),
    FuncCall(Item, Vec<Expression>),
    BinaryOp(Operator, Box<Expression>, Box<Expression>),
    UnaryOp(Operator, Box<Expression>)
}

struct Operator(pub(crate) Op, pub(crate) Span);