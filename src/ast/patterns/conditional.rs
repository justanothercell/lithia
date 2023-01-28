use crate::ast::patterns::{Consumer, Pat};
use crate::error::{LithiaError, LithiaET};
use crate::tokens::TokIter;

pub(crate) struct While<Pred, Item>(pub(crate) Pat<Pred>, pub(crate) Pat<Item>);
impl<Pred, Item> Consumer for While<Pred, Item> {
    type Output = Vec<Item>;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
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

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        let start = iter.here();
        for (pred, item) in &self.0 {
            let mut pred_it = iter.clone();
            if pred.consume(&mut pred_it).is_ok() {
                return item.consume(iter)
            }
        }
        Err(LithiaET::ParsingError(format!("could not match to any branch in match, found {:?}", iter.this()?.tt)).at(start))
    }
}

pub(crate) struct Optional<Pred, Out>(pub(crate) Pat<Pred>, pub(crate) Pat<Out>);
impl<Pred, Out> Consumer for Optional<Pred, Out>{
    type Output = Option<Out>;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        Ok(if self.0.consume(&mut iter.clone()).is_ok() {
            Some(self.1.consume(iter)?)
        } else { None })
    }
}

pub(crate) struct Or<Pred, Out>(pub(crate) Pat<Pred>, pub(crate) Pat<Out>, pub(crate) Pat<Out>);
impl<Pred, Out> Consumer for Or<Pred, Out>{
    type Output = Out;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        Ok(if self.0.consume(&mut iter.clone()).is_ok() {
            self.1.consume(iter)?
        } else { self.2.consume(iter)? })
    }
}



pub(crate) struct Succeed<Out>(pub(crate) Pat<Out>);
impl<Out> Consumer for Succeed<Out>{
    type Output = ();

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        let mut start = iter.here();
        let out = self.0.consume(iter);
        start.combine(iter.here());
        match out {
            Ok(_) => Ok(()),
            Err(_) => Err(LithiaET::ParsingError("pattern expected to pass".to_string()).at(start))
        }
    }
}

pub(crate) struct Both<A, B>(pub(crate) Pat<A>, pub(crate) Pat<B>);
impl<A, B> Consumer for Both<A, B>{
    type Output = ();

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        let mut start = iter.here();
        let out1 = self.0.consume(&mut iter.clone());
        let out2 = self.1.consume(&mut iter.clone());
        if out1.is_err() || out2.is_err() {
            return Err(LithiaET::ParsingError("both pattern expected to pass".to_string()).at(start))
        }
        Ok(())
    }
}

pub(crate) struct Fail<Out>(pub(crate) Pat<Out>);
impl<Out> Consumer for Fail<Out>{
    type Output = ();

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        let mut start = iter.here();
        let out = self.0.consume(iter);
        start.combine(iter.here());
        match out {
            Ok(_) => Err(LithiaET::ParsingError("pattern expected to fail".to_string()).at(start)),
            Err(_) => Ok(())
        }
    }
}

pub(crate) struct IsOk<Out>(pub(crate) Pat<Out>);
impl<Out> Consumer for IsOk<Out>{
    type Output = bool;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        Ok(self.0.consume(iter).is_ok())
    }
}