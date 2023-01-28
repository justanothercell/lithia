use std::process::Command;
use crate::ast::code_printer::CodePrinter;
use crate::ast::parser::parse;
use crate::error::LithiaError;
use crate::llvm::gen_llvm::{build_exe, build_llvm_ir};
use crate::source::Source;
use crate::tokens::tokenizer::tokenize;

pub(crate) struct Arguments{

}

pub(crate) fn compile(args: Arguments) -> Result<(), LithiaError>{
    let source = Source::from_file("examples/testing/primitive_if.li")?;
    let tokens = tokenize(source)?;
    println!("{tokens:?}");
    let module = parse(tokens, ("main".to_string(), None))?;
    println!("{}", module.print());
    let llvm_mod = build_llvm_ir(module)?;
    build_exe(llvm_mod, env!("LLVM_SYS_150_PREFIX"), "examples/testing/primitive_if.bc", "examples/testing/primitive_if.exe",  true, true)?;
    println!();
    let code = Command::new("examples/testing/primitive_if.exe")
        .spawn().unwrap().wait().unwrap();
    println!("executed with {code}");
    Ok(())
}