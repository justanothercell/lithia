use crate::ast::{Ident, Module};
use crate::ast::create_patterns::build_patterns;
use crate::error::ParseError;
use crate::source::span::Span;
use crate::tokens::{Token, TokIter};

pub(crate) fn parse(tokens: Vec<Token>) -> Result<Module, ParseError>{
    let patterns = build_patterns();
    let mut tokens = TokIter::new(tokens);
    let r = patterns.module_content.consume(&mut tokens)?;
    println!("{r:?}");
    Ok(Module{
        name: Ident("".to_string(), Span::dummy()),
        sub_modules: Default::default(),
        functions: Default::default(),
        loc: Span::dummy()
    })
}