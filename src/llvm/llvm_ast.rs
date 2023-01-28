use std::collections::HashMap;
use std::ffi::{c_uint, c_ulonglong};
use llvm_sys::{prelude::LLVMBool, prelude, core, LLVMOpcode, LLVMIntPredicate};
use llvm_sys::prelude::{LLVMTypeRef};
use crate::ast::{AstLiteral, Block, Const, Expr, Expression, Func, Ident, Item, Module, Op, Ty, Type};
use crate::{c_str_ptr};
use crate::ast::code_printer::CodePrinter;
use crate::ast::types_impl::TySat;
use crate::ast::types_impl::TySat::No;
use crate::error::{OnParseErr, LithiaError, LithiaET};
use crate::llvm::{LLVMModGenEnv, ReturnInfo, Variable};
use crate::llvm::gen_flow_expressions::compile_if;
use crate::tokens::{Literal, NumLit};

impl Module {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv) -> Result<(), LithiaError> {
        // === global consts ===
        for (_ident, constant) in &self.constants {
            constant.build(env)?;
        }
        // === register functions ===
        for (_ident, func) in &self.functions {
            func.register(env)?;
        }
        // === build functions ===
        for (_ident, func) in &self.functions {
            func.build(env)?;
        }
        Ok(())
    }
}

impl Const {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv) -> Result<(), LithiaError> {
        unsafe {
            let ty = if let Ty::Pointer(ty) = &self.ty.0 {
                ty.llvm_type(env)?
            } else if let Ty::Slice(ty) = &self.ty.0 {
                Type(Ty::Array(ty.clone(), 0), self.ty.1.clone()).llvm_type(env)?
            } else {
                return Err(LithiaET::CompilationError(format!("constant can only be pointer, found {}", self.print())).at(self.val.2.clone()).when("compiling constant"))
            };
            let v = core::LLVMAddGlobal(env.module, ty, c_str_ptr!(self.name.0));
            let val = if let Expr::Point(box Expression(tags, Expr::Literal(lit), _)) = &self.val.1 {
                let Variable {
                    ast_type,
                    llvm_type,
                    llvm_value,
                    ..
                } = lit.llvm_literal(env)?;
                let loc = ast_type.1.clone();
                Variable {
                    ast_type: Type(Ty::Pointer(Box::new(ast_type)), loc),
                    llvm_type,
                    llvm_value,
                    mutable: false
                }
            } else {
                return Err(LithiaET::CompilationError(format!("constant can only be initialized by literal pointer, found {}", self.print())).at(self.val.2.clone()).when("compiling constant"))
            };
            val.ast_type.satisfies_or_err(&self.ty, TySat::Yes)?;
            core::LLVMSetInitializer(v, val.llvm_value);
            env.globals.insert(self.name.0.to_string(), Variable {
                ast_type: self.ty.clone(),
                llvm_type: ty,
                llvm_value: v,
                mutable: false
            });
        }
        Ok(())
    }
}

