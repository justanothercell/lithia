use std::collections::HashMap;
use std::hash::Hash;
use crate::ast::{Block, Expr, Expression, Type, Func, Item, Statement, Ty, Const, AstLiteral};
use crate::ast::patterns::{Consumer, Pat, Pattern};
use crate::ast::patterns::conditional::{While, Match, Succeed, Fail, IsOk, Optional};
use crate::ast::patterns::dynamic::{Latent, Mapping};
use crate::ast::patterns::simple::{ExpectIdent, ExpectParticle, ExpectParticleExact, GetIdent, GetLiteral, GetNext, GetParticle};
use crate::error::{ParseET};
use crate::source::span::Span;
use crate::tokens::{Literal, NumLit, NumLitTy};

pub(crate) struct Patterns{
    pub(crate) module_content: Pat<((HashMap<String, Func>, HashMap<String, Const>), Span)>
}

pub(crate) fn build_patterns() -> Patterns {
    let item = Pattern::named("identifier",
              (
                  GetIdent,
                  While(
                      (ExpectParticle(':'), ExpectParticleExact(':', true), GetIdent).pat(),
                      (ExpectParticle(':'), ExpectParticleExact(':', true), GetIdent).map(|(_, _, i), _| i).pat()
                  ),
              ),
        |(ident, mut vec), loc| {vec.insert(0, ident); Item(vec, loc)});

    let (type_pat, type_finalizer) = Latent::new();
    type_finalizer.finalize(Pattern::named("type", Match(vec![
        (Succeed(ExpectParticle('&').pat()).pat(), (ExpectParticle('&'),
                                                    Optional(type_pat.clone(), type_pat.clone()))
            .map(|(_, ty), _| ty.map(|ty| Ty::Pointer(Box::new(ty))).unwrap_or(Ty::RawPointer)).pat()),
        (Succeed(ExpectParticle('[').pat()).pat(), (ExpectParticle('['), type_pat.clone(),
                                                    Optional(
                                                        ExpectParticle(';').pat(),
                                                        (ExpectParticle(';'), GetLiteral).pat()),
                                                    ExpectParticle(']'))
            .map_res(|(_, ty, maybe_count, _), _| {
                if let Some((_, count) ) = maybe_count {
                    if let AstLiteral(Literal::Number(NumLit::Integer(c), th), loc) = count.clone() {
                        if th.as_ref().map(|t| t == &NumLitTy::UPtr).unwrap_or(true) {
                            Ok(Ty::Array(Box::new(ty), c as usize))
                        } else {
                            Err(ParseET::LiteralError(count.0, format!("expected uptr, found {}", th.unwrap())).at(loc).when("parsing array type"))
                        }
                    } else {
                        Err(ParseET::LiteralError(count.0, "expected uptr".to_string()).at(count.1).when("parsing array type"))
                    }
                } else {
                    Ok(Ty::Slice(Box::new(ty)))
                }
            }).pat()),
        (Succeed(item.clone()).pat(), item.clone().map(|item, loc| Ty::Single(vec![], item)).pat()),
    ]), |ty, loc| Type(ty, loc)));

    let (expression, expression_finalizer) = Latent::new();
    let let_create = Pattern::named("variable creation", (
        ExpectIdent("let".to_string()),
        GetIdent,
        ExpectParticle('='),
        expression.clone()
    ), |(_, name, _, expr), loc| Expr::VarCreate(name, false, None, Box::new(expr)));
    let function_call = Pattern::named("function call", (
        item.clone(),
        ExpectParticle('('),
        Optional(expression.clone(), expression.clone()),
        While(
            Fail(ExpectParticle(')').pat()).pat(),

            (ExpectParticle(','), expression.clone()).map(|(_, expr), _|expr).pat()
        ),
        ExpectParticle(')'),
    ), |(item, _, arg0, mut args, _), loc| {
        arg0.map(|arg0| args.insert(0, arg0));
        Expr::FuncCall(item, args)
    });
    expression_finalizer.finalize(Pattern::named("expression",
            Match(vec![
                (Succeed(ExpectIdent("let".to_string()).pat()).pat(), let_create.clone()),
                (Succeed((item.clone(), ExpectParticle('(')).pat()).pat(), function_call.clone()),
                (Succeed(ExpectParticle('&').pat()).pat(), (ExpectParticle('&'), expression.clone()).map(|(_, expr), loc| Expr::Point(Box::new(expr))).pat()),
                (Succeed(ExpectParticle('*').pat()).pat(), (ExpectParticle('*'), expression.clone()).map(|(_, expr), loc| Expr::Deref(Box::new(expr))).pat()),
                (Succeed(GetIdent.pat()).pat(), GetIdent.map(|ident, loc| Expr::Variable(ident)).pat()),
                (Succeed(GetLiteral.pat()).pat(), GetLiteral.map(|lit, loc| Expr::Literal(lit)).pat())
            ]), |expr, loc| Expression(expr, loc)));
    let statement = Pattern::named("statement", (
            expression.clone(),
            IsOk(ExpectParticle(';').pat())
        ), |(expr, terminated), loc| Statement(expr, terminated, loc));
    let block = Pattern::named("block",
        While(
            Fail(ExpectParticle('}').pat()).pat(),
            statement.clone()
        ), |stmts, loc| Block(stmts, loc));
    let function = Pattern::named("function", (
            ExpectIdent("fn".to_string()),
            GetIdent,
            ExpectParticle('('),
            Optional(GetIdent.pat(), (GetIdent, ExpectParticle(':'), type_pat.clone()).map(|(i, _, t), _| (i, t)).pat()),
            While(
                Fail(ExpectParticle(')').pat()).pat(),

                (ExpectParticle(','), GetIdent, ExpectParticle(':'), type_pat.clone()).map(|(_, i, _, t), _| (i, t)).pat()
            ),
            ExpectParticle(')').map(|_, loc|loc),
            Optional(ExpectParticle('-').pat(), (ExpectParticle('-'), ExpectParticleExact('>', true), type_pat.clone()).map(|(_, _, ty), _|ty).pat()),
            Match(vec![
                (Succeed(ExpectParticle('{').pat()).pat(), (ExpectParticle('{'), block.clone(), ExpectParticle('}')).map(|(_, block, _), _| Some(block)).pat()),
                (Succeed(ExpectParticle(';').pat()).pat(), ExpectParticle(';').map(|_, _| None).pat())
            ])
    ), |(_, name, _, arg0, mut args, sig_end_loc, ret_ty, body), loc| {
        arg0.map(|arg0| args.insert(0, arg0));
        let mut signature_loc = name.1.clone();
        signature_loc.combine(sig_end_loc);
        Func {
            name,
            args,
            ret: ret_ty.unwrap_or(Type(Ty::Tuple(vec![]), signature_loc)),
            body,
            loc,
    }});
    let constant = Pattern::named("constant", (
        ExpectIdent("const".to_string()),
        GetIdent,
        ExpectParticle(':'),
        type_pat.clone(),
        ExpectParticle('='),
        expression.clone(),
        ExpectParticle(';'),
        ), |(_, name, _, ty, _, val, _), loc| Const { name, ty, val });
    enum ModuleContent{
        Function(Func),
        Const(Const)
    }
    let module_content = Pattern::named("module content",
        While(
            GetNext.pat(),
            Match(vec![
                (Succeed(ExpectIdent("fn".to_string()).pat()).pat(), function.clone().map(|f, _| ModuleContent::Function(f)).pat()),
                (Succeed(ExpectIdent("const".to_string()).pat()).pat(), constant.clone().map(|c, _| ModuleContent::Const(c)).pat())
            ]).pat()
        ).map_res(|content, loc| {
            let mut functions = HashMap::new();
            let mut constants = HashMap::new();
            for c in content.into_iter() {
                match c {
                    ModuleContent::Function(f) => {
                        let l = f.name.1.clone();
                        if constants.contains_key(&f.name.0){
                            return Err(ParseET::AlreadyDefinedError("constant".to_string(), f.name.0).ats(vec![l, f.name.1]))
                        }
                        if let Some(f) = functions.insert(f.name.0.clone(), f){
                            return Err(ParseET::AlreadyDefinedError("function".to_string(), f.name.0).ats(vec![l, f.name.1]))
                        }
                    },
                    ModuleContent::Const(c) => {
                        let l = c.name.1.clone();
                        if functions.contains_key(&c.name.0){
                            return Err(ParseET::AlreadyDefinedError("function".to_string(), c.name.0).ats(vec![l, c.name.1]))
                        }
                        if let Some(c) = constants.insert(c.name.0.clone(), c){
                            return Err(ParseET::AlreadyDefinedError("constant".to_string(), c.name.0).ats(vec![l, c.name.1]))
                        }
                    }
                };
            }
            Ok((functions, constants))
        }), |content, loc| (content, loc));
    Patterns {
        module_content
    }
}
