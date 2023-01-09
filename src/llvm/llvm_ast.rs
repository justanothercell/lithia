use std::ffi::{c_uint, c_ulonglong};
use llvm_sys::{prelude::LLVMBool, prelude, core};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use crate::ast::{AstLiteral, Block, Expr, Expression, Ident, Item, Module, Ty, Type};
use crate::{c_str_ptr};
use crate::ast::code_printer::CodePrinter;
use crate::error::{ParseError, ParseET};
use crate::llvm::{LLVMModGenEnv, Variable};
use crate::source::span::Span;
use crate::tokens::{Literal, NumLit, NumLitTy};

impl Module {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv) -> Result<(), ParseError> {
        // === manual extern ===
        unsafe {
            let puts_ty_param = Type(Ty::RawPointer, Span::dummy());
            let puts_ret = Type(Ty::Single(vec![], Item::new(&vec!["i32"], Span::dummy())), Span::dummy());
            let puts_fn_ty = core::LLVMFunctionType(puts_ret.llvm_type(env)?, [puts_ty_param.llvm_type(env)?].as_mut_ptr(), 1 as c_uint, false as LLVMBool);
            let puts_fn = core::LLVMAddFunction(env.module, c_str_ptr!("puts"), puts_fn_ty.clone());
            env.globals.insert("puts".to_string(), Variable {
                ast_type: Type(Ty::Signature(vec![puts_ty_param], Box::new(puts_ret)), Span::dummy()),
                llvm_type: puts_fn_ty,
                llvm_value: puts_fn,
            });
        }
        // === global consts ===
        for (ident, constant) in &self.constants {
            unsafe {
                let ty = if let Ty::Pointer(ty) = &constant.ty.0 {
                    ty.llvm_type(env)?
                } else if let Ty::Slice(ty) = &constant.ty.0 {
                    Type(Ty::Array(ty.clone(), 0), constant.ty.1.clone()).llvm_type(env)?
                } else {
                    return Err(ParseET::CompilationError(format!("constant can only be pointer, found {}", constant.print())).at(constant.val.1.clone()).when("compiling constant"))
                };
                let v = core::LLVMAddGlobal(env.module, ty, c_str_ptr!(ident));
                let val = if let Expr::Point(box Expression(Expr::Literal(lit), _)) = &constant.val.0 {
                    let Variable {
                        ast_type,
                        llvm_type,
                        llvm_value
                    } = lit.llvm_literal(env)?;
                    let loc = ast_type.1.clone();
                    Variable {
                        ast_type: Type(Ty::Pointer(Box::new(ast_type)), loc),
                        llvm_type,
                        llvm_value,
                    }
                } else {
                    return Err(ParseET::CompilationError(format!("constant can only be initialized by literal pointer, found {}", constant.print())).at(constant.val.1.clone()).when("compiling constant"))
                };
                val.ast_type.satisfies_or_err(&constant.ty)?;
                core::LLVMSetInitializer(v, val.llvm_value);
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
                core::LLVMFunctionType(func.ret.llvm_type(env)?, func.args.clone().into_iter().map(|(i, t)|t.llvm_type(env)).collect::<Result<Vec<_>, _>>()?.as_mut_ptr(), func.args.len() as u32, false as LLVMBool)
            };
            let function = unsafe { core::LLVMAddFunction(env.module, c_str_ptr!(ident), function_type) };
            env.globals.insert(ident.to_string(), Variable {
                ast_type: Type(Ty::Signature(func.args.clone().into_iter().map(|(i, t)|t).collect(), Box::new(func.ret.clone())), func.name.1.clone()),
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
            env.push_stack(true);
            func.args.iter()
                .map(|(ident, ty)|(ident, ty, ty.llvm_type(env)))
                .collect::<Vec<(&Ident, &Type, Result<LLVMTypeRef, ParseError>)>>()
                .into_iter()
                .enumerate()
                .map(|(i, (ident, ty, llvm_ty))| {
                    let _ = env.stack.last_mut().unwrap().0.insert(ident.0.clone(),
                        Variable {
                            ast_type: ty.clone(),
                            llvm_type: llvm_ty?,
                            llvm_value: unsafe {core::LLVMGetParam(function, i as c_uint)},
                        });
                    Ok(())
                })
                .collect::<Result<Vec<()>, ParseError>>()?;
            let mut ret = func.body.as_ref().unwrap().build(env, None)?;
            env.pop_stack();
            ret.ast_type.satisfies_or_err(&func.ret)?;
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
                    let v = expr.build(env, None)?;
                    let ptr = core::LLVMBuildAlloca(env.builder, v.llvm_type, c_str_ptr!(ret_name.unwrap_or(String::new())));
                    core::LLVMBuildStore(env.builder, v.llvm_value, ptr);
                    Variable {
                        ast_type: Type(Ty::Pointer(Box::new(v.ast_type)),self.1.clone()),
                        llvm_type: core::LLVMPointerType(v.llvm_type, 0), // TODO: replace 0
                        llvm_value: ptr,
                    }
                },
                Expr::Deref(expr) => {
                    let v = expr.build(env, None)?;
                    if let Ty::RawPointer = &v.ast_type.0 {
                        return Err(ParseET::TypeError("pointer".to_string(), "raw pointer".to_string()).at(self.1.clone()).when("compiling deref"))
                    }
                    let inner_ty = if let Ty::Pointer(box ty) = &v.ast_type.0 { ty } else {
                        return Err(ParseET::TypeError("pointer".to_string(), v.ast_type.print()).at(self.1.clone()).when("compiling deref"))
                    };
                    let llvm_ty = inner_ty.llvm_type(env)?;
                    let deref = core::LLVMBuildLoad2(env.builder, llvm_ty, v.llvm_value, c_str_ptr!(ret_name.unwrap_or(String::new())));
                    Variable {
                        ast_type: inner_ty.clone(),
                        llvm_type: llvm_ty, // TODO: replace 0
                        llvm_value: deref,
                    }
                }
                Expr::Variable(var) => env.get_var(&var.0, Some(&var.1))?,
                Expr::Block(block) => block.build(env, ret_name)?,
                Expr::FuncCall(fun, args) => {
                    let var = env.get_var(&fun.0.first().unwrap().0, Some(&fun.1))?;
                    if let Ty::Signature(arg_types, ret) = var.ast_type.0 {
                        if arg_types.len() != args.len() {
                            return Err(ParseET::CompilationError(format!("expected {} args, got {}", arg_types.len(), args.len())).at(self.1.clone()).when("compiling function call"))
                        }
                        let mut args = args.iter().zip(arg_types)
                            .map(|(expr, t)| expr.build(env, None).map(|v| {
                                v.ast_type.satisfies_or_err(&t)?;
                                Ok(v.llvm_value)
                            }).flatten())
                            .collect::<Result<Vec<_>, _>>()?;
                        let ty = ret.llvm_type(env)?;
                        let out = core::LLVMBuildCall2(env.builder, var.llvm_type, var.llvm_value, args.as_mut_ptr(), args.len() as c_uint, c_str_ptr!(ret_name.unwrap_or(String::new())));
                        Variable {
                            ast_type: *ret,
                            llvm_type: ty,
                            llvm_value: out,
                        }
                    } else {
                        return Err(ParseET::TypeError("function".to_string(), format!("{:?}", var.ast_type.0)).at(self.1.clone()).when("compiling expression"))
                    }
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
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<Variable, ParseError> {
        let mut ret = None;
        for (i, stmt) in self.0.iter().enumerate() {
            let r = stmt.0.build(env, None)?;
            if let Expr::Return(_) = stmt.0.0 {
                ret = Some(r);
                break
            }
            if !stmt.1 {
                ret = Some(r);
                if self.0.len() != i + 1 {
                    return Err(ParseET::CompilationError(format!("returning expression needs to be at end of block")).at(stmt.2.clone()).when("compiling block"))
                }
                break
            }
        }
        unsafe {Ok(ret.unwrap_or_else(||Variable {
            ast_type: Type(Ty::Tuple(vec![]), self.1.clone()),
            llvm_type: core::LLVMVoidType(),
            llvm_value: *[].as_mut_ptr(),
        }))}
    }
}

impl Type {
    pub(crate) fn llvm_type(&self, env: &mut LLVMModGenEnv) -> Result<prelude::LLVMTypeRef, ParseError> {
        unsafe {
            Ok(match &self.0 {
                Ty::Single(generics, base_type) => {
                    if generics.len() > 0 || base_type.0.len() > 1 {
                        panic!("type was not correctly resolved")
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
                Ty::RawPointer => core::LLVMPointerType(core::LLVMVoidType(), 0), // TODO: replace 0 with adapting value
                Ty::Pointer(ty) => core::LLVMPointerType(ty.llvm_type(env)?, 0), // TODO: replace 0 with adapting value
                Ty::Array(ty, usize) => core::LLVMArrayType(ty.llvm_type(env)?, *usize as c_uint),
                Ty::Slice(ty) => Type(Ty::Array(ty.clone(), 0), self.1.clone()).llvm_type(env)?,
                Ty::Tuple(tys) => {
                    if tys.len() > 0 {
                        *tys.iter().map(|ty|ty.llvm_type(env)).collect::<Result<Vec<_>, ParseError>>()?.as_mut_ptr()
                    } else {
                        core::LLVMVoidType()
                    }
                },
                Ty::Signature(_, _) => unimplemented!("signature types to llvm type not implemented yet")
            })
        }
    }
}

impl AstLiteral {
    pub(crate) fn llvm_literal(&self, env: &mut LLVMModGenEnv) -> Result<Variable, ParseError>{
        Ok(Variable{
            ast_type: self.get_type()?,
            llvm_type: self.get_type()?.llvm_type(env)?,
            llvm_value: unsafe {
            match &self.0 {
                Literal::String(s) => AstLiteral::llvm_literal(
                    &AstLiteral(Literal::Array(
                        {
                            let mut s = s.clone();
                            s.push('\0');
                            s.chars().map(|c| AstLiteral(Literal::Char(c), self.1.clone())).collect()
                        },
                        Type(Ty::Single(vec![], Item::new(&vec!["u8"], self.1.clone())), self.1.clone()),
                        s.len() + 1), self.1.clone()), env)?.llvm_value,
                Literal::Char(c) => core::LLVMConstInt(core::LLVMInt8Type(), *c as u8 as c_ulonglong, false as LLVMBool),
                Literal::Number(NumLit::Integer(num), _) => {
                    core::LLVMConstInt( self.get_type()?.llvm_type(env)?, *num as u8 as c_ulonglong, false as LLVMBool)
                }
                Literal::Bool(b) => core::LLVMConstInt(core::LLVMInt1Type(), *b as c_ulonglong, false as LLVMBool),
                Literal::Array(arr, elem_ty , len) =>
                    core::LLVMConstArray(elem_ty.llvm_type(env)?,
                                         arr.iter().map(|e|e.llvm_literal(env).map(|v|v.llvm_value)).collect::<Result<Vec<_>, ParseError>>()?.as_mut_ptr(),
                                         *len as c_uint),
                _ => unimplemented!("ty to llvm ty")
            }
        }})
    }
}

impl Type {
    pub(crate) fn satisfies(&self, other: &Type) -> bool {
        if self == other { true } else {
            match (&self.0, &other.0) {
                (Ty::Single(_, t1), Ty::Single(_, t2)) => t1 == t2,
                (Ty::RawPointer, Ty::RawPointer) => true,
                (Ty::Pointer(t1), Ty::Pointer(t2)) => t1.satisfies(t2),
                    (Ty::Pointer(_t), Ty::RawPointer) => true, // pointer satisfies raw pointer
                (Ty::Array(t1, l1), Ty::Array(t2, l2)) => t1.satisfies(t2) && l1 == l2,
                    (Ty::Array(t1, _l1), Ty::Slice(t2)) => t1.satisfies(t2), // array satisfies slice
                (Ty::Slice(t1), Ty::Slice(t2)) => t1.satisfies(t2),
                (Ty::Tuple(t1), Ty::Tuple(t2)) => t1.iter().zip(t2).all(|(t1, t2)|t1.satisfies(t2)),
                (Ty::Signature(a1, r1), Ty::Signature(a2, r2)) => a1.iter().zip(a2).all(|(t1, t2)|t1.satisfies(t2)) && r1.satisfies(r2),
                _ => false
            }
        }
    }

    pub(crate) fn satisfies_or_err(&self, other: &Type) -> Result<(), ParseError> {
        if self.satisfies(other) {
            Ok(())
        } else {
            Err(ParseET::TypeError(other.print(), self.print()).ats(vec![self.1.clone(), other.1.clone()]))
        }
    }
}