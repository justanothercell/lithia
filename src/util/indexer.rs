use std::rc::Rc;
use crate::error::{LithiaError, LithiaET};
use crate::source::span::Span;

pub(crate) trait Indexable {
    type Item;
    const ITEM_NAME: &'static str;
    fn get(&self, i: usize) -> Self::Item;
    fn loc_at(&self, i: usize) -> Span;
    fn len(&self) -> usize;
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

    pub(crate) fn get(&self, index: usize) -> Result<T::Item, LithiaError> {
        if index >= self.len() {
            Err(LithiaET::EOF.at(self.here()).when(format!("trying to get {}", T::ITEM_NAME)))
        }
        else {
            Ok(self.list.get(index))
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.list.len()
    }

    pub(crate) fn elems_left(&self) -> usize {
        self.list.len() - self.index
    }

    pub(crate) fn this(&self) -> Result<T::Item, LithiaError> {
        self.get(self.index)
    }

    pub(crate) fn here(&self) -> Span {
        if self.index >= self.list.len() {
            self.list.loc_at(self.list.len() - 1)
        } else {
            self.list.loc_at(self.index)
        }
    }

    pub(crate) fn next(&mut self){
        self.index += 1;
    }

    pub(crate) fn peek(&self) -> Result<T::Item, LithiaError>{
        self.get(self.index + 1)
    }

    pub(crate) fn peekn(&self, n: isize) -> Result<T::Item, LithiaError>{
        self.get((self.index as isize + n) as usize)
    }
}