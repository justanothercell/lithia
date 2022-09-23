use std::iter::Peekable;
use std::vec::IntoIter;
use crate::compiler::compiler::{Loc, ParseError};
use crate::compiler::tokenizer::{Token, Tokens};
use crate::{Expr, Ident, Stmt};

type TokIter = Peekable<IntoIter<Token>>;

pub(crate) fn parse(tokens: Tokens) -> Result<Expr, ParseError> {
    let mut token_iter = tokens.get_tokens().into_iter().peekable();
    parse_scope(&mut token_iter)
}

fn parse_scope(mut token_iter: &mut TokIter) -> Result<Expr, ParseError> {
    while let Some(t) = token_iter.next() {
        if let Token::Ident(s, loc) = t {
            if &s == "let" {
                parse_var_creation(&mut token_iter, loc)?;
            }
            else { // variable assignment or function call

            }
        }
        else {
            return Err(t.loc().error(format!("Unexpected token {:?} '{}'", t, t)))
        }
    }
    Ok(Expr::Empty(Loc::none()))
}

macro_rules! expect_tok {
    ($token: expr, $tok_variant: path) => {
        match $token {
            $tok_variant(..) => (),
            _ => return Err($token.loc().error(format!("Expected {} got {:?}", stringify!($tok_variant), $token)))
        };
    };
}

fn parse_expr(token_iter: &mut TokIter) -> Result<Expr, ParseError> {

}

fn parse_var_creation(mut token_iter: &mut TokIter, start: Loc) -> Result<Stmt, ParseError> {
    match token_iter.next()? {
        Token::Ident(ident, _) => {
            expect_tok!(token_iter.next()?, Token::TypeSep);
            match token_iter.next()? {
                Token::Ident(type_ident, _) => {
                    expect_tok!(token_iter.next()?, Token::Assign);
                    return Ok(Stmt::Create(Ident(ident), parse_expr(&mut token_iter)?, start))
                },
                tok => Err(tok.loc().error(format!("Unexpected token {:?} '{}', expected Ident", tok, tok)))
            }
        }
        tok => Err(tok.loc().error(format!("Unexpected token {:?} '{}', expected Ident", tok, tok)))
    }
}
