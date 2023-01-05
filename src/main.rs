#![feature(pattern)]
#![feature(try_blocks)]
#![feature(adt_const_params)]
#![feature(inherent_associated_types)]
#![feature(box_patterns)]

extern crate core;

use std::process::exit;
use crate::compiler::{compile, Arguments};

pub(crate) mod ast;
pub(crate) mod llvm;
pub(crate) mod source;
pub(crate) mod tokens;
pub(crate) mod error;
pub(crate) mod compiler;
pub(crate) mod util;

fn main() {
   let args = Arguments {

   };
   match compile(args) {
      Ok(_) => (),
      Err(e) => {
         println!("{e}");
         exit(1)
      }
   }
}
