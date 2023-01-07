use std::ffi::{c_uint, c_ulonglong};
use llvm_sys::{prelude::LLVMBool, prelude, core};
use llvm_sys::prelude::LLVMValueRef;
use crate::ast::{AstLiteral, Block, Expr, Expression, Ident, Item, Module, Ty, Type};
use crate::{c_str_ptr};
use crate::error::{ParseError, ParseET};
use crate::llvm::{LLVMModGenEnv, Variable};
use crate::source::span::Span;
use crate::tokens::Literal;

impl Module {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv) -> Result<(), ParseError> {
        unsafe {
            let puts_ty_param = Type(Ty::Pointer(Box::new(
                Type(Ty::Array(Box::new(Type(Ty::Single {
                    generics: vec![],
                    base_type: Item(vec![Ident("u8".to_string(), Span::dummy())], Span::dummy()),
                    loc: Span::dummy(),
                }, Span::dummy())), 0), Span::dummy())
            )), Span::dummy());
            let puts_ret = Type(Ty::Single {
                generics: vec![],
                base_type: Item(vec![Ident("i32".to_string(), Span::dummy())], Span::dummy()),
                loc: Span::dummy(),
            }, Span::dummy());
            let puts_fn_ty = core::LLVMFunctionType(puts_ret.llvm_type(env)?, [puts_ty_param.llvm_type(env)?].as_mut_ptr(), 1 as c_uint, false as LLVMBool);
            let puts_fn = core::LLVMAddFunction(env.module, c_str_ptr!("puts"), puts_fn_ty.clone());
            env.globals.insert("puts".to_string(), Variable {
                ast_type: Type(Ty::Signature(vec![puts_ty_param], Box::new(puts_ret)), Span::dummy()),
                llvm_type: puts_fn_ty,
                llvm_value: puts_fn,
            });
        }

        for (ident, constant) in &self.constants {
            unsafe {
                let ty = constant.ty.llvm_type(env)?;
                let v = core::LLVMAddGlobal(env.module, ty, c_str_ptr!(ident));
                let val = if let Expr::Literal(lit) = &constant.val.0 {
                    lit.llvm_literal(env)?
                } else {
                    return Err(ParseET::CompilationError("constant can only be initialized by literal".to_string()).at(constant.val.1.clone()).when("compiling constant"))
                };
                core::LLVMSetInitializer(v, val);
                env.globals.insert(ident.to_string(), Variable {
                    ast_type: constant.ty.clone(),
                    llvm_type: ty,
                    llvm_value: v,
                });
            }
        }
        // === register functions ===
        for (ident, func) in &self.functions {
            let function_type = unsafe {
                core::LLVMFunctionType(core::LLVMVoidType(), [].as_mut_ptr(), 0 as u32, false as LLVMBool)
            };
            let function = unsafe { core::LLVMAddFunction(env.module, c_str_ptr!(ident), function_type) };
            env.globals.insert(ident.to_string(), Variable {
                ast_type: func.ret.clone(),
                llvm_type: function_type,
                llvm_value: function,
            });
        }
        // === build functions ===
        for (ident, func) in &self.functions {
            let function = env.get_var(ident, Some(&func.loc))?.llvm_value;
            let entry_block = unsafe { core::LLVMAppendBasicBlock(function, c_str_ptr!("entry")) };
            let entry_builder = env.builder;
            env.builder = unsafe {
                let b = core::LLVMCreateBuilder();
                core::LLVMPositionBuilderAtEnd(b, entry_block);
                b
            };
            func.body.as_ref().unwrap().build(env, None, true)?;
            unsafe {
                core::LLVMBuildRetVoid(env.builder);
                core::LLVMDisposeBuilder(env.builder);
            }
            env.builder = entry_builder;
        }
        Ok(())
    }
}

impl Expression {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<Variable, ParseError> {
        unsafe {
            Ok(match &self.0 {
                Expr::Literal(lit) => lit.llvm_literal(env)?,
                Expr::Point(expr) => {
                    let v = expr.build(env, ret_name)?;
                    let ptr = core::LLVMBuildAlloca(env.builder, )
                    core::LLVMBuildStore(env.builder, v)
                }
                Expr::Variable(var) => env.get_var(&var.0, Some(&var.1))?.llvm_value,
                Expr::Block(block) => block.build(env, None, false)?,
                Expr::FuncCall(fun, args) => {
                    let var = env.get_var(&fun.0.first().unwrap().0, Some(&fun.1))?;
                    let mut args = args.iter().map(|a| a.build(env, None)).collect::<Result<Vec<_>, _>>()?;
                    core::LLVMBuildCall2(env.builder, var.llvm_type, var.llvm_value, args.as_mut_ptr(), args.len() as c_uint, c_str_ptr!(ret_name.unwrap_or(String::new())))
                },
                //Expr::BinaryOp(_, _, _) => {}
                //Expr::UnaryOp(_, _) => {}
                //Expr::VarCreate(_, _, _, _) => {}
                //Expr::VarAssign(_, _, _) => {}
                _ => unimplemented!()
            })
        }
    }
}

