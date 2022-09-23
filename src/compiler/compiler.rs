use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
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

    pub(crate) fn compile(mut self, ast: Expr) -> Result<Vec<u8>, ParseError> {
        let mut builder = BinBuilder::new();
        for (name, (args, ret)) in self.needed_externs_raw.clone() {
            let id = builder.gen_var_id();
            builder.load_extern(&name, &id, Type::Fn(args, ret));
            self.needed_externs.insert(name, id);
        }
        self.compile_ast(ast, &mut builder)?;
        builder.return_scope(|_builder|{});
        Ok(builder.build())
    }

    fn compile_ast(&mut self, ast: Expr, builder: &mut BinBuilder) -> Result<(), ParseError> {
        match ast {
            Expr::Call(func, loc) => {
                let fn_id = if let Some(fn_id) = self.variables.get(&func.ident.0) {
                    fn_id.clone()
                }
                else {
                    return Err(loc.error(format!("No extern function with name '{}'", func.ident.0)));
                };
                let mut args = func.args.clone();
                while let Some(expr) = args.pop() {
                    self.compile_ast(expr, builder)?;
                }
                builder.call_function(&fn_id, |_builder| {});
            }
            Expr::Stmts(stmts, expr, _type, _) => {
                for stmt in stmts {
                    self.compile_stmt(stmt, builder)?;
                }
                if let Some(box e) = expr {
                    self.compile_ast(e, builder)?;
                }
            },
            Expr::Variable(var, loc) => {
                if let Some(var_id) = self.variables.get(&var.0) {
                    builder.push_variable(var_id);
                }
                else{
                    return Err(loc.error(format!("No variable with ident '{}'", var.0)))
                }
            }
            Expr::Value(val, _) => {
                builder.push_value(val);
            }
            Expr::LoopWhile(box cond, box body, _) => {
                let loop_start = builder.gen_marker();
                let loop_end = builder.gen_marker();
                builder.set_marker(&loop_start);
                self.compile_ast(cond, builder)?;
                builder.jump(JmpType::Unless, &loop_end);
                self.compile_ast(body, builder)?;
                builder.jump(JmpType::Jmp, &loop_start);
                builder.set_marker(&loop_end);
            }
            Expr::Empty(_) => {}
        }
        Ok(())
    }

    fn compile_stmt(&mut self, stmt: Stmt, builder: &mut BinBuilder) -> Result<(), ParseError>{
        match stmt {
            Stmt::Create(var, expr, _) => {
                let var_id = builder.gen_var_id();
                self.compile_ast(expr, builder)?;
                builder.set_var(&var_id, |_builder| {});
                let _ = self.variables.insert(var.0.clone(), var_id);
            }
            Stmt::Delete(var, loc) => {
                if let None = self.variables.remove(&var.0.clone()) {
                    return Err(loc.error(format!("No variable with ident '{}'", var.0)))
                }
            }
            Stmt::Assign(var, expr, loc) => {
                let var_id = if let Some(var_id) = self.variables.get(&var.0){
                    var_id.clone()
                }
                else{
                    return Err(loc.error(format!("No variable with ident {}", var.0)))
                };
                self.compile_ast(expr, builder)?;
                builder.set_var(&var_id, |_builder| {});
            }
            Stmt::Expr(expr, _) => {
                self.compile_ast(expr, builder)?;
            }
            Stmt::Return(expr, _) => {
                self.compile_ast(expr, builder)?;
                builder.return_scope(|_builder| {});
            }
        }
        Ok(())
    }
}


#[derive(Debug)]
pub(crate) struct ParseError(String, Loc);

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f,"{}\n{}", self.0, self.1)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        ""
    }
}


#[derive(Clone)]
pub(crate) struct Loc {
    pub(crate) original: String,
    pub(crate) index: usize,
}

impl Display for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}^", self.original, " ".repeat(if self.index > 0 { self.index - 1 } else { 0 }))
    }
}

impl Debug for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Loc({})", self.index)
    }
}

impl Loc {
    pub(crate) fn none() -> Self {
        Loc { original: String::new(), index: 0 }
    }

    pub(crate) fn error(&self, msg: String) -> ParseError {
        ParseError(msg, self.clone())
    }
}