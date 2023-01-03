use crate::error::ParseError;
use crate::source::Source;
use crate::tokens::tokenizer::tokenize;
use crate::tokens::TokIter;

pub(crate) struct Arguments{

}

pub(crate) fn compile(args: Arguments) -> Result<(), ParseError>{
    let source = Source::from_file("examples/testing/tokenizing.li")?;
    let tokens = tokenize(source)?;
    println!("{tokens:?}");
    let v = TokIter::new(tokens);
    Ok(())
}