use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use crate::Compiler;
use crate::compiler::parser::parse;
use crate::compiler::tokenizer::tokenize;
use crate::variable::Type;

pub(crate) fn example(file: &str) -> Vec<u8> {
    let mut code_file = File::open(format!("src/codegen_examples/code/v1/{}", file)).expect("Could not open file");
    let mut code = String::new();
    code_file.read_to_string(&mut code).expect("Could not read to buffer");

    let tokens = match tokenize(&code) {
        Ok(tokens) => tokens,
        Err(e) => panic!("Error tokenizing:\n{}", e)
    };

    println!("Tokens:\n{}", tokens);

    let ast = parse(tokens).expect("Error parsing");

    println!("ast: \n{:#?}", ast);

    let compiler = Compiler::new(HashMap::from([
        ("i32::lt".to_string(), (vec![Type::I32, Type::I32], vec![Type::Bool])),
        ("i32::add".to_string(), (vec![Type::I32, Type::I32], vec![Type::I32])),
        ("i32::to_string".to_string(), (vec![Type::I32], vec![Type::String])),
        ("string::join".to_string(), (vec![Type::String, Type::String], vec![Type::String])),
        ("println".to_string(), (vec![Type::String], vec![])),
        ("to_dbg_string".to_string(), (vec![Type::Empty], vec![Type::String])),
        ("File::create".to_string(), (vec![Type::String], vec![])),
        ("File::write".to_string(), (vec![Type::String, Type::Object], vec![])),
    ]));

    compiler.compile(ast).expect("Error compiling!")
}