use crate::ast::{Block, Expression};
use crate::c_str_ptr;
use crate::error::{LithiaError, LithiaET};
use crate::llvm::{LLVMModGenEnv, ReturnInfo, Variable};
use llvm_sys::core;
use crate::ast::code_printer::CodePrinter;

/// How to: cook an if
/// 1. create blocks, save reference to start_block
/// 2. build condition expression
/// 2. goto then_block, build block body
/// 3. goto else_block, build block body
/// 4. resolve all types, check for compatability, test if both branches resolve to a value
///     1. goto start_block, build alloc resolved value
///     2. goto then_block, build store resolved value
///     3. goto else_block, build store resolved value
///     4. goto continue_block, build load resolved value
/// 5. goto then_block, build jmp continue_block
/// 6. goto else_block, build jmp continue_block
/// 7. goto start_block, build break
/// 8. goto continue_block
pub(crate) fn compile_if(cond: &Expression, body: &Block, else_body: &Block, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<ReturnInfo, LithiaError> {
    let then_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("then")) };
    let start_block = unsafe { core::LLVMGetPreviousBasicBlock(then_block) };
    let else_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("else")) };
    let continue_block = unsafe { core::LLVMAppendBasicBlock(env.function.unwrap(), c_str_ptr!("ifcont")) };
    let c = cond.build(env, None)?;
    unsafe {
        core::LLVMPositionBuilderAtEnd(env.builder, then_block); // START THEN CLAUSE
    };
    let body_r = body.build(env, None)?;
    unsafe {
        core::LLVMPositionBuilderAtEnd(env.builder, else_block); // START ELSE CLAUSE
    };
    let else_body_r = else_body.build(env, None)?;
    unsafe {
        core::LLVMPositionBuilderAtEnd(env.builder, continue_block); // START CONTINUE BLOCK
    };
    let returns = (body_r.return_t.is_some(), else_body_r.return_t.is_some());
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
    let v = match match (body_r.variable.clone(), else_body_r.variable.clone()) {
        (None, None) => None,
        (Some(v), None) | (None, Some(v)) => return Err(LithiaET::CompilationError(format!("if resolves to {} but there is no else", v.ast_type.print())).at(v.ast_type.1.clone())),
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
            let r = unsafe {
                core::LLVMPositionBuilderAtEnd(env.builder, start_block);
                let alloc_ret = core::LLVMBuildAlloca(env.builder, v.llvm_type, c_str_ptr!(""));
                if let Some(var) = &body_r.variable {
                    core::LLVMPositionBuilderAtEnd(env.builder, then_block);
                    core::LLVMBuildStore(env.builder, var.llvm_value.clone(), alloc_ret);
                }
                if let Some(var) = &else_body_r.variable {
                    core::LLVMPositionBuilderAtEnd(env.builder, else_block);
                    core::LLVMBuildStore(env.builder, var.llvm_value.clone(), alloc_ret);
                }
                core::LLVMPositionBuilderAtEnd(env.builder, continue_block);
                core::LLVMBuildLoad2(env.builder, v.llvm_type, alloc_ret, c_str_ptr!(ret_name.unwrap_or(String::new())))
            };
            Some(Variable {
                ast_type: v.ast_type,
                llvm_type: v.llvm_type,
                llvm_value: r,
            })
        }
    };
    unsafe {
        if !returns.0 {
            core::LLVMPositionBuilderAtEnd(env.builder, then_block);
            core::LLVMBuildBr(env.builder, continue_block);
        }
        if !returns.1 {
            core::LLVMPositionBuilderAtEnd(env.builder, else_block);
            core::LLVMBuildBr(env.builder, continue_block);
        }

        core::LLVMPositionBuilderAtEnd(env.builder, start_block);
        core::LLVMBuildCondBr(env.builder, c.resolve_var()?.llvm_value, then_block, else_block); // IF CONDITION CALL
        core::LLVMPositionBuilderAtEnd(env.builder, continue_block);
    }
    Ok(ReturnInfo {
        variable: v,
        return_t: ret_t,
        loc: cond.2.clone(),
    })
}