use std::rc::Rc;
use crate::ast::patterns::{Consumer, Pat};
use crate::error::{ParseError};
use crate::source::span::Span;
use crate::tokens::TokIter;

pub(crate) struct MapperRes<Out, Mapped>(Pat<Out>, fn(Out, Span) -> Result<Mapped, ParseError>);
pub(crate) struct Mapper<Out, Mapped>(Pat<Out>, fn(Out, Span) -> Mapped);

pub(crate) trait Mapping{
    type Output;
    fn map_res<Mapped>(self, mapper: fn(Self::Output, Span) -> Result<Mapped, ParseError>) -> MapperRes<Self::Output, Mapped>;
    fn map<Mapped>(self, mapper: fn(Self::Output, Span) -> Mapped) -> Mapper<Self::Output, Mapped>;
}

impl<Out> Mapping for Pat<Out>  {
    type Output = Out;
    fn map_res<Mapped>(self, mapper: fn(Self::Output, Span) -> Result<Mapped, ParseError>) -> MapperRes<Self::Output, Mapped> {
        MapperRes(self, mapper)
    }
    fn map<Mapped>(self, mapper: fn(Self::Output, Span) -> Mapped) -> Mapper<Self::Output, Mapped> {
        Mapper(self, mapper)
    }
}

impl<T: Consumer + 'static> Mapping for T  {
    type Output = T::Output;
    fn map_res<Mapped>(self, mapper: fn(Self::Output, Span) -> Result<Mapped, ParseError>) -> MapperRes<Self::Output, Mapped> {
        MapperRes(Rc::new(Box::new(self)), mapper)
    }

    fn map<Mapped>(self, mapper: fn(Self::Output, Span) -> Mapped) -> Mapper<Self::Output, Mapped> {
        Mapper(Rc::new(Box::new(self)), mapper)
    }
}

impl<Out, Mapped> Consumer for MapperRes<Out, Mapped>{
    type Output = Mapped;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut start = iter.here();
        let out = self.0.consume(iter)?;
        start.combine(iter.here());
        self.1(out, start)
    }
}

impl<Out, Mapped> Consumer for Mapper<Out, Mapped>{
    type Output = Mapped;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut start = iter.here();
        let out = self.0.consume(iter)?;
        start.combine(iter.here());
        Ok(self.1(out, start))
    }
}