impl Block {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv, ret_name: Option<String>, opaque: bool) -> Result<Variable, ParseError> {
        env.push_stack(opaque);
        let mut r = None;
        for stmt in &self.0 {
            r = Some(stmt.0.build(env, None)?);
        }
        env.pop_stack();
        unsafe {Ok(r.unwrap_or_else(||Variable {
            ast_type: Type(Ty::Tuple(vec![]), self.1.clone()),
            llvm_type: core::LLVMVoidType(),
            llvm_value: core::LLVMConstNull(core::LLVMVoidType()),
        }))}
    }
}

impl Type {
    pub(crate) fn llvm_type(&self, env: &mut LLVMModGenEnv) -> Result<prelude::LLVMTypeRef, ParseError> {
        unsafe {
            Ok(match &self.0 {
                Ty::Single { generics, base_type, loc } => {
                    if base_type.0.len() > 1 {
                        unimplemented!("types with more than one item in path")
                    }
                    match base_type.0.first().unwrap().0.as_str() {
                        "u8" | "i8" => core::LLVMInt8Type(),
                        "u16" | "i16" => core::LLVMInt16Type(),
                        "u32" | "i32" => core::LLVMInt32Type(),
                        "u64" | "i64" => core::LLVMInt64Type(),
                        "u128" | "i128" => core::LLVMInt8Type(),
                        "uptr" | "iptr" => {
                            #[cfg(target_pointer_width = "16")]
                                let t = core::LLVMInt8Type();
                            #[cfg(target_pointer_width = "32")]
                                let t = core::LLVMInt32Type();
                            #[cfg(target_pointer_width = "64")]
                                let t = core::LLVMInt64Type();
                            t
                        }
                        _ => unimplemented!("primitive type not figured out yet, come back tomorrow")
                    }
                }
                Ty::Pointer(ty) => core::LLVMPointerType(ty.llvm_type(env)?, 0), // TODO: replace 0 with adapting value
                Ty::Array(ty, usize) => core::LLVMArrayType(ty.llvm_type(env)?, *usize as c_uint),
                Ty::Tuple(tys) => *tys.iter().map(|ty|ty.llvm_type(env)).collect::<Result<Vec<_>, ParseError>>()?.as_mut_ptr(),
                Ty::Signature(_, _) => unimplemented!("signature types to llvm type not implemented yet")
            })
        }
    }
}

impl AstLiteral {
    pub(crate) fn llvm_literal(&self, env: &mut LLVMModGenEnv) -> Result<Variable, ParseError>{
        unsafe {
            Ok(match &self.0 {
                Literal::String(s) => AstLiteral::llvm_literal(
                    &AstLiteral(Literal::Array(
                        {
                            let mut s = s.clone();
                            s.push('\0');
                            s.chars().map(|c| AstLiteral(Literal::Char(c), self.1.clone())).collect()
                        }
                        , Type(Ty::Single {
                    generics: vec![],
                    base_type: Item(vec![Ident("u8".to_string(), self.1.clone())], self.1.clone()),
                    loc: self.1.clone(),
                }, self.1.clone()), s.len() + 1), self.1.clone()), env)?,
                Literal::Char(c) => Variable { ast_type: Type(Ty::Single { generics: vec![], base_type: Item(vec![Ident("u8".to_string(), self.1.clone())], self.1.clone()), loc: self.1.clone() }, self.1.clone()), llvm_type: core::LLVMInt8Type(), llvm_value: core::LLVMConstInt(core::LLVMInt8Type(), *c as u8 as c_ulonglong, false as LLVMBool, ) },
                //Literal::Number(_, _) => {}
                Literal::Bool(b) => Variable { ast_type: Type(Ty::Single { generics: vec![], base_type: Item(vec![Ident("u8".to_string(), self.1.clone())], self.1.clone()), loc: self.1.clone() }, self.1.clone()), llvm_type: core::LLVMInt8Type(), llvm_value: core::LLVMConstInt(core::LLVMInt8Type(), *b as u8 as c_ulonglong, false as LLVMBool, ) },,
                Literal::Array(arr, elem_ty , len) =>
                    core::LLVMConstArray(elem_ty.llvm_type(env)?,
                                         arr.iter().map(|e|e.llvm_literal(env)).collect::<Result<Vec<_>, ParseError>>()?.as_mut_ptr(),
                                         *len as c_uint),
                _ => unimplemented!("ty to llvm ty")
            })
        }
    }
}