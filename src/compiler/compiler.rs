use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::backtrace::{Backtrace};
use std::usize;
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
            Expr::Call(func, _, loc) => {
                let fn_id = if let Some(fn_id) = self.needed_externs.get(&func.ident.0) {
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
            Expr::Variable(var, _, loc) => {
                if let Some(var_id) = self.variables.get(&var.0) {
                    builder.push_variable(var_id);
                }
                else{
                    return Err(loc.error(format!("No variable with ident '{}'", var.0)))
                }
            }
            Expr::Value(val,_, _) => {
                builder.push_value(val);
            }
            Expr::While(box cond, box body, _, _) => {
                let loop_start = builder.gen_marker();
                let loop_end = builder.gen_marker();
                builder.set_marker(&loop_start);
                self.compile_ast(cond, builder)?;
                builder.jump(JmpType::Unless, &loop_end);
                self.compile_ast(body, builder)?;
                builder.jump(JmpType::Jmp, &loop_start);
                builder.set_marker(&loop_end);
            }
            Expr::If(box cond, box body_if, box body_else, _, _) => {
                let marker_else = builder.gen_marker();
                let marker_end = builder.gen_marker();
                self.compile_ast(cond, builder)?;
                builder.jump(JmpType::Unless, &marker_else);
                self.compile_ast(body_if, builder)?;
                builder.jump(JmpType::Jmp, &marker_end);
                builder.set_marker(&marker_else);
                self.compile_ast(body_else, builder)?;
                builder.set_marker(&marker_end);
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


pub(crate) struct ParseError(String, Loc, Backtrace);

impl ParseError {
    fn create_here(msg: String, loc: Loc) -> Self {
        let bt = Backtrace::capture();
        ParseError(msg, loc, bt)
    }

    pub(crate) fn with_loc(msg: String, loc: Loc) -> Self{
        ParseError::create_here(msg, loc)
    }

    pub(crate) fn without_loc(msg: String) -> Self{
        ParseError::create_here(msg, Loc::dummy())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.1.is_dummy {
            write!(f, "{}\n\n{}", self.0, self.2)
        }
        else {
            // println!("{.2}", err)
            let surrounding = if let Some(precision) = f.precision() {
                precision
            }
            else {
                2
            };
            let lines = self.1.source_code.split('\n').collect::<Vec<&str>>();
            let mut code_fmt = String::new();
            for l in (usize::max(self.1.line, surrounding + 1) - surrounding - 1)..usize::min(self.1.line + surrounding, lines.len()) {
                code_fmt.push_str(&format!("{:3} | ", l + 1));
                code_fmt.push_str(lines[l]);
                code_fmt.push_str("\n");
                if l == self.1.line - 1 {
                    code_fmt.push_str(&format!("    | {}^ {}:{}\n", &" ".repeat(self.1.index_in_line), self.1.line, self.1.index_in_line + 1));
                }
            }

            write!(f, "{:?}\n\n{}\n\n{}", self.0, code_fmt, self.2)
        }
    }
}

impl Error for ParseError {}


#[derive(Clone)]
pub(crate) struct Loc {
    pub(crate) index: usize,
    pub(crate) line: usize,
    pub(crate) index_in_line: usize,
    pub(crate) is_dummy: bool,
    pub(crate) source_code: String
}

impl Debug for Loc {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Loc({}:{})", self.line, self.index_in_line)
    }
}

impl Loc {
    pub(crate) fn dummy() -> Self {
        Loc { index: 0, line: 0, index_in_line: 0, is_dummy: true, source_code: String::new() }
    }

    pub(crate) fn error(&self, msg: String) -> ParseError {
        ParseError::with_loc(msg, self.clone())
    }
}