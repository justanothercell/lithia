use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::compiler::compiler::{Loc, ParseError};
use crate::compiler::tokenizer::{Bracket, Op, OpGroup, Side, Token, Tokens, value_from_numer_literal};
use crate::{Expr, FuncCall, Ident, Stmt};
use crate::variable::{ToOneType, Type, Value};
use crate::vm::bindings::Function;

#[derive(Debug)]
pub(crate) struct ParseContext {
    externs: HashMap<String, Function>,
    used_externs: HashMap<String, (Vec<Type>, Vec<Type>)>,
    var_stack: Vec<HashMap<String, Option<Type>>>
}

type var_nonexistent = bool;
type var_already_exists = bool;
type var_type_overridden = bool;

struct VarTypeAssign(String, var_nonexistent, var_type_overridden);
struct VarCreation(String, var_already_exists, var_type_overridden);

impl VarTypeAssign {
    fn ok(self, loc: &Loc) -> Result<(), ParseError>{
        match (self.1, self.2) {
            (true, _) => Err(loc.error(format!("Variable {} does not exist", self.0))),
            (false, true) => Err(loc.error(format!("Variable {} has a different type", self.0))),
            (false, false) => Ok(())
        }
    }
}

impl VarCreation {
    fn ok(self, loc: &Loc) -> Result<(), ParseError>{
        match (self.1, self.2) {
            (true, _) => Err(loc.error(format!("Variable {} already exists in this scope", self.0))),
            (false, true) => unreachable!(),
            (false, false) => Ok(())
        }
    }
}

impl ParseContext {
    fn use_extern(&mut self, ident: &str) -> Option<&Function>{
        self.externs.get(ident).map(|f| {
            self.used_externs.insert(ident.to_string(), (f.1.clone(), f.2.clone()));
            f
        })
    }

    fn add_frame(&mut self) {
        self.var_stack.push(HashMap::new())
    }

    fn remove_frame(&mut self) {
        let _ = self.var_stack.pop();
    }

    fn var_exists(&self, ident: &str) -> bool {
        for i in (0..self.var_stack.len()).rev() {
            if self.var_stack[i].contains_key(ident) {
                return true
            }
        }
        false
    }

    fn var_get_type(&self, ident: &str) -> &Option<Type> {
        for i in (0..self.var_stack.len()).rev() {
            if let Some(t) = self.var_stack[i].get(ident) {
                return t
            }
        }
        &None
    }

    fn var_set_type(&mut self, ident: String, ty: Option<Type>) -> VarTypeAssign{
        let i = self.var_stack.len() - 1;
        match self.var_stack[i].get_mut(&ident) {
            Some(t) => {
                let r = match t {
                    Some(_) => VarTypeAssign(ident, false, true),
                    None => VarTypeAssign(ident, false, false)
                };
                *t = ty;
                r
            }
            None => VarTypeAssign(ident, true, false)
        }
    }

    fn var_create(&mut self, ident: String, ty: Option<Type>) -> VarCreation{
        let i = self.var_stack.len() - 1;
        match self.var_stack[i].get_mut(&ident) {
            Some(t) => {
                let r = match t {
                    Some(_) => VarCreation(ident, true, true),
                    None => VarCreation(ident, true, false)
                };
                *t = ty;
                r
            }
            None => {
                self.var_stack[i].insert(ident.to_string(), ty);
                VarCreation(ident, false, false)
            }
        }
    }

    pub(crate) fn used_externs(self) -> HashMap<String, (Vec<Type>, Vec<Type>)>{
        self.used_externs
    }
}

impl ParseContext {
    pub(crate) fn new(externs: HashMap<String, Function>) -> Self {
        ParseContext { externs, used_externs: HashMap::new(), var_stack: vec![] }
    }
}

pub(crate) fn parse(tokens: Tokens, mut ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    let mut token_iter = TokIter::new(tokens);
    parse_scope(&mut token_iter, &mut ctx)
}

