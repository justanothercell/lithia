#![feature(inherent_associated_types)]

pub(crate) mod ast;
pub(crate) mod llvm;
pub(crate) mod source;
pub(crate) mod tokens;
pub(crate) mod error;
pub(crate) mod compiler;
pub(crate) mod lib;

fn main() {
    println!("Hello, world!");
}
