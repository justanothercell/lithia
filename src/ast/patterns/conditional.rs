use crate::ast::patterns::Consumer;
use crate::error::ParseError;
use crate::tokens::TokIter;

struct While();

impl Consumer for While {
    type Output = ();

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        todo!()
    }
}