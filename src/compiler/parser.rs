use std::iter::Peekable;
use std::vec::IntoIter;
use crate::compiler::compiler::{Loc, ParseError};
use crate::compiler::tokenizer::{Bracket, Side, Token, Tokens, value_from_numer_literal};
use crate::{Expr, FuncCall, Ident, Stmt};
use crate::returnable::Returnable;
use crate::variable::{Type, Value};

type TokIter = Peekable<IntoIter<Token>>;

pub(crate) fn parse(tokens: Tokens) -> Returnable<Expr, ParseError> {
    let mut token_iter = tokens.get_tokens().into_iter().peekable();
    parse_scope(&mut token_iter)
}

macro_rules! expect_tok {
    ($token: expr, $tok_variant: path) => {
        match $token {
            $tok_variant(..) => (),
            tok => return Returnable::Err($token.loc().error(format!("Expected {} got {:?} '{}'", stringify!($tok_variant), tok, tok)))
        };
    };
}

macro_rules! expected {
    ($token: expr, $expected: expr) => {
        Returnable::Err($token.loc().error(format!("Unexpected token {:?} '{}', expected {}", $token, $token, $expected)))
    };
}

macro_rules! unexpected {
    ($token: expr) => {
        Returnable::Err($token.loc().error(format!("Unexpected token {:?} '{}'", $token, $token)))
    };
}

fn parse_scope(mut token_iter: &mut TokIter) -> Returnable<Expr, ParseError> {
    let start = token_iter.peek()?.loc().clone();
    let mut stmts = vec![];
    while let Some(t) = token_iter.peek() {
        let tok = t.clone();
        match tok {
            Token::Ident(s, loc) => {
                if s == "let" {
                    token_iter.next()?;
                    stmts.push(parse_var_creation(&mut token_iter, loc.clone())?);
                }
                else { // variable assignment or function call. let's ignore variable assignment for now
                    stmts.push(Stmt::Expr(parse_expr(token_iter)?, loc));
                    expect_tok!(token_iter.next()?, Token::EndStmt);
                }
            },
            Token::EOF(_) => {
                return Returnable::Ok(Expr::Stmts(stmts, None, Type::Empty, start));
            }
            tok => unexpected!(tok)?
        }
    }
    Returnable::Err(start.error(format!("Unexpected end while parsing this scope {:?}", start)))
}

fn parse_expr(token_iter: &mut TokIter) -> Returnable<Expr, ParseError> {
    match token_iter.peek()? {
        Token::Ident(_, _) => {
            let start = token_iter.peek()?.loc().clone();
            let path = parse_path(token_iter)?;
            return if let Token::Bracket(Bracket::Round(Side::Open), _) = token_iter.peek()? {
                token_iter.next()?;
                Returnable::Ok(Expr::Call(FuncCall {
                    ident: Ident(path),
                    args: parse_args(token_iter)?
                }, start))
            } else {
                Returnable::Ok(Expr::Variable(Ident(path), start))
            }
        }
        Token::Bracket(_, _) => {
            if let Token::Bracket(bracket, loc) = token_iter.next()? {
                return match bracket {
                    Bracket::Curly(Side::Open) => {
                        parse_scope(token_iter)
                    },
                    br => {
                        Returnable::Err(loc.error(format!("Unexpected bracket variation, expected curly opening bracket '{{', got: {:?} '{}'", br, br)))
                    }
                }
            }
            unreachable!()
        }
        Token::String(_, _) => {
            if let Token::String(val, loc) = token_iter.next()? {
                return Returnable::Ok(Expr::Value(Value::String(val), loc))
            }
            unexpected!(token_iter.next()?)?
        }
        Token::NumberLiteral(_, _, _) => {
            let num = token_iter.next()?;
            let loc = num.loc().clone();
            Returnable::Ok(Expr::Value(value_from_numer_literal(num)?, loc))
        }
        tok => unexpected!(tok)?
    }
}

fn parse_path(token_iter: &mut TokIter) -> Returnable<String, ParseError> {
    match token_iter.next()? {
        Token::Ident(mut ident, _) => {
            return match token_iter.peek()? {
                Token::PathSep(_) => {
                    token_iter.next()?;
                    ident.push_str("::");
                    ident.push_str(&parse_path(token_iter)?);
                    Returnable::Ok(ident)
                }
                _ => {
                    Returnable::Ok(ident)
                }
            }
        },
        tok => expected!(tok, "Ident")?
    }
}

fn parse_var_creation(mut token_iter: &mut TokIter, start: Loc) -> Returnable<Stmt, ParseError> {
    match token_iter.next()? {
        Token::Ident(ident, _) => {
            expect_tok!(token_iter.next()?, Token::TypeSep);
            match token_iter.next()? {
                Token::Ident(type_ident, _) => {
                    expect_tok!(token_iter.next()?, Token::Assign);
                    let expr = parse_expr(&mut token_iter)?;
                    expect_tok!(token_iter.next()?, Token::EndStmt);
                    return Returnable::<Stmt, ParseError>::Ok(Stmt::Create(Ident(ident), expr, start))
                },
                tok => expected!(tok, "Ident")
            }
        }
        tok => expected!(tok, "Ident"),
    }
}

fn parse_args(token_iter: &mut TokIter) -> Returnable<Vec<Expr>, ParseError> {
    let mut args = vec![];
    loop {
        args.push(parse_arg(token_iter)?);
        if let Token::Bracket(Bracket::Round(Side::Close), _) = token_iter.peek()? {
            token_iter.next()?;
            return Returnable::Ok(args)
        }
        expect_tok!(token_iter.next()?, Token::ArgSep);
    }
}

fn parse_arg(mut token_iter: &mut TokIter) -> Returnable<Expr, ParseError> {
    let expr = parse_expr(token_iter)?;
    Returnable::Ok(expr)
}