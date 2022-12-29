use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use crate::{Compiler, vm};
use crate::compiler::parser::{parse, ParseContext};
use crate::compiler::tokenizer::tokenize;

pub(crate) fn example(file: &str) -> Vec<u8> {
    let mut code_file = File::open(format!("src/codegen_examples/code/v1/{}", file)).expect("Could not open file");
    let mut code = String::new();
    code_file.read_to_string(&mut code).expect("Could not read to buffer");

    let tokens = match tokenize(&code) {
        Ok(tokens) => tokens,
        Err(e) => panic!("Error tokenizing:\n{}", e)
    };

    println!("Tokens:\n{}", tokens);

    let mut ctx = ParseContext::new(vm::bindings::standard_bindings());

    let ast = parse(tokens, &mut ctx).expect("Error parsing");

    println!("Ast:\n{}", ast);

    let externs = ctx.used_externs();

    println!("{:?}", externs.keys());

    //println!("ast: \n{:#?}", ast);

    let compiler = Compiler::new(externs);

    compiler.compile(ast).expect("Error compiling!")
}