impl Func {
    pub(crate) fn register(&self, env: &mut LLVMModGenEnv) -> Result<(), LithiaError> {
        let function_type = unsafe {
            core::LLVMFunctionType(self.ret.llvm_type(env)?, self.args.clone().into_iter().map(|(i, t)|t.llvm_type(env)).collect::<Result<Vec<_>, _>>()?.as_mut_ptr(), self.args.len() as u32, self.tags.contains_key("vararg") as LLVMBool)
        };
        let function = unsafe { core::LLVMAddFunction(env.module, c_str_ptr!(self.name.0), function_type) };
        env.globals.insert(self.name.0.to_string(), Variable {
            ast_type: Type(Ty::Signature(self.args.clone().into_iter().map(|(i, t)|t).collect(), Box::new(self.ret.clone()), self.tags.contains_key("unsafe"), self.tags.contains_key("vararg")), self.name.1.clone()),
            llvm_type: function_type,
            llvm_value: function,
            mutable: false
        });
        Ok(())
    }
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv) -> Result<(), LithiaError> {
        if self.tags.contains_key("extern") {
            if self.body.is_some() {
                return Err(LithiaET::CompilationError("extern function may not havea body".to_string()).at(self.name.1.clone()))
            }
            return if self.tags.contains_key("unsafe") {
                Ok(())
            } else {
                Err(LithiaET::UnsafeError("extern function".to_string()).at(self.name.1.clone()))
            }
        }
        let body = self.body.as_ref().unwrap();
        let function = env.get_var(&self.name.0, Some(&self.loc))?.llvm_value;
        let outer_f = env.function;
        env.function = Some(function);
        let entry_block = unsafe { core::LLVMAppendBasicBlock(function, c_str_ptr!("entry")) };
        let entry_builder = env.builder;
        env.builder = unsafe {
            let b = core::LLVMCreateBuilder();
            core::LLVMPositionBuilderAtEnd(b, entry_block);
            b
        };
        env.push_stack(true, self.tags.contains_key("unsafe"));
        self.args.iter()
            .map(|(ident, ty)|(ident, ty, ty.llvm_type(env)))
            .collect::<Vec<(&Ident, &Type, Result<LLVMTypeRef, LithiaError>)>>()
            .into_iter()
            .enumerate()
            .map(|(i, (ident, ty, llvm_ty))| {
                let _ = env.stack.last_mut().unwrap().vars.insert(ident.0.clone(),
                                                               Variable {
                                                                   ast_type: ty.clone(),
                                                                   llvm_type: llvm_ty?,
                                                                   llvm_value: unsafe {core::LLVMGetParam(function, i as c_uint)},
                                                                   mutable: false
                                                               });
                Ok(())
            })
            .collect::<Result<Vec<()>, LithiaError>>()?;
        let r = body.build(env, None)?;
        r.variable.as_ref().map(|v| unsafe { core::LLVMBuildRet(env.builder, v.llvm_value) });
        let v = match (r.variable, r.return_t) {
            (None, None) => None,
            (None, Some(rt)) => Some(rt),
            (Some(v), None) => Some((v.ast_type, v.llvm_type)),
            (Some(v), Some(rt)) => {
                if v.ast_type != rt.0 {
                    return Err(LithiaET::TypeError(v.ast_type.clone(), rt.0.clone())
                        .ats(vec![v.ast_type.1.clone(), rt.0.1.clone()]))
                }
                Some(rt)
            }
        };
        env.pop_stack();
        if let Some(r) = &v {
            r.0.satisfies_or_err(&self.ret, TySat::Yes).e_at_add(r.0.1.clone())?;
        } else {
            if !self.ret.0.is_empty() {
                return Err(LithiaET::CompilationError(format!("function returns {} but got empty type", self.ret.print())).at(self.ret.1.clone()))
            }
            unsafe { core::LLVMBuildRetVoid(env.builder); }
        }
        unsafe {
            core::LLVMDisposeBuilder(env.builder);
        }
        env.builder = entry_builder;
        env.function = outer_f;
        Ok(())
    }
}

