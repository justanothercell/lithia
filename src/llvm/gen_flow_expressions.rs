use crate::ast::{Block, Expression};
use crate::c_str_ptr;
use crate::error::{LithiaError, LithiaET};
use crate::llvm::{LLVMModGenEnv, ReturnInfo, Variable};
use llvm_sys::core;

pub(crate) fn compile_if(cond: &Expression, body: &Block, else_body: &Block, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<ReturnInfo, LithiaError> {
    let then_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("then")) };
    let else_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("else")) };
    let continue_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("ifcont")) };
    let c = cond.build(env, None)?;
    let v = c.resolve_var()?;
    let alloc_ret = unsafe {
        // RET VAL
        let alloc_ret = core::LLVMBuildAlloca(env.builder, [].as_mut_ptr(), c_str_ptr!(""));

        core::LLVMBuildCondBr(env.builder, v.llvm_value, then_block, else_block); // IF CONDITION CALL
        core::LLVMPositionBuilderAtEnd(env.builder, then_block); // START THEN CLAUSE
        alloc_ret
    };
    println!("alloced");
    let body_r = body.build(env, None)?;
    unsafe {
        if let Some(var) = &body_r.variable {
            core::LLVMBuildStore(env.builder, var.llvm_value.clone(), alloc_ret);
            core::LLVMBuildBr(env.builder, continue_block); // END THEN CLAUSE
        }
        core::LLVMPositionBuilderAtEnd(env.builder, else_block); // START ELSE CLAUSE
    };
    println!("bodied");
    let else_body_r = else_body.build(env, None)?;
    unsafe {
        if let Some(var) = &else_body_r.variable {
            core::LLVMBuildStore(env.builder, var.llvm_value.clone(), alloc_ret);
            core::LLVMBuildBr(env.builder, continue_block); // END ELSE CLAUSE
        }
        core::LLVMPositionBuilderAtEnd(env.builder, continue_block); // START CONTINUE BLOCK
    };
    println!("elsed");
    let ret_t = match (body_r.return_t, else_body_r.return_t) {
        (None, None) => None,
        (Some(t), None) => Some(t),
        (None, Some(t)) => Some(t),
        (Some(t1), Some(t2)) => {
            if t1.0 != t2.0 {
                return Err(LithiaET::TypeError(t1.0.clone(), t2.0.clone())
                    .ats(vec![t1.0.1.clone(), t2.0.1.clone()]))
            }
            Some(t1)
        }
    };
    if let Some(rt) = &ret_t && let Some(r) = &c.return_t {
        if rt.0 != r.0 {
            return Err(LithiaET::TypeError(rt.0.clone(), r.0.clone())
                .ats(vec![rt.0.1.clone(), r.0.1.clone()]))
        }
    }
    let v = match match (body_r.variable, else_body_r.variable) {
        (None, None) => None,
        (Some(v), None) => Some(v),
        (None, Some(v)) => Some(v),
        (Some(v1), Some(v2)) => {
            if v1.ast_type != v2.ast_type {
                return Err(LithiaET::TypeError(v1.ast_type.clone(), v2.ast_type.clone())
                    .ats(vec![v1.ast_type.1.clone(), v2.ast_type.1.clone()]))
            }
            Some(v1)
        }
    } {
        None => None,
        Some(v) => {
            let r = unsafe { core::LLVMBuildLoad2(env.builder, v.llvm_type, v.llvm_value, c_str_ptr!(ret_name.unwrap_or(String::new()))) };
            Some(Variable {
                ast_type: v.ast_type,
                llvm_type: v.llvm_type,
                llvm_value: r,
            })
        }
    };
    println!("retted");
    Ok(ReturnInfo {
        variable: v,
        return_t: ret_t,
        loc: cond.2.clone(),
    })
}