use crate::ast::patterns::{Consumer, Pat};
use crate::error::{ParseError, ParseET};
use crate::tokens::TokIter;

pub(crate) struct While<Pred, Item>(pub(crate) Pat<Pred>, pub(crate) Pat<Item>);
impl<Pred, Item> Consumer for While<Pred, Item> {
    type Output = Vec<Item>;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut pred_it = iter.clone();
        let mut out = vec![];
        while self.0.consume(&mut pred_it).is_ok() {
            out.push(self.1.consume(iter)?);
            pred_it = iter.clone();
        }
        Ok(out)
    }
}

pub(crate) struct Match<Item>(pub(crate) Vec<(Pat<()>, Pat<Item>)>);
impl<Item> Consumer for Match<Item> {
    type Output = Item;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let start = iter.here();
        for (pred, item) in &self.0 {
            let mut pred_it = iter.clone();
            if pred.consume(&mut pred_it).is_ok() {
                return item.consume(iter)
            }
        }
        Err(ParseET::ParsingError(format!("could not match to any branch in match, found {:?}", iter.this()?.tt)).at(start))
    }
}

pub(crate) struct Succeed<Out>(pub(crate) Pat<Out>);
impl<Out> Consumer for Succeed<Out>{
    type Output = ();

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut start = iter.here();
        let out = self.0.consume(iter);
        start.combine(iter.here());
        match out {
            Ok(_) => Ok(()),
            Err(_) => Err(ParseET::ParsingError("pattern expected to pass".to_string()).at(start))
        }
    }
}

pub(crate) struct Fail<Out>(pub(crate) Pat<Out>);
impl<Out> Consumer for Fail<Out>{
    type Output = ();

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut start = iter.here();
        let out = self.0.consume(iter);
        start.combine(iter.here());
        match out {
            Ok(_) => Err(ParseET::ParsingError("pattern expected to fail".to_string()).at(start)),
            Err(_) => Ok(())
        }
    }
}

pub(crate) struct IsOk<Out>(pub(crate) Pat<Out>);
impl<Out> Consumer for IsOk<Out>{
    type Output = bool;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        Ok(self.0.consume(iter).is_ok())
    }
}