impl Expression {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<ReturnInfo, LithiaError> {
        let outer_unsafe = env.stack.last().unwrap().unsafe_ctx;
        if self.0.contains_key("unsafe") {
            env.stack.last_mut().unwrap().unsafe_ctx = true;
        }
        let r =
            Ok(match &self.1 {
                Expr::Expr(box expr) => expr.build(env, ret_name)?,
                Expr::Literal(lit) => {
                    let v = lit.llvm_literal(env)?;
                    ReturnInfo {
                        variable: Some(v),
                        return_t: None,
                        loc: self.2.clone()
                    }
                },
                Expr::Return(e) => {
                    let rt = e.as_ref().map(|e|e.build(env, None))
                        .map(|r| r.map(|r| r.variable))
                        .map_or(Ok(None), |v| v.map(Some))
                        .map(|x| x.flatten().map(|v| {
                            unsafe { core::LLVMBuildRet(env.builder, v.llvm_value); }
                            Ok((v.ast_type, v.llvm_type))
                        }))?
                        .unwrap_or_else(|| {
                            unsafe { core::LLVMBuildRetVoid(env.builder); }
                            let t = Type(Ty::Tuple(vec![]), self.2.clone());
                            t.llvm_type(env).map(|lt| (t, lt))
                        })?;
                    ReturnInfo {
                        variable: None,
                        return_t: Some(rt),
                        loc: self.2.clone()
                    }
                }
                Expr::Point(expr) => {
                    let r = expr.build(env, None)?;
                    let v = r.resolve_var()?;
                    let ptr =  unsafe {
                        let ptr = core::LLVMBuildAlloca(env.builder, v.llvm_type, c_str_ptr!(ret_name.unwrap_or(String::new())));
                        core::LLVMBuildStore(env.builder, v.llvm_value, ptr);
                        ptr
                    };
                    ReturnInfo {
                        variable: Some(Variable{
                            ast_type: Type(Ty::Pointer(Box::new(v.ast_type)), self.2.clone()),
                            llvm_type:  unsafe { core::LLVMPointerType(v.llvm_type, 0) } , // TODO: replace 0
                            llvm_value: ptr,
                            mutable: false
                        }),
                        return_t: r.return_t,
                        loc: self.2.clone()
                    }
                },
                Expr::Deref(expr) => {
                    let r = expr.build(env, None)?;
                    let v = r.resolve_var()?;
                    if let Ty::RawPointer = &v.ast_type.0 {
                        return Err(LithiaET::TypeError(Type(Ty::Pointer(Box::new(Type::placeholder(self.2.clone()))), self.2.clone()), v.ast_type).at(self.2.clone()).when("compiling deref"))
                    }
                    let inner_ty = if let Ty::Pointer(box ty) = &v.ast_type.0 { ty } else {
                        return Err(LithiaET::TypeError(Type(Ty::Pointer(Box::new(Type::placeholder(self.2.clone()))), self.2.clone()), v.ast_type).at(self.2.clone()).when("compiling deref"))
                    };
                    let llvm_ty = inner_ty.llvm_type(env)?;
                    let deref =  unsafe { core::LLVMBuildLoad2(env.builder, llvm_ty, v.llvm_value, c_str_ptr!(ret_name.unwrap_or(String::new()))) };
                    ReturnInfo {
                        variable: Some(Variable {
                            ast_type: inner_ty.clone(),
                            llvm_type: llvm_ty,
                            llvm_value: deref,
                            mutable: false
                        }),
                        return_t: r.return_t,
                        loc: self.2.clone()
                    }
                }
                Expr::Variable(var) => {
                    let mut var = env.get_var(&var.0, Some(&var.1))?;
                    if var.mutable {
                        var.llvm_value = unsafe { core::LLVMBuildLoad2(env.builder, var.llvm_type, var.llvm_value, c_str_ptr!(ret_name.unwrap_or(String::new()))) };
                    }
                    var.mutable = false;
                    ReturnInfo {
                        variable: Some(var),
                        return_t: None,
                        loc: self.2.clone()
                    }
                },
                Expr::Block(block) => block.build(env, ret_name)?,
                Expr::FuncCall(fun, args) => {
                    let var = env.get_var(&fun.0.first().unwrap().0, Some(&fun.1))?;
                    if let Ty::Signature(arg_types, ret, is_unsafe, vararg) = var.ast_type.0 {
                        if is_unsafe && !env.stack.last().unwrap().unsafe_ctx {
                            return Err(LithiaET::UnsafeError("unsafe function".to_string()).ats(vec![var.ast_type.1.clone(), fun.1.clone()]))
                        }
                        if arg_types.len() != args.len() && (arg_types.len() > args.len() || !vararg) {
                            return if vararg {
                                Err(LithiaET::CompilationError(format!("expected {} args or more, got {}", arg_types.len(), args.len())).at(self.2.clone()).when("compiling function call"))
                            } else {
                                Err(LithiaET::CompilationError(format!("expected {} args, got {}", arg_types.len(), args.len())).at(self.2.clone()).when("compiling function call"))
                            }
                        }
                        let mut ret_t: Option<(Type, LLVMTypeRef)> = None;
                        let mut llvm_args = args.iter().zip(arg_types)
                            .map(|(expr, t)| expr.build(env, None).map(|r| {
                                let v = r.resolve_var()?;
                                if let Some(rt) = &r.return_t {
                                    if let Some(rtt) = &ret_t {
                                        rt.0.satisfies_or_err(&rtt.0, TySat::Yes)?;
                                    } else { ret_t = r.return_t.clone() }
                                }
                                v.ast_type.satisfies_or_err(&t, TySat::Yes).e_at_add(expr.2.clone())?;
                                Ok(v.llvm_value)
                            }).flatten())
                            .collect::<Result<Vec<_>, _>>()?;
                        if llvm_args.len() < args.len() {
                            llvm_args.append(&mut args.into_iter().skip(llvm_args.len())
                                .map(|expr| expr.build(env, None).map(|r|{
                                    let v = r.resolve_var()?;
                                    if let Some(rt) = &r.return_t {
                                        if let Some(rtt) = &ret_t {
                                            rt.0.satisfies_or_err(&rtt.0, TySat::Yes)?;
                                        } else { ret_t = r.return_t.clone() }
                                    }
                                    Ok(v.llvm_value)
                                }).flatten())
                                .collect::<Result<Vec<_>, _>>()?)
                        }
                        let ty = ret.llvm_type(env)?;
                        let out =  unsafe { core::LLVMBuildCall2(env.builder, var.llvm_type, var.llvm_value, llvm_args.as_mut_ptr(), args.len() as c_uint, c_str_ptr!(ret_name.unwrap_or(String::new()))) };

                        ReturnInfo {
                            variable: Some(Variable {
                                ast_type: *ret,
                                llvm_type: ty,
                                llvm_value: out,
                                mutable: false
                            }),
                            return_t: ret_t,
                            loc: self.2.clone()
                        }
                    } else {
                        return Err(LithiaET::TypeError(Type(Ty::Signature(vec![], Box::new(Type::placeholder(self.2.clone())), false, false), self.2.clone()), var.ast_type).at(self.2.clone()).when("compiling expression"))
                    }
                },
                Expr::VarCreate(name, mutable, ty, expr) => {
                    let mut r = expr.build(env, Some(name.0.clone()))?;
                    let mut v = r.resolve_var()?;
                    v.mutable = *mutable;
                    if v.mutable {
                        unsafe {
                            let ptr = core::LLVMBuildAlloca(env.builder, v.llvm_type, c_str_ptr!(""));
                            core::LLVMBuildStore(env.builder, v.llvm_value, ptr);
                            v.llvm_value = ptr;
                        }
                    }
                    if let Some(t) = &ty {
                        v.ast_type.satisfies_or_err(t, TySat::Yes)?;
                    }
                    env.stack.last_mut().unwrap().vars.insert(name.0.clone(), v.clone());
                    r.variable = None; // var creation doesnt resolve to variable
                    r
                }
                Expr::VarAssign(name, op, expr) => {
                    let mut expr = expr.clone();
                    let var = env.get_var(&name.0, Some(&self.2))?;
                    if let Some(op) = op {
                        expr = Box::new(Expression(HashMap::new(), Expr::BinaryOp(
                            op.clone(),
                            Box::new(Expression(HashMap::new(), Expr::Variable(name.clone()), op.1.clone())),
                            expr
                        ), op.1.clone()))
                    }
                    let mut r = expr.build(env, None)?;
                    if !var.mutable {
                        return Err(LithiaET::CompilationError(format!("cant assign to immutable variable")).at(self.2.clone()))
                    }
                    let mut v = r.resolve_var()?;
                    v.ast_type.satisfies_or_err(&var.ast_type, TySat::Yes)?;
                    unsafe { core::LLVMBuildStore(env.builder, v.llvm_value, var.llvm_value); }
                    r.variable = None;
                    r
                }
                Expr::Cast(expr, target_t) => {
                    let r = expr.build(env, None)?;
                    let v = r.resolve_var()?;
                    let sat = v.ast_type.satisfies(target_t);
                    if sat != TySat::Cast && sat != TySat::CastUnsafe {
                        return Err(LithiaET::CastError(v.ast_type, target_t.clone()).at(self.2.clone()))
                    }
                    if sat == TySat::CastUnsafe && !env.stack.last().unwrap().unsafe_ctx {
                        return Err(LithiaET::UnsafeError("unsafe cast".to_string()).at(self.2.clone()))
                    }
                    let llvm_type = target_t.llvm_type(env)?;
                    let op_code =  unsafe { core::LLVMGetCastOpcode(v.llvm_value, false as LLVMBool, llvm_type, false as LLVMBool) };
                    ReturnInfo {
                        variable: Some(Variable {
                            ast_type: target_t.clone(),
                            llvm_type,
                            llvm_value: unsafe { core::LLVMBuildCast(env.builder, op_code, v.llvm_value, llvm_type, c_str_ptr!(ret_name.unwrap_or(String::new()))) },
                            mutable: false
                        }),
                        return_t: r.return_t,
                        loc: self.2.clone()
                    }
                }
                Expr::BinaryOp(op, a, b) => {
                    let ra = a.build(env, None)?;
                    let rb = b.build(env, None)?;
                    let va = ra.resolve_var()?;
                    let vb = rb.resolve_var()?;
                    let opc = match &op.0 {
                        Op::Add => Some(LLVMOpcode::LLVMAdd),
                        Op::Sub => Some(LLVMOpcode::LLVMSub),
                        Op::Mul => Some(LLVMOpcode::LLVMMul),
                        Op::Div => Some(LLVMOpcode::LLVMUDiv),
                        Op::Or => Some(LLVMOpcode::LLVMOr),
                        Op::And => Some(LLVMOpcode::LLVMAnd),
                        Op::BinOr => Some(LLVMOpcode::LLVMOr),
                        Op::BinAnd => Some(LLVMOpcode::LLVMAnd),
                        Op::LShift => Some(LLVMOpcode::LLVMShl),
                        Op::RShift => Some(LLVMOpcode::LLVMAShr), // A stands for Arithmetic and L stands for logical, see: https://stackoverflow.com/questions/141525/what-are-bitwise-shift-bit-shift-operators-and-how-do-they-work
                        Op::LT => None,
                        Op::LE => None,
                        Op::GT => None,
                        Op::GE => None,
                        Op::EQ => None,
                        Op::NE => None,
                        invalid => panic!("didnt expect op {invalid:?}")
                    };
                    if let Some(op) = opc {
                        let r =  unsafe { core::LLVMBuildBinOp(env.builder, op, va.llvm_value, vb.llvm_value, c_str_ptr!(ret_name.unwrap_or(String::new()))) };
                        ReturnInfo {
                            variable: Some(Variable {
                                ast_type: va.ast_type,
                                llvm_type: va.llvm_type,
                                llvm_value: r,
                                mutable: false
                            }),
                            return_t: ra.return_t,
                            loc: self.2.clone()
                        }
                    } else {
                        let r =  unsafe { core::LLVMBuildICmp(env.builder, match &op.0 {
                            Op::LT => LLVMIntPredicate::LLVMIntSLT,
                            Op::LE => LLVMIntPredicate::LLVMIntSLE,
                            Op::GT => LLVMIntPredicate::LLVMIntSGT,
                            Op::GE => LLVMIntPredicate::LLVMIntSGE,
                            Op::EQ => LLVMIntPredicate::LLVMIntEQ,
                            Op::NE => LLVMIntPredicate::LLVMIntNE,
                            invalid => panic!("didnt expect op {invalid:?}")
                        }, va.llvm_value, vb.llvm_value, c_str_ptr!(ret_name.unwrap_or(String::new()))) };
                        let loc = op.1.clone();
                        ReturnInfo {
                            variable: Some(Variable {
                                ast_type: Type(Ty::Single(vec![], Item::new(&vec!["bool"], loc.clone())), loc.clone()),
                                llvm_type: unsafe { core::LLVMInt1Type() },
                                llvm_value: r,
                                mutable: false
                            }),
                            return_t: ra.return_t,
                            loc: self.2.clone()
                        }
                    }
                }
                Expr::If(expr, body, else_body) => compile_if(expr, body, else_body, env, ret_name)?,
                _ => unimplemented!()
            });
        if self.0.contains_key("unsafe") {
            env.stack.last_mut().unwrap().unsafe_ctx = outer_unsafe;
        }
        r
    }
}

