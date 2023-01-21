use crate::ast::{Block, Expression};
use crate::c_str_ptr;
use crate::error::ParseError;
use crate::llvm::{LLVMModGenEnv, Variable};
use llvm_sys::core;

pub(crate) fn compile_if(cond: &Expression, body: &Block, else_body: &Block, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<Variable, ParseError> {
    let then_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("then")) };
    let else_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("else")) };
    let continue_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("ifcont")) };
    let c = cond.build(env, None)?;
    let alloc_ret = unsafe {
        // RET VAL
        let alloc_ret = core::LLVMBuildAlloca(env.builder, [].as_mut_ptr(), c_str_ptr!(""));

        core::LLVMBuildCondBr(env.builder, c.llvm_value, then_block, else_block); // IF CONDITION CALL
        core::LLVMPositionBuilderAtEnd(env.builder, then_block); // START THEN CLAUSE
        alloc_ret
    };
    let body_r = body.build(env, None)?.0;
    unsafe {
        if !body_r.ast_type.is_return() {
            core::LLVMBuildStore(env.builder, body_r.0.llvm_value, alloc_ret);
            core::LLVMBuildBr(env.builder, continue_block); // END THEN CLAUSE
        }
        core::LLVMPositionBuilderAtEnd(env.builder, else_block); // START ELSE CLAUSE
    };
    let else_body_r = body.build(env, None)?.0;
    unsafe {
        if !else_body_r.ast_type.is_return() {
            core::LLVMBuildStore(env.builder, else_body_r.0.llvm_value, alloc_ret);
            core::LLVMBuildBr(env.builder, continue_block); // END THEN CLAUSE
        }
    };
    unsafe {
        match (body_r.ast_type.is_return(), else_body_r.ast_type.is_return()) {

            core::LLVMPositionBuilderAtEnd(env.builder, continue_block); // START ELSE CLAUSE
            let r = core::LLVMBuildLoad2(env.builder, body_r.unwrap().0.llvm_type, alloc_ret, c_str_ptr!(ret_name.unwrap_or(String::new())));
            Ok(Variable {
                ast_type: body_r.unwrap().0.ast_type,
                llvm_type: body_r.unwrap().0.llvm_type,
                llvm_value: r,
            })
        } else { Ok(None) }
    }
}