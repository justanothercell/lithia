use std::collections::HashMap;
use crate::{BinBuilder, JmpType, Stmt};
use crate::compiler::ast::Expr;
use crate::compiler::bin_builder::VarId;
use crate::variable::Type;

pub(crate) struct Compiler{
    needed_externs_raw: HashMap<String, (Vec<Type>, Vec<Type>)>,
    needed_externs: HashMap<String, VarId>,
    variables: HashMap<String, VarId>,
}

impl Compiler {
    pub(crate) fn new(needed_externs: HashMap<String, (Vec<Type>, Vec<Type>)>) -> Self {
        Compiler {
            needed_externs_raw: needed_externs,
            needed_externs: HashMap::new(),
            variables: HashMap::new()
        }
    }

    pub(crate) fn compile(mut self, ast: Expr) -> Vec<u8> {
        let mut builder = BinBuilder::new();
        for (name, (args, ret)) in self.needed_externs_raw.clone() {
            let id = builder.gen_var_id();
            builder.load_extern(&name, &id, Type::Fn(args, ret));
            self.needed_externs.insert(name, id);
        }
        self.compile_ast(ast, &mut builder);
        builder.return_scope(|_builder|{});
        builder.build()
    }

    fn compile_ast(&mut self, ast: Expr, builder: &mut BinBuilder) {
        match ast {
            Expr::Call(func) => {
                let fn_id = self.needed_externs.get(&func.ident.0).expect("No extern function with this name").clone();
                let mut args = func.args.clone();
                while let Some(expr) = args.pop() {
                    self.compile_ast(expr, builder);
                }
                builder.call_function(&fn_id, |_builder| {});
            }
            Expr::Stmts(stmts, expr, _type) => {
                for stmt in stmts {
                    self.compile_stmt(stmt, builder);
                }
                if let Some(box e) = expr {
                    self.compile_ast(e, builder);
                }
            },
            Expr::Variable(var) => {
                let var_id = self.variables.get(&var.0).expect(&format!("No variable with ident {:?}", var));
                builder.push_variable(var_id);
            }
            Expr::Value(val) => {
                builder.push_value(val);
            }
            Expr::LoopWhile(box cond, box body) => {
                let loop_start = builder.gen_marker();
                let loop_end = builder.gen_marker();
                builder.set_marker(&loop_start);
                self.compile_ast(cond, builder);
                builder.jump(JmpType::Unless, &loop_end);
                self.compile_ast(body, builder);
                builder.jump(JmpType::Jmp, &loop_start);
                builder.set_marker(&loop_end);
            }
            Expr::Empty => {}
        }
    }

    fn compile_stmt(&mut self, stmt: Stmt, builder: &mut BinBuilder){
        match stmt {
            Stmt::Create(var, expr) => {
                let var_id = builder.gen_var_id();
                self.compile_ast(expr, builder);
                builder.set_var(&var_id, |_builder| {});
                let _ = self.variables.insert(var.0.clone(), var_id);
            }
            Stmt::Delete(var) => {
                self.variables.remove(&var.0.clone()).expect(&format!("No variable with ident {:?}", var));
            }
            Stmt::Assign(var, expr) => {
                let var_id = self.variables.get(&var.0).expect(&format!("No variable with ident {:?}", var)).clone();
                self.compile_ast(expr, builder);
                builder.set_var(&var_id, |_builder| {});
            }
            Stmt::Expr(expr) => {
                self.compile_ast(expr, builder);
            }
            Stmt::Return(expr) => {
                self.compile_ast(expr, builder);
                builder.return_scope(|_builder| {});
            }
        }
    }
}