impl Block {
    pub(crate) fn build(&self, env: &mut LLVMModGenEnv, ret_name: Option<String>) -> Result<ReturnInfo, LithiaError> {
        let mut ret_t: Option<(Type, LLVMTypeRef)> = None;
        for (i, stmt) in self.0.iter().enumerate() {
            let r = stmt.0.build(env, ret_name.clone())?;
            if let Some(rt) = &r.return_t {
                if let Some(rtt) = &ret_t {
                    rt.0.satisfies_or_err(&rtt.0, TySat::Yes)?;
                } else { ret_t = r.return_t.clone() }
                if r.variable.is_none() {
                    return Ok(ReturnInfo {
                        variable: None,
                        return_t: r.return_t,
                        loc: stmt.2.clone(),
                    })
                }
            }
            if let Expr::Return(_) = stmt.0.1 {
                return Ok(ReturnInfo {
                    variable: None,
                    return_t: r.return_t,
                    loc: stmt.2.clone(),
                })
            }
            if !stmt.1 && !stmt.0.1.is_block_like() {
                if self.0.len() != i + 1 {
                    return Err(LithiaET::CompilationError(format!("returning expression needs to be at end of block")).at(stmt.2.clone()).when("compiling block"))
                }
                return Ok(ReturnInfo {
                    variable: r.variable,
                    return_t: r.return_t,
                    loc: stmt.2.clone(),
                })
            }
            if self.0.len() == i + 1 && stmt.0.1.is_block_like() && !stmt.1 {
                return Ok(ReturnInfo {
                    variable: r.variable,
                    return_t: r.return_t,
                    loc: stmt.2.clone(),
                })
            }
        }
        Ok(ReturnInfo {
            variable: None,
            return_t: None,
            loc: self.1.clone(),
        })
    }
}

