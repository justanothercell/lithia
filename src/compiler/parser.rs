use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::compiler::compiler::{Loc, ParseError};
use crate::compiler::tokenizer::{Bracket, Side, Token, Tokens, value_from_numer_literal};
use crate::{Expr, FuncCall, Ident, Stmt};
use crate::variable::{Type, Value};

pub(crate) fn parse(tokens: Tokens) -> Result<Expr, ParseError> {
    let mut token_iter = TokIter::new(tokens);
    parse_scope(&mut token_iter)
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

fn parse_scope(mut token_iter: &mut TokIter) -> Result<Expr, ParseError> {
    let start = token_iter.peek()?.loc().clone();
    let mut stmts = vec![];
    while let Some(t) = token_iter.peek().ok() {
        let tok = t.clone();
        match tok {
            Token::Ident(s, loc) => {
                if s == "let" { // filter out let statements
                    token_iter.next()?;
                    stmts.push(parse_var_creation(&mut token_iter, loc.clone())?);
                }
                else { // variable assignment
                    if let Token::Assign(_) = token_iter.peek_ahead(1)? {
                        if let Token::Ident(ident, _) = token_iter.next()? {
                            expect_tok!(token_iter.next()?, Token::Assign);
                            let expr = parse_expr(token_iter)?;
                            stmts.push(Stmt::Assign(Ident(ident), expr, loc.clone()));
                            expect_tok!(token_iter.next()?, Token::EndStmt);
                        }

                    }
                    else { // if/loop or function call
                        let expr = parse_expr(token_iter)?;
                        let needs_end_stmt = if let Expr::While(..) | Expr::If(..) = expr { false } else { true };
                        stmts.push(Stmt::Expr(expr, loc.clone()));
                        if needs_end_stmt {
                            expect_tok!(token_iter.next()?, Token::EndStmt);
                        }
                    }
                }
            },
            Token::Bracket(Bracket::Curly(Side::Close), _) => {
                return Ok(Expr::Stmts(stmts, None, Type::Empty, start));
            }
            Token::EOF(_) => {
                return Ok(Expr::Stmts(stmts, None, Type::Empty, start));
            }
            tok => unexpected!(tok)?
        }
    }
    Err(start.error(format!("Unexpected end while parsing this scope {:?}", start)))
}

fn parse_expr(mut token_iter: &mut TokIter) -> Result<Expr, ParseError> {
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
                    parse_if(&mut token_iter, start)
                }
                "while" => {
                    parse_while(&mut token_iter, start)
                }
                _ => {
                    let path = parse_path(token_iter)?;
                    if let Token::Bracket(Bracket::Round(Side::Open), _) = token_iter.peek()? {
                        token_iter.next()?;
                        Ok(Expr::Call(FuncCall {
                            ident: Ident(path),
                            args: parse_args(token_iter)?
                        }, start))
                    } else {
                        Ok(Expr::Variable(Ident(path), start))
                    }
                }
            }
        }
        Token::Bracket(_, _) => {
            if let Token::Bracket(bracket, loc) = token_iter.next()? {
                return match bracket {
                    Bracket::Curly(Side::Open) => {
                        parse_scope(token_iter)
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
                return Ok(Expr::Value(Value::String(val), loc))
            }
            unexpected!(token_iter.next()?)?
        }
        Token::NumLiteral(_, _, _) => {
            let num = token_iter.next()?;
            let loc = num.loc().clone();
            Ok(Expr::Value(value_from_numer_literal(num)?, loc))
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

fn parse_var_creation(mut token_iter: &mut TokIter, start: Loc) -> Result<Stmt, ParseError> {
    match token_iter.next()? {
        Token::Ident(ident, _) => {
            expect_tok!(token_iter.next()?, Token::TypeSep);
            match token_iter.next()? {
                Token::Ident(_type_ident, _) => {
                    expect_tok!(token_iter.next()?, Token::Assign);
                    let expr = parse_expr(&mut token_iter)?;
                    expect_tok!(token_iter.next()?, Token::EndStmt);
                    return Ok(Stmt::Create(Ident(ident), expr, start))
                },
                tok => expected!(tok, "Ident")
            }
        }
        tok => expected!(tok, "Ident"),
    }
}

fn parse_if(mut token_iter: &mut TokIter, start: Loc) -> Result<Expr, ParseError> {
    expect_ident!(token_iter.next()?, "if");
    let cond = parse_expr(&mut token_iter)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Open), _));
    let body_if = parse_scope(&mut token_iter)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Close), _));
    expect_ident!(token_iter.next()?, "else");
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Open), _));
    let body_else = parse_scope(&mut token_iter)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Close), _));
    Ok(Expr::If(Box::from(cond), Box::from(body_if), Box::from(body_else), start.clone()))
}

fn parse_while(mut token_iter: &mut TokIter, start: Loc) -> Result<Expr, ParseError> {
    expect_ident!(token_iter.next()?, "while");
    let cond = parse_expr(&mut token_iter)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Open), _));
    let body = parse_scope(&mut token_iter)?;
    expect_tok_specific!(token_iter.next()?, Token::Bracket(Bracket::Curly(Side::Close), _));
    Ok(Expr::While(Box::from(cond), Box::from(body), start.clone()))
}

fn parse_args(token_iter: &mut TokIter) -> Result<Vec<Expr>, ParseError> {
    let mut args = vec![];
    loop {
        args.push(parse_arg(token_iter)?);
        if let Token::Bracket(Bracket::Round(Side::Close), _) = token_iter.peek()? {
            token_iter.next()?;
            return Ok(args)
        }
        expect_tok!(token_iter.next()?, Token::ArgSep);
    }
}

fn parse_arg(token_iter: &mut TokIter) -> Result<Expr, ParseError> {
    let expr = parse_expr(token_iter)?;
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