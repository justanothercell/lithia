use std::rc::Rc;
use crate::error::ParseError;
use crate::source::span::Span;
use crate::tokens::TokIter;

pub(crate) mod simple;
pub(crate) mod conditional;
pub(crate) mod create_patterns;

pub(crate) struct Pattern<T: Consumer, Out> {
    name: Option<String>,
    pub(crate) consumer: T,
    mapper: fn(T::Output, Span) -> Out
}

#[derive(Clone)]
pub(crate) struct Pat<Out>(Rc<Box<dyn Consumer<Output=Out>>>);
impl<Out> Consumer for Pat<Out> {
    type Output = Out;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        self.0.consume(iter)
    }
}

impl<T: Consumer + 'static, Out: 'static> Pattern<T, Out> {
    pub(crate) fn inline(consumer: T, mapper: fn(T::Output, Span) -> Out) -> Pat<Out>{
        let s = Box::new(Self {
            name: None,
            consumer,
            mapper
        }) as Box<dyn Consumer<Output=Out>>;
        let b = s;
        let rc = Rc::new(b);
        Pat(rc)
    }

    pub(crate) fn named(name: &str, consumer: T, mapper: fn(T::Output, Span) -> Out) -> Pat<Out>{
        Pat(Rc::new(Box::new(Self {
            name: Some(name.to_string()),
            consumer,
            mapper
        })))
    }
}

impl<T: Consumer, Out> Consumer for Pattern<T, Out> {
    type Output = Out;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut start = iter.here();
        let mut out = self.consumer.consume(iter)?;
        start.combine(iter.here());
        Ok((self.mapper)(out, start))
    }
}

pub(crate) trait Consumer {
    type Output;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError>;
}

macro_rules! tuple_consumer {
    ($($t:ident, $n: tt;)*) => {
        impl<$($t: Consumer,)*> Consumer for ($($t,)*) {
            type Output = ($($t::Output,)*);
            #[inline]
            fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
                Ok(($(self.$n.consume(iter)?,)*))
            }
        }
    };
}
tuple_consumer!();
tuple_consumer!(T0, 0;);
tuple_consumer!(T0, 0; T1, 1;);
tuple_consumer!(T0, 0; T1, 1; T2, 2;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9; T10, 10;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9; T10, 10; T11, 11;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9; T10, 10; T11, 11; T12, 12;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9; T10, 10; T11, 11; T12, 12; T13, 13;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9; T10, 10; T11, 11; T12, 12; T13, 13; T14, 14;);
tuple_consumer!(T0, 0; T1, 1; T2, 2; T3, 3; T4, 4; T5, 5; T6, 6; T7, 7; T8, 8; T9, 9; T10, 10; T11, 11; T12, 12; T13, 13; T14, 14; T15, 15;);
