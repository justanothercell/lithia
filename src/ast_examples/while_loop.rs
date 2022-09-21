use crate::{Expr, FuncCall, Ident, Stmt};
use crate::variable::{Type, Value};

pub(crate) fn example() -> Expr {
    let var_i = Ident("i".to_string());

    let exec = Expr::Stmts(vec![
        Stmt::Create(var_a.clone()),
        Stmt::Assign(var_a.clone(), Expr::Value(Value::I32(42))),
        Stmt::Create(var_b.clone()),
        Stmt::Assign(var_b.clone(), Expr::Value(Value::I32(69))),
        Stmt::Create(var_res.clone()),
        Stmt::Assign(var_res.clone(), Expr::Call(FuncCall{
            name: Ident("i32::add".to_string()),
            args: vec![Expr::Variable(var_a), Expr::Variable(var_b)]
        })),
        Stmt::Return(Expr::Variable(var_res))
    ], None, Type::Empty);
    println!("{:#?}", exec);
}