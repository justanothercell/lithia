#![feature(fmt_helpers_for_derive)]
#![feature(fn_traits)]
#![feature(pattern)]
#![feature(box_patterns)]
#![feature(try_blocks)]
#![feature(try_trait_v2)]
#![feature(try_trait_v2_yeet)]
#![feature(trait_upcasting)]

extern crate core;

use std::fs::File;
use std::io::Write;
use std::ops::Deref;
use memmap2::Mmap;
use compiler::ast::*;
use compiler::bin_builder::{BinBuilder, JmpType};
use vm::virtual_machine::{Executor, Word};
use crate::compiler::compiler::Compiler;
use crate::variable::Ident;

mod vm;
mod compiler;
mod variable;
mod codegen_examples;

fn main() {

    let code = codegen_examples::code::example("write_file_option.li");
    println!();
    //let code = codegen_examples::ast::while_loop::example();
    //let code = codegen_examples::bytecode::for_loop::example();

    {
        let mut file = File::create("test.lbc").expect("Could not create file");
        file.write_all(&code).expect("Could not write to file");
        file.flush().expect("Could not flush file");
    }


    let mut vm = Executor {
        stack_frames: vec![],
        stack: vec![],
        program: unsafe { Mmap::map(&File::open("test.lbc").expect("Could not open file!")).expect("Could not map file!") },
        externs: vm::bindings::standard_bindings(),
        current_marker: 0
    };

    println!("{:02X?}", vm.program.deref());
    println!("Program length: {} bytes", vm.program.len());
    println!();

    let (ret, time) = vm.run(vec![]);
    println!("execution returned: {:?} ({:?})", ret, time)
}