macro_rules! expect_tok {
    ($token: expr, $tok_variant: path) => {
        match $token {
            $tok_variant(..) => (),
            tok => return Err(tok.loc().error(format!("Expected {} got {:?} '{}'", stringify!($tok_variant), tok, tok)))
        };
    };
}

macro_rules! expect_ident {
    ($token: expr, $ident: expr) => {
        let tok = $token;
        match tok.clone() {
            Token::Ident(s, _) => if &s != $ident {
                return Err(tok.loc().error(format!("Expected '{}' got '{}'", $ident, s)))
            },
            tok => return Err(tok.loc().error(format!("Expected '{}' got {:?} '{}'", $ident, tok, tok)))
        };
    };
}

macro_rules! expect_tok_specific {
    ($token: expr, $tok_variant: pat_param) => {
        match $token {
            $tok_variant => (),
            tok => return Err(tok.loc().error(format!("Expected {} got {:?} '{}'", stringify!($tok_variant), tok, tok)))
        };
    };
}

macro_rules! expected {
    ($token: expr, $expected: expr) => {
        Err($token.loc().error(format!("Unexpected token {:?} '{}', expected {}", $token, $token, $expected)))
    };
}

macro_rules! unexpected {
    ($token: expr) => {
        Err($token.loc().error(format!("Unexpected token {:?} '{}'", $token, $token)))
    };
}

macro_rules! negotiate_type {
    ($expr_1: ident, $expr_2: ident, $loc: ident) => {
        match ($expr_1.get_type(), $expr_2.get_type()) {
            (Some(t1), Some(t2)) => {
                if t1 != t1 {
                    return Err($loc.error(format!("Expected same type, got {} and {}", t1.to_string(), t2.to_string())))
                }
            }
            (Some(t1), None) => $expr_1.set_type(Some(t1.clone())),
            (None, Some(t2)) => $expr_2.set_type(Some(t2.clone())),
            (None, None) =>  return Err($loc.error("No concrete type could be inferred".to_string()))
        }
    };
}

macro_rules! verify_or_assign_type {
    ($expr: ident, $t: expr, $loc: ident, $ctx: ident) => {
        let ty = $t;
        match $expr.get_type().clone() {
            Some(t) => {
                if &t != ty {
                    if &t != &Type::String {
                        let ident = format!("{}::to_string", t.to_string());
                        let f = $ctx.use_extern(&ident).ok_or_else(|| $loc.error(format!("No such extern function provided: '{}", ident)))?.clone();
                        verify_arglen!(f, 1, $loc);
                        if &t != &f.1[0] {
                            return Err($loc.error(format!("Expected same type, got {} and {}", t.to_string(), f.1[0].to_string())))
                        }
                        let l = $expr.loc().clone();
                        $expr = Expr::Call(FuncCall {
                            ident: Ident(ident),
                            args: vec![$expr]
                        }, Some(f.2.clone().to_one()),  l);
                    }
                    else {
                        return Err($loc.error(format!("Expected same type, got {} and {}", t.to_string(), ty.to_string())))
                    }
                }
            }
            None => $expr.set_type(Some(ty.clone()))
        }
    };
}

macro_rules! verify_arglen {
    ($f: expr, $provided: expr, $loc: ident) => {
        let p = $provided;
        let args =  $f.1.len();
        if args != p {
            return Err($loc.error(format!("Expected {} args, got {}", args, p)))
        }
    };
}

