use std::cell::{UnsafeCell};
use std::rc::Rc;
use crate::ast::patterns::{Consumer, Pat};
use crate::error::{LithiaError};
use crate::source::span::Span;
use crate::tokens::TokIter;

pub(crate) struct MapperRes<Out, Mapped>(Pat<Out>, fn(Out, Span) -> Result<Mapped, LithiaError>);
pub(crate) struct Mapper<Out, Mapped>(Pat<Out>, fn(Out, Span) -> Mapped);

pub(crate) trait Mapping{
    type Output;
    fn map_res<Mapped>(self, mapper: fn(Self::Output, Span) -> Result<Mapped, LithiaError>) -> MapperRes<Self::Output, Mapped>;
    fn map<Mapped>(self, mapper: fn(Self::Output, Span) -> Mapped) -> Mapper<Self::Output, Mapped>;
}

impl<T: Consumer + 'static> Mapping for T  {
    type Output = T::Output;
    fn map_res<Mapped>(self, mapper: fn(Self::Output, Span) -> Result<Mapped, LithiaError>) -> MapperRes<Self::Output, Mapped> {
        MapperRes(Rc::new(Box::new(self)), mapper)
    }

    fn map<Mapped>(self, mapper: fn(Self::Output, Span) -> Mapped) -> Mapper<Self::Output, Mapped> {
        Mapper(Rc::new(Box::new(self)), mapper)
    }
}

impl<Out, Mapped> Consumer for MapperRes<Out, Mapped>{
    type Output = Mapped;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        let mut start = iter.here();
        let out = self.0.consume(iter)?;
        start.combine(iter.here());
        self.1(out, start)
    }
}

impl<Out, Mapped> Consumer for Mapper<Out, Mapped>{
    type Output = Mapped;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        let mut start = iter.here();
        let out = self.0.consume(iter)?;
        start.combine(iter.here());
        Ok(self.1(out, start))
    }
}

pub(crate) struct Latent<Out>(UnsafeCell<Option<Pat<Out>>>);
impl<Out> Consumer for Latent<Out>{
    type Output = Out;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, LithiaError> {
        if let Some(p) = unsafe {&*self.0.get()}{
            p.consume(iter)
        } else {
            panic!("Latent was not finalized!")
        }
    }
}

impl<Out: 'static> Latent<Out> {
    pub(crate) fn new() -> (Pat<Out>, Rc<Self>){
        let rc = Rc::new(Self(UnsafeCell::new(None)));
        let c = Rc::clone(&rc);
        let pat = c.pat();
        (pat, rc)
    }
    pub(crate) fn finalize(&self, p: Pat<Out>){
        unsafe {*self.0.get() = Some(p);}
    }
}