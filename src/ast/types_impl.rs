use std::cmp::{min, Ordering};
use std::ops::{BitAnd, BitOr};
use crate::ast::{Item, Ty, Type};
use crate::ast::code_printer::CodePrinter;
use crate::error::{ParseError, ParseET};
use crate::source::span::Span;

#[derive(Eq, PartialEq, PartialOrd, Clone, Debug)]
#[repr(u8)]
pub(crate) enum TySat {
    No = 0,
    CastUnsafe = 1,
    Cast = 2,
    Yes = 3
}

impl TySat {
    pub(crate) fn and(self, pred: bool) -> Self{
        return if pred { self } else { TySat::No }
    }
}

impl BitAnd for TySat {
    type Output = TySat;

    fn bitand(self, rhs: Self) -> Self::Output {
        min(self, rhs)
    }
}

impl Ord for TySat {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.clone() as u8;
        let b = other.clone() as u8;
        return if a == b {
            Ordering::Equal
        } else if a < b {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
}

impl Type {
    pub(crate) fn placeholder(loc: Span) -> Self {
        Type(Ty::Single(vec![], Item::placeholder(loc.clone())), loc)
    }
    pub(crate) fn satisfies(&self, other: &Type) -> TySat {
        if self == other { TySat::Yes } else {
            match (&self.0, &other.0) {
                (Ty::Single(_, t1), Ty::Single(_, t2)) => if t1 == t2 {
                    TySat::Yes
                } else { TySat::Cast },
                (Ty::RawPointer, Ty::RawPointer) => TySat::Yes,
                (Ty::Pointer(t1), Ty::Pointer(t2)) => t1.satisfies(t2),
                (Ty::Pointer(_), Ty::RawPointer) => TySat::Yes,
                (Ty::RawPointer, Ty::Pointer(_)) => TySat::CastUnsafe,
                (Ty::Pointer(_)|Ty::RawPointer, Ty::Single(generics, name)) if generics.len() == 0 && name == &Item::new(&vec!["uptr"], self.1.clone()) => TySat::CastUnsafe,
                (Ty::Single(generics, name), Ty::Pointer(_)|Ty::RawPointer) if generics.len() == 0 && name == &Item::new(&vec!["uptr"], self.1.clone()) => TySat::CastUnsafe,
                (Ty::Array(t1, l1), Ty::Array(t2, l2)) => t1.satisfies(t2).and(l1 == l2),
                (Ty::Array(t1, _l1), Ty::Slice(t2)) => t1.satisfies(t2), // array satisfies slice
                (Ty::Slice(t1), Ty::Slice(t2)) => t1.satisfies(t2),
                (Ty::Slice(t1), Ty::Array(t2, _)) => t1.satisfies(t2) & TySat::CastUnsafe,
                (Ty::Tuple(t1), Ty::Tuple(t2)) => t1.iter().zip(t2).fold(TySat::Yes, |acc, (t1, t2)| acc & t1.satisfies(t2)),
                (Ty::Signature(a1, r1, unsafe_fn1, vararg1), Ty::Signature(a2, r2, unsafe_fn2, vararg2)) =>
                        a1.iter().zip(a2).fold(TySat::Yes, |acc, (t1, t2) | acc & t1.satisfies(t2)).and((a1.len() == a2.len() && vararg1 == vararg2) || *vararg2)
                            & r1.satisfies(r2).and(unsafe_fn1 == unsafe_fn2 || !*unsafe_fn2),
                _ => TySat::No
            }
        }
    }
    pub(crate) fn satisfies_or_err(&self, other: &Type, sat: TySat) -> Result<(), ParseError> {
        let s = self.satisfies(other);
        if s == sat {
            Ok(())
        } else {
            Err(ParseET::TypeError(other.clone(), self.clone()).ats(vec![self.1.clone(), other.1.clone()]))
        }
    }
    pub(crate) fn equals_or_err(&self, other: &Type) -> Result<(), ParseError> {
        if self == other {
            Ok(())
        } else {
            Err(ParseET::TypeError(other.clone(), self.clone()).ats(vec![self.1.clone(), other.1.clone()]))
        }
    }
    pub(crate) fn is_return(&self) -> bool {
        if let Ty::Returns(_) = &self.0 { true } else { false }
    }

    pub(crate) fn unwrap_return(self) -> Self {
        if let Ty::Returns(box ty) = self.0 { ty } else { self }
    }
}

impl Ty {
    pub(crate) fn empty() -> Self{
        Ty::Tuple(vec![])
    }
    pub(crate) fn is_empty(&self) -> bool {
        if let Ty::Tuple(ty) = self {
            ty.len() == 0
        } else {
            false
        }
    }
}