fn parse_scope(mut token_iter: &mut TokIter, ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    ctx.add_frame();
    let start = token_iter.peek()?.loc().clone();
    let mut stmts = vec![];
    while let Some(t) = token_iter.peek().ok() {
        let tok = t.clone();
        match tok {
            Token::Ident(s, loc) => {
                if s == "let" { // filter out let statements
                    token_iter.next()?;
                    stmts.push(parse_var_creation(&mut token_iter, loc.clone(), ctx)?);
                }
                else { // variable assignment
                    if let Token::Assign(_) = token_iter.peek_ahead(1)? {
                        if let Token::Ident(ident, _) = token_iter.next()? {
                            expect_tok!(token_iter.next()?, Token::Assign);
                            let expr = parse_expr(token_iter, ctx)?;
                            stmts.push(Stmt::Assign(Ident(ident), expr, loc.clone()));
                            expect_tok!(token_iter.next()?, Token::EndStmt);
                        }

                    }
                    else { // if/loop or function call, or operators
                        let expr = parse_expr(token_iter, ctx)?;
                        let needs_end_stmt = if let Expr::While(..) | Expr::If(..) = expr { false } else { true };
                        stmts.push(Stmt::Expr(expr, loc.clone()));
                        if needs_end_stmt {
                            expect_tok!(token_iter.next()?, Token::EndStmt);
                        }
                    }
                }
            },
            Token::Bracket(Bracket::Curly(Side::Close), _) => {
                // TODO: returns / trailing expr
                ctx.remove_frame();
                return Ok(Expr::Stmts(stmts, None, Some(Type::Empty), start));
            }
            Token::EOF(_) => {
                ctx.remove_frame();
                // TODO: returns / trailing expr
                return Ok(Expr::Stmts(stmts, None, Some(Type::Empty), start));
            }
            tok => unexpected!(tok)?
        }
    }
    Err(start.error(format!("Unexpected end while parsing this scope {:?}", start)))
}

fn parse_expr(token_iter: &mut TokIter, ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    enum ExprChain {
        Expr(Expr),
        Op(Op, Loc)
    }
    let mut expr_chain = vec![ExprChain::Expr(parse_single_expr(token_iter, ctx)?)];

    loop {
        match token_iter.peek()? {
            Token::Op(_, _) => {
                if let Token::Op(op, loc) = token_iter.next()? {
                    expr_chain.push(ExprChain::Op(op, loc));
                    expr_chain.push(ExprChain::Expr(parse_single_expr(token_iter, ctx)?));
                }
            }
            _ => break
        }
    }

    fn combine_chain(op_group: OpGroup, expr_chain: Vec<ExprChain>, ctx: &mut ParseContext) -> Result<Vec<ExprChain>, ParseError>{
        let mut expr_iter = expr_chain.into_iter();
        let mut combined_expr_chain: Vec<ExprChain> = vec![];
        while let Some(chain) = expr_iter.next() {
            match chain {
                ExprChain::Op(op, loc) if op.group() == op_group => {
                    if let (Some(ExprChain::Expr(mut left)), Some(ExprChain::Expr(mut right))) = (combined_expr_chain.pop(), expr_iter.next()){
                        let t = left.get_type().clone().ok_or_else(|| loc.error("Type is ambiguous. Please provide explicit type.".to_string()))?;
                        let ident = if t == Type::String && op == Op::Add {
                            "string::join".to_string()
                        }
                        else {
                            format!("{}::{}", t.to_string(), op.fn_op())
                        };
                        let f = ctx.use_extern(&ident).ok_or_else(|| loc.error(format!("No such extern function provided: '{}", ident)))?.clone();
                        verify_arglen!(f, 2, loc);
                        verify_or_assign_type!(left, &f.1[0], loc, ctx);
                        verify_or_assign_type!(right, &f.1[1], loc, ctx);
                        combined_expr_chain.push(ExprChain::Expr(Expr::Call(FuncCall {
                            ident: Ident(ident),
                            args: vec![left, right]
                        }, Some(f.2.clone().to_one()), loc)))
                    }
                    else{
                        return Err(loc.error(format!("Operation '{}' requires expressions on both sides", op)));
                    }
                },
                ec => combined_expr_chain.push(ec)
            }
        }
        Ok(combined_expr_chain)
    }

    expr_chain = combine_chain(OpGroup::Pow, expr_chain, ctx)?;
    expr_chain = combine_chain(OpGroup::Dot, expr_chain, ctx)?;
    expr_chain = combine_chain(OpGroup::Dash, expr_chain, ctx)?;
    expr_chain = combine_chain(OpGroup::Bit, expr_chain, ctx)?;
    expr_chain = combine_chain(OpGroup::Comp, expr_chain, ctx)?;
    expr_chain = combine_chain(OpGroup::Bool, expr_chain, ctx)?;

    if expr_chain.len() == 1 {
        if let Some(ExprChain::Expr(expr)) = expr_chain.pop(){
            return Ok(expr);
        }
    }

    if expr_chain.len() > 1 {
        return Err(match expr_chain.pop().unwrap() {
            ExprChain::Expr(expr) => expr.loc().error("Found more than one expressions without operator in between!".to_string()),
            ExprChain::Op(_, loc) => loc.error("Found loose operator!".to_string()),
        })
    }
    unreachable!()
}

