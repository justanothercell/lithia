use crate::ast::{Block, Expression};
use crate::c_str_ptr;
use crate::error::ParseError;
use crate::llvm::{LLVMModGenEnv, Variable};

pub(crate) fn compile_if(cond: &Expression, body: &Block, else_body: &Block, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<Variable, ParseError> {
    let then_block = unsafe { core::LLVMAppendBasicBlock(*function, c_str_ptr!("then")) };
    let else_block = unsafe { core::LLVMAppendBasicBlock(*function, c_str_ptr!("else")) };
    let continue_block = unsafe { core::LLVMAppendBasicBlock(*function, c_str_ptr!("ifcont")) };
    let c = cond.build(env, None)?;
    unsafe {
        core::LLVMBuildCondBr(*builder, c.llvm_value, then_block, else_block); // IF CONDITION CALL
        core::LLVMPositionBuilderAtEnd(*builder, then_block); // START THEN CLAUSE
    };
    let body_r = body.build(env, None)?;
    unsafe {
        if body_r.is_some() {
            core::LLVMBuildBr(*builder, continue_block); // END THEN CLAUSE
        }
        core::LLVMPositionBuilderAtEnd(*builder, else_block); // START ELSE CLAUSE
    };
}