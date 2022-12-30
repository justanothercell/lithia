pub(crate) mod span;

use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufRead, Read};
use std::rc::Rc;
use std::string::ParseError;

#[derive(PartialEq)]
pub(crate) struct Source {
    st: SourceType,
    source: String
}

impl Debug for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source({:?})", self.st)
    }
}

impl Source {
    pub(crate) fn from_file(path: String) -> Result<Self, ParseError> {
        Ok(Self {
            st: SourceType::File(path.clone()),
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
    String,
}

impl Debug for SourceType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SourceType::File(f) =>  format!("{}", f),
            SourceType::String => format!("<string>")
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CodePoint(Rc<Source>, usize);

#[allow(non_camel_case_types)]
type line = usize;
#[allow(non_camel_case_types)]
type index_in_line = usize;

impl CodePoint {
    pub(crate) fn span(self) -> Span {
        Span::single(self)
    }

    pub(crate) fn pos(&self) -> (line, index_in_line){
        let first_part = &self.0.source[0..self.1];
        let mut lines_split = first_part.split("\n").collect::<Vec<&str>>();
        (lines_split.len(), lines_split.pop().unwrap().len())
    }
}