fn parse_single_expr(mut token_iter: &mut TokIter, ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    match token_iter.peek()? {
        Token::Ident(_, _) => {
            let (ident, start) = if let Token::Ident(i, loc) = token_iter.peek()? {
                (i.clone(), loc.clone())
            }
            else{
                unreachable!()
            };
            return match ident.as_str() {
                "if" => {
                    parse_if(&mut token_iter, start, ctx)
                }
                "while" => {
                    parse_while(&mut token_iter, start, ctx)
                }
                _ => {
                    let path = parse_path(token_iter)?;
                    if let Token::Bracket(Bracket::Round(Side::Open), _) = token_iter.peek()? {
                        token_iter.next()?;
                        let f = ctx.use_extern(&path).ok_or_else(|| start.error(format!("No such extern function provided: '{}", ident)))?.clone();
                        let mut args = parse_args(token_iter, ctx)?;
                        verify_arglen!(f, args.len(), start);
                        let args_iter = args.into_iter().enumerate();
                        args = vec![];
                        for (i, mut arg) in args_iter {
                            verify_or_assign_type!(arg, &f.1[i], start, ctx);
                            args.push(arg)
                        }

                        let ret = f.2.clone().to_one();
                        Ok(Expr::Call(FuncCall {
                            ident: Ident(path),
                            args: args
                        }, Some(ret), start))
                    } else {
                        let t = ctx.var_get_type(&path).clone();
                        Ok(Expr::Variable(Ident(path), t, start))
                    }
                }
            }
        }
        Token::Bracket(_, _) => {
            if let Token::Bracket(bracket, loc) = token_iter.next()? {
                return match bracket {
                    Bracket::Curly(Side::Open) => {
                        parse_scope(token_iter, ctx)
                    },
                    br => {
                        Err(loc.error(format!("Unexpected bracket variation, expected curly opening bracket '{{', got: {:?} '{}'", br, br)))
                    }
                }
            }
            unreachable!()
        }
        Token::String(_, _) => {
            if let Token::String(val, loc) = token_iter.next()? {
                return Ok(Expr::Value(Value::String(val),  Some(Type::String), loc))
            }
            unexpected!(token_iter.next()?)?
        }
        Token::NumLiteral(_, _, _) => {
            let num = token_iter.next()?;
            let loc = num.loc().clone();
            let val = value_from_numer_literal(num)?;
            let t = val.get_type();
            Ok(Expr::Value(val, Some(t), loc))
        }
        tok => unexpected!(tok)?
    }
}

fn parse_path(token_iter: &mut TokIter) -> Result<String, ParseError> {
    match token_iter.next()? {
        Token::Ident(mut ident, _) => {
            return match token_iter.peek()? {
                Token::PathSep(_) => {
                    token_iter.next()?;
                    ident.push_str("::");
                    let path = parse_path(token_iter)?;
                    ident.push_str(&path);
                    Ok(ident)
                }
                _ => {
                    Ok(ident)
                }
            }
        },
        tok => expected!(tok, "Ident")?
    }
}

fn parse_var_creation(mut token_iter: &mut TokIter, start: Loc, ctx: &mut ParseContext) -> Result<Stmt, ParseError> {
    match token_iter.next()? {
        Token::Ident(ident, _) => {
            expect_tok!(token_iter.next()?, Token::TypeSep);
            match token_iter.next()? {
                Token::Ident(type_ident, _) => {
                    expect_tok!(token_iter.next()?, Token::Assign);
                    let mut expr = parse_expr(&mut token_iter, ctx)?;
                    expect_tok!(token_iter.next()?, Token::EndStmt);
                    let mut dummy = Expr::Value(Value::Empty, Type::from_str(&type_ident), Loc::dummy());
                    negotiate_type!(dummy, expr, start);
                    ctx.var_create(ident.clone(), dummy.get_type().to_owned()).ok(&start)?;
                    return Ok(Stmt::Create(Ident(ident), expr, start))
                },
                tok => expected!(tok, "Ident")
            }
        }
        tok => expected!(tok, "Ident"),
    }
}