impl Type {
    pub(crate) fn llvm_type(&self, env: &mut LLVMModGenEnv) -> Result<prelude::LLVMTypeRef, LithiaError> {
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
                        t => unimplemented!("primitive type '{}' not figured out yet, come back tomorrow", t)
                    }
                }
                Ty::RawPointer => core::LLVMPointerType(core::LLVMVoidType(), 0), // TODO: replace 0 with adapting value
                Ty::Pointer(ty) => core::LLVMPointerType(ty.llvm_type(env)?, 0), // TODO: replace 0 with adapting value
                Ty::Array(ty, usize) => core::LLVMArrayType(ty.llvm_type(env)?, *usize as c_uint),
                Ty::Slice(ty) => Type(Ty::Array(ty.clone(), 0), self.1.clone()).llvm_type(env)?,
                Ty::Tuple(tys) => {
                    if tys.len() > 0 {
                        *tys.iter().map(|ty|ty.llvm_type(env)).collect::<Result<Vec<_>, LithiaError>>()?.as_mut_ptr()
                    } else {
                        core::LLVMVoidType()
                    }
                },
                Ty::Signature(_, _, _, _) => unimplemented!("signature types to llvm type not implemented yet")
            })
        }
    }
}

impl AstLiteral {
    pub(crate) fn llvm_literal(&self, env: &mut LLVMModGenEnv) -> Result<Variable, LithiaError> {
        Ok(Variable {
            ast_type: self.get_type()?,
            llvm_type: self.get_type()?.llvm_type(env)?,
            llvm_value: unsafe {
                let r = match &self.0 {
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
                        core::LLVMConstInt(self.get_type()?.llvm_type(env)?, *num as u8 as c_ulonglong, false as LLVMBool)
                    }
                    Literal::Bool(b) => core::LLVMConstInt(core::LLVMInt1Type(), *b as c_ulonglong, false as LLVMBool),
                    Literal::Array(arr, elem_ty, len) =>
                        core::LLVMConstArray(elem_ty.llvm_type(env)?,
                                             arr.iter().map(|e| e.llvm_literal(env).map(|v| v.llvm_value)).collect::<Result<Vec<_>, LithiaError>>()?.as_mut_ptr(),
                                             *len as c_uint),
                    _ => unimplemented!("ty to llvm ty")
                };
                r
            },
            mutable: false
        })
    }
}