use crate::ast::{Ident, Module};
use crate::error::ParseError;
use crate::source::span::Span;
use crate::tokens::Token;

pub(crate) fn parse(tokens: Vec<Token>) -> Result<Module, ParseError>{
    Ok(Module{
        name: Ident("".to_string(), Span::dummy()),
        sub_modules: Default::default(),
        functions: Default::default(),
        loc: Span::dummy()
    })
}