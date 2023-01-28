use std::collections::HashMap;
use crate::ast::{Block, Expr, Expression, Type, Func, Item, Statement, Ty, Const, AstLiteral, TagValue, Tag, Operator, Op};
use crate::ast::patterns::{Consumer, Pat, Pattern};
use crate::ast::patterns::conditional::{While, Match, Succeed, Fail, IsOk, Optional, Or, Both};
use crate::ast::patterns::dynamic::{Latent, Mapping};
use crate::ast::patterns::simple::{ExpectIdent, ExpectParticle, ExpectParticleExact, GetGluedParticle, GetIdent, GetLiteral, GetNext, GetParticle};
use crate::error::{LithiaET};
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
                            Err(LithiaET::LiteralError(count.0, format!("expected uptr, found {}", th.unwrap())).at(loc).when("parsing array type"))
                        }
                    } else {
                        Err(LithiaET::LiteralError(count.0, "expected uptr".to_string()).at(count.1).when("parsing array type"))
                    }
                } else {
                    Ok(Ty::Slice(Box::new(ty)))
                }
            }).pat()),
        (Succeed(item.clone().map_res(|t, loc| if t == Item::new(&vec!["as"], loc){Err(LithiaET::ParsingError(String::new()).error())} else {Ok(())}).pat()).pat(), item.clone().map(|item, _| Ty::Single(vec![], item)).pat()),
    ]), |ty, loc| Type(ty, loc)));
    let (tag_args, tag_arg_finalizer) = Latent::new();
    let tag = Pattern::inline((
        GetIdent,
        Optional(ExpectParticle('(').pat(),(
        ExpectParticle('('),
        Optional(Fail(ExpectParticle(')').pat()).pat(), tag_args.clone()),
        While(
    Fail(ExpectParticle(')').pat()).pat(),
    (ExpectParticle(','), tag_args.clone()).map(|(_, f), _|f).pat()
        ),
                                   ExpectParticle(')')).pat())
    ), |(name, args_opt), loc|{
        let (_, arg0, mut args, _) = args_opt.unwrap_or(((), None, vec![], ()));
        arg0.map(|arg0| args.insert(0, arg0));
        Tag(name, args, loc)
    });
    tag_arg_finalizer.finalize(Pattern::named("tag arg", Match(vec![
        (Succeed((GetIdent, ExpectParticle('(')).pat()).pat(), tag.clone().map(|f, _| TagValue::Tag(Box::new(f))).pat()),
        (Succeed(GetIdent.pat()).pat(), GetIdent.map(|id, _| TagValue::Ident(id)).pat()),
        (Succeed(GetLiteral.pat()).pat(), GetLiteral.map(|lit, _| TagValue::Lit(lit)).pat()),
    ]), |v, _|v));
    let full_tag = Pattern::named("tag", (
        ExpectParticle('#'),
        ExpectParticle('['),
        tag.clone(),
        ExpectParticle(']')
    ), |(_, _, flag, _), _| flag);
    let tags = Pattern::named("tags",
                                    While(ExpectParticle('#').pat(), full_tag.clone()),
                                    |tags, _| tags.into_iter().map(|tag| (tag
                                                                              .0.0.clone(), tag)).collect::<HashMap<String, Tag>>());
    let operator = Pattern::named("operator", (
        GetParticle, Optional(Both(GetGluedParticle.pat(), Fail(ExpectParticleExact('=', true).pat()).pat()).pat(), GetGluedParticle.pat())
    ).map_res(|((p1, _), p2), loc| Ok(match p2.map(|c| p1.to_string() + &c.to_string()).unwrap_or(p1.to_string()).as_str() {
        "+" => Op::Add,
        "-" => Op::Sub,
        "*" => Op::Mul,
        "/" => Op::Div,
        "&" => Op::And,
        "|" => Op::Or,
        "&&" => Op::BinAnd,
        "||" => Op::BinOr,
        "<<" => Op::LShift,
        ">>" => Op::RShift,
        "<" => Op::LT,
        "<=" => Op::LE,
        ">" => Op::GT,
        ">=" => Op::GE,
        "==" => Op::EQ,
        "!=" => Op::NE,
        invalid => return Err(LithiaET::ParsingError(format!("invalid op {invalid}")).at(loc))
    })), |op, loc| Operator(op, loc));
    let (expression, expression_finalizer) = Latent::new();
    let let_create = Pattern::named("variable creation", (
        ExpectIdent("let".to_string()),
        Optional(ExpectIdent("mut".to_string()).pat(), ExpectIdent("mut".to_string()).pat()),
        GetIdent,
        Optional(ExpectParticle(':').pat(), (ExpectParticle(':'), type_pat.clone()).map(|(_, t), _| t).pat()),
        ExpectParticle('='),
        expression.clone()
    ), |(_, mutable, name, opt_ty, _, expr), _|
        Expr::VarCreate(name, mutable.is_some(), opt_ty, Box::new(expr)));
    let function_call = Pattern::named("function call", (
        item.clone(),
        ExpectParticle('('),
        Optional(expression.clone(), expression.clone()),
        While(
            Fail(ExpectParticle(')').pat()).pat(),
            (ExpectParticle(','), expression.clone()).map(|(_, expr), _|expr).pat()
        ),
        ExpectParticle(')'),
    ), |(item, _, arg0, mut args, _), _| {
        arg0.map(|arg0| args.insert(0, arg0));
        Expr::FuncCall(item, args)
    });
    let statement = Pattern::named("statement", (
            expression.clone(),
            IsOk(ExpectParticle(';').pat())
        ), |(expr, terminated), loc| Statement(expr, terminated, loc));
    let block_content = Pattern::named("block",
        While(
            Fail(ExpectParticle('}').pat()).pat(),
            statement.clone()
        ), |stmts, loc| Block(stmts, loc));
    let block = Pattern::inline(    (
            ExpectParticle('{'), block_content.clone(), ExpectParticle('}')
        ), |(_, block, _), _| block);
    let var_assign = Pattern::named("variable assign", (
        GetIdent,
        Or(operator.clone(),
           (operator.clone(), ExpectParticleExact('=', true)).map(|(op, _), _| Some(op)).pat(),
           ExpectParticle('=').map(|_, _| None).pat()
        ),
        expression.clone()
    ), |(item, op, expr), loc| Expr::VarAssign(item, op, Box::new(expr)));
    let if_expr = Pattern::named("if", (
            ExpectIdent("if".to_string()).pat(),
            expression.clone(),
            block.clone(),
            Optional(ExpectIdent("else".to_string()).pat(), (
                ExpectIdent("else".to_string()).pat(),
                block.clone(),
            ).map(|(_, opt_b), _| opt_b).pat())
        ),|(_, cond, body, opt_else), loc|
        Expr::If(Box::new(cond), body, opt_else.unwrap_or_else(||Block(vec![], loc))));
    expression_finalizer.finalize(Pattern::named("expression",(
        tags.clone(),
        Match(vec![
            (ExpectIdent("return".to_string()).pat(), (ExpectIdent("return".to_string()), Optional(expression.clone(), expression.clone()))
                .map(|(_, expr), _|{
                Expr::Return(expr.map(|e| Box::new(e)))
            }).pat()),
            (ExpectIdent("if".to_string()).pat(), if_expr.clone()),
            (ExpectParticle('{').pat(), block.clone().map(|block, _| Expr::Block(block)).pat()),
            (ExpectParticle('(').pat(), (ExpectParticle('('), expression.clone(), ExpectParticle(')')).map(|(_, expr, _), _| Expr::Expr(Box::new(expr))).pat()),
            (Succeed((GetIdent, Or(operator.clone(),
                                       Succeed((operator.clone(), ExpectParticleExact('=', true)).pat()).pat(),
                                       Succeed(ExpectParticle('=').pat()).pat()
            )).pat()).pat(), var_assign.clone()),
            (Succeed((item.clone(), ExpectParticle('(')).pat()).pat(), function_call.clone()),
            (ExpectIdent("let".to_string()).pat(), let_create.clone()),
            (ExpectParticle('&').pat(), (ExpectParticle('&'), expression.clone()).map(|(_, expr), _| Expr::Point(Box::new(expr))).pat()),
            (ExpectParticle('*').pat(), (ExpectParticle('*'), expression.clone()).map(|(_, expr), _| Expr::Deref(Box::new(expr))).pat()),
            (Succeed(GetIdent.pat()).pat(), GetIdent.map(|ident, loc| Expr::Variable(ident)).pat()),
            (Succeed(GetLiteral.pat()).pat(), GetLiteral.map(|lit, loc| Expr::Literal(lit)).pat())
        ]),
        While((tags.clone(), ExpectIdent("as".to_string())).pat(), (tags.clone(), ExpectIdent("as".to_string()), type_pat.clone())
            .map(|(tags, _, ty), loc|(loc, tags, ty)).pat()
        ),
        Optional(operator.clone(), (operator.clone(), expression.clone()).pat())
    ), |(tags, expr, casts, op), loc| {
        let mut ex = Expression(tags, expr, loc.clone());
        for (loc, tags, cast) in casts {
            ex = Expression(tags, Expr::Cast(Box::new(ex), cast), loc);
        }
        if let Some(op) = op {
            ex = Expression(HashMap::new(), Expr::BinaryOp(op.0, Box::new(ex), Box::new(op.1)), loc)
        }
        ex
    }));
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
                (Succeed(ExpectParticle('{').pat()).pat(), block.clone().map(|block, _| Some(block)).pat()),
                (Succeed(ExpectParticle(';').pat()).pat(), ExpectParticle(';').map(|_, _| None).pat())
            ])
    ), |(_, name, _, arg0, mut args, sig_end_loc, ret_ty, body), loc| {
        arg0.map(|arg0| args.insert(0, arg0));
        let mut signature_loc = name.1.clone();
        signature_loc.combine(sig_end_loc);
        Func {
            tags: HashMap::new(),
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
        (tags.clone(),
         Match(vec![
            (Succeed(ExpectIdent("fn".to_string()).pat()).pat(), function.clone().map(|f, _| ModuleContent::Function(f)).pat()),
            (Succeed(ExpectIdent("const".to_string()).pat()).pat(), constant.clone().map(|c, _| ModuleContent::Const(c)).pat())
        ])).pat()
        ).map_res(|content, _| {
            let mut functions = HashMap::new();
            let mut constants = HashMap::new();
            for (tags, c) in content.into_iter() {
                match c {
                    ModuleContent::Function(mut f) => {
                        f.tags = tags;
                        let l = f.name.1.clone();
                        if constants.contains_key(&f.name.0){
                            return Err(LithiaET::AlreadyDefinedError("constant".to_string(), f.name.0).ats(vec![l, f.name.1]))
                        }
                        if let Some(f) = functions.insert(f.name.0.clone(), f){
                            return Err(LithiaET::AlreadyDefinedError("function".to_string(), f.name.0).ats(vec![l, f.name.1]))
                        }
                    },
                    ModuleContent::Const(c) => {
                        if tags.len() > 0 {
                            return Err(LithiaET::TagError("tags not applicable for consts".to_string()).at(c.name.1.clone()))
                        }
                        let l = c.name.1.clone();
                        if functions.contains_key(&c.name.0){
                            return Err(LithiaET::AlreadyDefinedError("function".to_string(), c.name.0).ats(vec![l, c.name.1]))
                        }
                        if let Some(c) = constants.insert(c.name.0.clone(), c){
                            return Err(LithiaET::AlreadyDefinedError("constant".to_string(), c.name.0).ats(vec![l, c.name.1]))
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
