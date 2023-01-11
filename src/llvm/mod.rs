pub(crate) mod gen_llvm;
pub(crate) mod llvm_ast;
pub(crate) mod gen_flow_expressions;

use std::collections::HashMap;
use std::ffi::c_uint;

use llvm_sys::{prelude, core};
use crate::ast::Type;
use crate::error::{ParseError, ParseET};
use crate::source::span::Span;

#[macro_export]
macro_rules! c_str {
    ($s:literal) => (
        #[allow(unused_unsafe)]
        unsafe { std::ffi::CStr::from_ptr(concat!($s, "\0").as_ptr() as *const i8) }
    );
    ($s:expr) => (
        #[allow(unused_unsafe)]
        unsafe { std::ffi::CStr::from_ptr(($s.to_string() + "\0").as_ptr() as *const i8) }
    );
}

#[macro_export]
macro_rules! c_str_ptr {
    ($s:expr) => (
        $crate::c_str!($s).as_ptr()
    );
}

pub(crate) struct LLVMModGenEnv {
    globals: HashMap<String, Variable>,
    stack: Vec<StackEnv>,
    mod_name: String,
    module: prelude::LLVMModuleRef,
    builder: prelude::LLVMBuilderRef
}

pub(crate) struct StackEnv {
    vars: HashMap<String, Variable>,
    opaque: bool,
    unsafe_ctx: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Variable{
    ast_type: Type,
    llvm_type: prelude::LLVMTypeRef,
    llvm_value: prelude::LLVMValueRef
}

impl LLVMModGenEnv{
    pub(crate) fn new(mod_name: String) -> Self{
        let module = unsafe { core::LLVMModuleCreateWithName(c_str_ptr!(mod_name)) };
        let main_entrypoint_function_type = unsafe {
            core::LLVMFunctionType(core::LLVMVoidType(), [].as_mut_ptr(), 0, 0)
        };
        let main_entrypoint_function = unsafe { core::LLVMAddFunction(module, c_str_ptr!("main"), main_entrypoint_function_type) };
        let entry_block = unsafe { core::LLVMAppendBasicBlock(main_entrypoint_function, c_str_ptr!("entry")) };
        let builder = unsafe {
            let b = core::LLVMCreateBuilder();
            core::LLVMPositionBuilderAtEnd(b, entry_block);
            b
        };
        Self {
            globals: HashMap::new(),
            stack: vec![],
            mod_name: mod_name.clone(),
            module,
            builder
        }
    }

    pub(crate) fn push_stack(&mut self, opaque: bool, unsafe_ctx: bool){
        self.stack.push(StackEnv {
            vars: Default::default(),
            opaque,
            unsafe_ctx: unsafe_ctx || (!opaque && self.stack.last().map(|s| s.unsafe_ctx).unwrap_or(false)),
        })
    }

    pub(crate) fn pop_stack(&mut self){
        self.stack.pop();
    }

    pub(crate) fn get_var(&self, ident: &str, loc: Option<&Span>) -> Result<Variable, ParseError>{
        for frame in self.stack.iter().rev(){
            if let Some(v) = frame.vars.get(ident){
                return Ok(v.clone())
            }
            if frame.opaque { break }
        }
        if let Some(v) = self.globals.get(ident){
            Ok(v.clone())
        } else {
            let et = ParseET::VariableNotFound(ident.to_string());
            Err(match loc {
                None => et.error(),
                Some(loc) => et.at(loc.clone())
            })
        }
    }

    pub(crate) fn finish(self) -> Result<prelude::LLVMModuleRef, ParseError>{
        unsafe {
            let fun = self.get_var("main", None)?;
            core::LLVMBuildCall2(self.builder, fun.llvm_type, fun.llvm_value, [].as_mut_ptr(), 0 as c_uint, c_str_ptr!(""));
            core::LLVMBuildRetVoid(self.builder);
        }
        Ok(self.module)
    }
}

impl Drop for LLVMModGenEnv {
    fn drop(&mut self) {
        unsafe {core::LLVMDisposeBuilder(self.builder)}
    }
}