use std::collections::HashMap;
use crate::{Compiler, Expr, FuncCall, Ident, Stmt};
use crate::compiler::compiler::Loc;
use crate::variable::{Type, Value};

#[allow(unused)]
pub(crate) fn example() -> Vec<u8> {
    let i32_lt = Ident("i32::lt".to_string());
    let i32_add = Ident("i32::add".to_string());
    let i32_to_string = Ident("i32::to_string".to_string());
    let string_join = Ident("string::join".to_string());
    let println = Ident("println".to_string());

    // local vars
    let var_i = Ident("i".to_string());

    let ast = Expr::Stmts(vec![
        // i = 0
        Stmt::Create(var_i.clone(), Expr::Value(Value::I32(0),  Some(Type::I32), Loc::dummy()), Loc::dummy()),
        Stmt::Expr(Expr::While(
            // condition: i < 10
            Box::from(Expr::Call(FuncCall {
                ident: i32_lt.clone(),
                args: vec![
                    Expr::Variable(var_i.clone(), Some(Type::I32), Loc::dummy()),
                    Expr::Value(Value::I32(10), Some(Type::I32), Loc::dummy())
                ]
            }, Some(Type::Bool), Loc::dummy())),
            // === body ===
            Box::from(Expr::Stmts(vec![
                Stmt::Expr(Expr::Call(
                    FuncCall {
                        ident: println.clone(),
                        args: vec![
                            Expr::Call(
                                FuncCall {
                                    ident: string_join.clone(),
                                    args: vec![
                                        Expr::Value(Value::String("Counting: ".to_string()), Some(Type::String), Loc::dummy()),
                                        Expr::Call(
                                            FuncCall {
                                                ident: i32_to_string.clone(),
                                                args: vec![
                                                    Expr::Variable(var_i.clone(), Some(Type::String), Loc::dummy())
                                                ]
                                            }, Some(Type::String), Loc::dummy()
                                        )
                                    ]
                                }, Some(Type::String), Loc::dummy()
                            )
                        ]
                    }, Some(Type::Empty), Loc::dummy()
                ), Loc::dummy()),
                // i += 1
                Stmt::Assign(var_i.clone(), Expr::Call(
                    FuncCall {
                        ident: i32_add.clone(),
                        args: vec![
                            Expr::Variable(var_i.clone(), Some(Type::I32), Loc::dummy()),
                            Expr::Value(Value::I32(1), Some(Type::I32), Loc::dummy())
                        ]
                    }, Some(Type::I32), Loc::dummy()
                ), Loc::dummy())
            ], None, Some(Type::Empty), Loc::dummy())), Some(Type::Empty), Loc::dummy()
        ), Loc::dummy())
    ], None, Some(Type::Empty), Loc::dummy());



    println!("{}", ast);

    let compiler = Compiler::new(HashMap::from([
         ("i32::lt".to_string(), (vec![Type::I32, Type::I32], vec![Type::Bool])),
         ("i32::add".to_string(), (vec![Type::I32, Type::I32], vec![Type::I32])),
        ("i32::to_string".to_string(), (vec![Type::I32], vec![Type::String])),
        ("string::join".to_string(), (vec![Type::String, Type::String], vec![Type::String])),
        ("println".to_string(), (vec![Type::String], vec![])),
    ]));
    compiler.compile(ast).expect("Error compiling!")
}