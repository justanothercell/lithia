use crate::ast::code_printer::CodePrinter;
use crate::ast::parser::parse;
use crate::error::ParseError;
use crate::source::Source;
use crate::tokens::tokenizer::tokenize;

pub(crate) struct Arguments{

}

pub(crate) fn compile(args: Arguments) -> Result<(), ParseError>{
    let source = Source::from_file("examples/testing/hello_world.li")?;
    let tokens = tokenize(source)?;
    println!("{tokens:?}");
    let module = parse(tokens, ("main".to_string(), None))?;
    println!("{}", module.print());
    Ok(())
}