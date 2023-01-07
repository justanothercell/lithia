use std::path::Path;
use std::process::Command;
use llvm_sys::{bit_writer, prelude, core};
use crate::ast::Module;
use crate::c_str_ptr;
use crate::error::ParseError;
use crate::llvm::LLVMModGenEnv;

pub(crate) fn build_llvm_ir(module: Module) -> Result<prelude::LLVMModuleRef, ParseError>{
    let mut env = LLVMModGenEnv::new(module.name.0.clone());
    module.build(&mut env)?;
    env.finish()
}

pub(crate) fn build_exe<P: AsRef<Path>>(module: prelude::LLVMModuleRef, llvm_root: P, bitcode_file: P, exe_file: P, dump_ir: bool, disassemble: bool) -> Result<(), ParseError>{
    let llvm_root = llvm_root.as_ref().to_string_lossy().to_string();
    let bitcode_file = bitcode_file.as_ref().to_string_lossy().to_string();
    let exe_file = exe_file.as_ref().to_string_lossy().to_string();
    let success = unsafe { bit_writer::LLVMWriteBitcodeToFile(module, c_str_ptr!(bitcode_file)) };
    println!("wrote to file with exit code: {success}");
    if dump_ir {
        println!();
        unsafe { core::LLVMDumpModule(module) }
        println!();
    }
    unsafe { core::LLVMDisposeModule(module) }
    println!("disposed of module");
    if disassemble {
        let dis_code = Command::new(format!("{}/bin/llvm-dis.exe", llvm_root))
            .args([bitcode_file.clone()])
            .spawn()?.wait()?;
        println!("disassembled .bc to .ll with {dis_code}");
    }
    let compile_code = Command::new(format!("{}/bin/clang.exe", llvm_root))
        .args([bitcode_file, "-v".to_string(), "-o".to_string(), exe_file])
        .spawn()?.wait()?;
    println!("compiled to binary with {compile_code}");
    Ok(())
}
