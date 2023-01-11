use std::rc::Rc;
use crate::error::{ParseError};
use crate::source::span::Span;
use crate::tokens::TokIter;

pub(crate) mod simple;
pub(crate) mod conditional;
pub(crate) mod dynamic;

pub(crate) struct Pattern<T: Consumer, Out> {
    name: Option<String>,
    pub(crate) consumer: T,
    mapper: fn(T::Output, Span) -> Out
}

pub(crate) type Pat<Out> = Rc<Box<dyn Consumer<Output=Out>>>;

impl<T: Consumer + 'static, Out: 'static> Pattern<T, Out> {
    pub(crate) fn inline(consumer: T, mapper: fn(T::Output, Span) -> Out) -> Pat<Out>{
        Rc::new(Box::new(Self {
            name: None,
            consumer,
            mapper
        }))
    }

    pub(crate) fn named(name: &str, consumer: T, mapper: fn(T::Output, Span) -> Out) -> Pat<Out>{
        Rc::new(Box::new(Self {
            name: Some(name.to_string()),
            consumer,
            mapper
        }))
    }
}

impl<T: Consumer, Out> Consumer for Pattern<T, Out> {
    type Output = Out;

    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        let mut start = iter.here();
        let out = self.consumer.consume(iter);
        if out.is_err() && self.name.is_some() {
            return Err(unsafe {out.unwrap_err_unchecked()}.when(format!("parsing {}", self.name.clone().unwrap())));
        }
        start.combine(iter.peekn(-1)?.loc);
        Ok((self.mapper)(out?, start))
    }
}

impl<Out> Consumer for Rc<Box<dyn Consumer<Output=Out>>> {
    type Output = Out;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        self.as_ref().consume(iter)
    }
}

impl<Out, T: Consumer<Output=Out>> Consumer for Rc<T> {
    type Output = Out;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError> {
        self.as_ref().consume(iter)
    }
}

pub(crate) trait Consumer {
    type Output;
    fn consume(&self, iter: &mut TokIter) -> Result<Self::Output, ParseError>;
    fn pat(self) -> Pat<Self::Output> where Self: Sized + 'static {
        Rc::new(Box::new(self))
    }
}

macro_rules! tuple_consumer {
    ($($t:ident, $n: tt;)*) => {
        impl<$($t: Consumer,)*> Consumer for ($($t,)*) {
            type Output = ($($t::Output,)*);
            #[inline]
            #[allow(unused_variables)]
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
