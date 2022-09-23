use std::{convert, ops};
use std::convert::Infallible;
use std::ops::{ControlFlow, Try};

pub(crate) enum Returnable<T, E> {
    None,
    Res(T),
    Err(E)
}

impl<T, E, F: From<E>> ops::FromResidual<Result<Infallible, Option<E>>> for Returnable<T, F> {
    #[inline]
    fn from_residual(Err(Some(e)): Result<Infallible, Option<E>>) -> Self {
        Returnable::Err(From::from(e))
    }
}

impl<T, E> Try for Returnable<T, E> {
    type Output = T;
    type Residual = Result<Infallible, Option<E>>;

    #[inline]
    fn from_output(output: Self::Output) -> Self { Self::Res(output) }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Self::None => ControlFlow::Break(Err(None)),
            Self::Res(v) => ControlFlow::Continue(v),
            Self::Err(e) => ControlFlow::Break(Err(Some(e))),
        }
    }
}