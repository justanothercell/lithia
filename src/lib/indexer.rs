use std::rc::Rc;
use crate::error::ParseET;

pub(crate) trait Indexable {
    type Elem;
    fn get(self, i: usize) -> Result<Self::Elem, ParseError>;
    fn len(self) -> usize;
}

#[derive(Clone)]
pub(crate) struct Indexer<T> {
    list: Rc<T>,
    pub(crate) index: usize,
}

impl<T: Indexable> Indexer<T> {
    pub(crate) fn new(list: T) -> Self {
        Self {
            list: Rc::new(list),
            index: 0,
        }
    }

    pub(crate) fn get(&self, index: usize) -> Result<char, ParseError> {
        if index >= self.list.len() {
            Err(ParseET::EOF.at(self.here().span()))
        }
        else {
            Ok(self.source.source.as_bytes()[index] as char)
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.list.len()
    }

    pub(crate) fn elems_left(&self) -> usize {
        self.list.len() - self.index
    }

    pub(crate) fn this(&self) -> Result<char, ParseError> {
        self.get(self.index)
    }

    pub(crate) fn here(&self) -> CodePoint {
        CodePoint(self.source.clone(), self.index)
    }

    pub(crate) fn next(&mut self){
        self.index += 1;
    }

    pub(crate) fn peek(&self) -> Result<char, ParseError>{
        self.get(self.index + 1)
    }

    pub(crate) fn peekn(&self, n: isize) -> Result<char, ParseError>{
        self.get((self.index as isize + n) as usize)
    }
}