fn parse_if(mut token_iter: &mut TokIter, start: Loc, ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    expect_ident!(token_iter.next()?, "if");
    let cond = parse_expr(&mut token_iter, ctx)?;
    match cond.get_type() {
        Some(Type::Bool) => (),
        Some(other) => return Err(start.error(format!("Expected expression of type bool, got {}", other.to_string()))),
        None => return Err(start.error("Expected expression of type bool, got ambiguous type".to_string())),
    }
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Open), _));
    let mut body_if = parse_scope(&mut token_iter, ctx)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Close), _));
    match token_iter.peek()? {
        Token::Ident(cwd, _) if cwd == "else" => {
            expect_ident!(token_iter.next()?, "else");
            expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Open), _));
            let mut body_else = parse_scope(&mut token_iter, ctx)?;
            expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Close), _));
            negotiate_type!(body_if, body_else, start);
            let t = body_if.get_type().clone();
            Ok(Expr::If(Box::from(cond), Box::from(body_if), Box::from(body_else), t.clone(), start))
        }
        _ => {
            let t = body_if.get_type().clone();
            match t {
                None => return Err(start.error("Expected expression of type bool, got ambiguous type".to_string())),
                _ => ()
            }
            Ok(Expr::If(Box::from(cond), Box::from(body_if), Box::from(Expr::Empty(start.clone())), t.clone(), start))
        }
    }
}

fn parse_while(mut token_iter: &mut TokIter, start: Loc, ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    expect_ident!(token_iter.next()?, "while");
    let cond = parse_expr(&mut token_iter, ctx)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Open), _));
    let body = parse_scope(&mut token_iter, ctx)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Close), _));
    let t = body.get_type().clone();
    Ok(Expr::While(Box::from(cond), Box::from(body),
                   Some(t.ok_or_else(|| start.error("Type is ambiguous. Please provide explicit type.".to_string()))?), start))
}

fn parse_args(token_iter: &mut TokIter, ctx: &mut ParseContext) -> Result<Vec<Expr>, ParseError> {
    let mut args = vec![];
    loop {
        args.push(parse_arg(token_iter, ctx)?);
        if let Token::Bracket(Bracket::Round(Side::Close), _) = token_iter.peek()? {
            token_iter.next()?;
            return Ok(args)
        }
        expect_tok!(token_iter.next()?, Token::ArgSep);
    }
}

fn parse_arg(token_iter: &mut TokIter, ctx: &mut ParseContext) -> Result<Expr, ParseError> {
    let expr = parse_expr(token_iter, ctx)?;
    Ok(expr)
}

#[derive(Debug)]
struct EOT;

impl Display for EOT {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected end of tokens!")
    }
}

impl Error for EOT {

}

impl From<EOT> for ParseError {
    fn from(eot: EOT) -> Self {
        ParseError::without_loc(format!("{}", eot))
    }
}

struct TokIter {
    tokens: VecDeque<Token>
}

impl TokIter {
    pub(crate) fn new(tokens: Tokens) -> Self{
        TokIter { tokens: VecDeque::from(tokens.get_tokens()) }
    }

    pub(crate) fn next(&mut self) -> Result<Token, EOT>{
        self.tokens.pop_front().ok_or(EOT)
    }

    pub(crate) fn peek(&mut self) -> Result<&Token, EOT>{
        self.tokens.get(0).ok_or(EOT)
    }

    pub(crate) fn peek_ahead(&mut self, i: usize) -> Result<&Token, EOT>{
        self.tokens.get(i).ok_or(EOT)
    }
}