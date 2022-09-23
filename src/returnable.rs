use std::{convert, ops};
use std::convert::Infallible;
use std::error::Error;
use std::fmt::Debug;
use std::ops::{ControlFlow, Try};

pub(crate) enum Returnable<T, E> {
    None,
    Ok(T),
    Err(E)
}

#[allow(unused)]
impl<T, E> Returnable<T, E> where E: Debug {
    pub(crate) fn from_option(opt: Option<T>) -> Self{
        match opt {
            None => Returnable::None,
            Some(v) => Returnable::Ok(v)
        }
    }

    pub(crate) fn from_result(res: Result<T, E>) -> Self{
        match res {
            Ok(v) => Returnable::Ok(v),
            Err(e) => Returnable::Err(e)
        }
    }

    pub(crate) fn to_option(self) -> Option<T> {
        match self {
            Returnable::Ok(v) => Some(v),
            _ => None
        }
    }

    pub(crate) fn to_result(self, none_err: E) -> Result<T, E> {
        match self {
            Returnable::Ok(v) => Ok(v),
            Returnable::Err(e) => Err(e),
            Returnable::None => Err(none_err)
        }
    }

    pub(crate) fn expect(self, msg: &str) -> T{
        match self {
            Returnable::Ok(v) => v,
            Returnable::Err(e) => Err(e).expect(msg),
            Returnable::None => panic!("Returnable is None: {}", msg)
        }
    }
}

impl<T, E, F: From<E>> ops::FromResidual<Result<Infallible, Option<E>>> for Returnable<T, F> {
    #[inline]
    fn from_residual(residual: Result<Infallible, Option<E>>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(Some(e)) => Returnable::Err(From::from(e)),
            Err(None) => Returnable::None
        }
    }
}


impl<T, F> ops::FromResidual<Option<Infallible>> for Returnable<T, F> {
    #[inline]
    fn from_residual(residual: Option<Infallible>) -> Self {
        match residual {
            None => Returnable::None,
            Some(_) => unreachable!()
        }
    }
}

impl<T, E> From<Option<T>> for Returnable<T, E> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => Returnable::None,
            Some(v) => Returnable::Ok(v)
        }
    }
}

impl<T, E> Try for Returnable<T, E> {
    type Output = T;
    type Residual = Result<Infallible, Option<E>>;

    #[inline]
    fn from_output(output: Self::Output) -> Self { Self::Ok(output) }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::None => ControlFlow::Break(Err(None)),
            Self::Ok(v) => ControlFlow::Continue(v),
            Self::Err(e) => ControlFlow::Break(Err(Some(e))),
        }
    }
}