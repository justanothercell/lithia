use llvm_sys::{prelude::LLVMBool, prelude, core};
use crate::ast::Module;
use crate::{c_str_ptr};
use crate::error::ParseError;
use crate::llvm::{LLVMModGenEnv, Variable};

impl Module {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv) -> Result<(), ParseError> {
        for (ident, func) in &self.functions {
            let function_type = unsafe {
                core::LLVMFunctionType(core::LLVMVoidType(), [].as_mut_ptr(), 0 as u32, false as LLVMBool)
            };
            let function = unsafe { core::LLVMAddFunction(env.module, c_str_ptr!(ident), function_type) };
            env.globals.insert(ident.to_string(), Variable {
                ast_type: func.signature.clone(),
                llvm_type: function_type,
                llvm_value: function,
            });
        }
        for (ident, func) in &self.functions {
            let function = env.get_var(ident, Some(&func.loc))?.llvm_value;
            let entry_block = unsafe { core::LLVMAppendBasicBlock(function, c_str_ptr!("entry")) };
            let builder = unsafe {
                let b = core::LLVMCreateBuilder();
                core::LLVMPositionBuilderAtEnd(b, entry_block);
                b
            };
            unsafe {
                core::LLVMBuildRetVoid(builder);
                core::LLVMDisposeBuilder(builder);
            }
        }
        Ok(())
    }
}