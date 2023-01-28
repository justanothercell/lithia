use crate::ast::{Ident, Module};
use crate::ast::create_patterns::build_patterns;
use crate::error::LithiaError;
use crate::source::span::Span;
use crate::tokens::{Token, TokIter};

pub(crate) fn parse(tokens: Vec<Token>, mod_name: (String, Option<Span>)) -> Result<Module, LithiaError>{
    let patterns = build_patterns();
    let mut tokens = TokIter::new(tokens);
    let ((functions, constants), loc) = patterns.module_content.consume(&mut tokens)?;
    Ok(Module{
        name: Ident(mod_name.0, mod_name.1.unwrap_or(loc.clone())),
        sub_modules: Default::default(),
        functions,
        constants,
        loc
    })
}