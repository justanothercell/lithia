pub(crate) mod span;

use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Read};
use std::path::Path;
use std::rc::Rc;
use crate::error::LithiaError;
use crate::util::indexer::{Indexable, Indexer};
use crate::source::span::Span;

#[derive(PartialEq)]
pub(crate) struct Source {
    st: SourceType,
    source: String
}

pub(crate) type SourceIter = Indexer<Rc<Source>>;
impl Indexable for Rc<Source> {
    type Item = char;
    const ITEM_NAME: &'static str = "char";

    fn get(&self, i: usize) -> Self::Item {
        self.source.as_bytes()[i] as char
    }

    fn loc_at(&self, i: usize) -> Span {
        CodePoint(self.clone(), i).span()
    }

    fn len(&self) -> usize {
        self.source.len()
    }
}

impl Debug for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.st)
    }
}

impl Source {
    pub(crate) fn from_file<P: AsRef<Path> + Display>(path: P) -> Result<Self, LithiaError> {
        Ok(Self {
            st: SourceType::File(path.to_string()),
            source: {
                let mut f = File::open(path)?;
                let mut buffer = String::new();
                f.read_to_string(&mut buffer)?;
                buffer
            }
        })
    }

    pub(crate) fn from_string(source: String) -> Self{
        Self {
            st: SourceType::String,
            source
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) enum SourceType {
    File(String),
    String
}

impl Debug for SourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SourceType::File(f) =>  format!("{}", f),
            SourceType::String => format!("<string>")
        })
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct CodePoint(pub(crate) Rc<Source>, pub(crate) usize);

impl CodePoint {
    #[allow(non_camel_case_types)]
    type line = usize;
    #[allow(non_camel_case_types)]
    type index_in_line = usize;
    pub(crate) fn span(self) -> Span {
        Span::single(self)
    }

    pub(crate) fn pos(&self) -> (Self::line, Self::index_in_line){
        let first_part = &self.0.source[0..self.1];
        let mut lines_split = first_part.split("\n").collect::<Vec<&str>>();
        (lines_split.len(), lines_split.pop().unwrap().len())
    }
}

impl Debug for CodePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (l, i) = self.pos();
        write!(f, "{}:{}", l, i+1)
    }
}