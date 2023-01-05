use std::collections::HashMap;
use std::hash::Hash;
use crate::ast::{Block, Expr, Expression, FullType, Func, Item, Statement, TypeT};
use crate::ast::patterns::{Consumer, Pat, Pattern};
use crate::ast::patterns::conditional::{While, Match, Succeed, Fail, IsOk};
use crate::ast::patterns::dynamic::{Latent, Mapping};
use crate::ast::patterns::simple::{ExpectIdent, ExpectParticle, ExpectParticleExact, GetIdent, GetLiteral, GetNext};
use crate::error::{ParseError, ParseET};
use crate::source::span::Span;

pub(crate) struct Patterns{
    pub(crate) item: Pat<Item>,
    pub(crate) module_content: Pat<((HashMap<String, Func>,), Span)>
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
    let (expression, expression_finalizer) = Latent::new();
    let function_call = Pattern::named("function call", (
        item.clone(),
        ExpectParticle('('),
        While(
            Fail(ExpectParticle(')').pat()).pat(),
            expression.clone()
        ),
        ExpectParticle(')'),
    ), |(item, _, args, _), loc| Expr::FuncCall(item, args));
    expression_finalizer.finalize(Pattern::named("expression",
            Match(vec![
                (Succeed((item.clone(), ExpectParticle('(')).pat()).pat(), function_call.clone()),
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
            ExpectParticle(')'),
            Match(vec![
                (Succeed(ExpectParticle('{').pat()).pat(), (ExpectParticle('{'), block.clone(), ExpectParticle('}')).map(|(_, block, _), _| Some(block)).pat()),
                (Succeed(ExpectParticle(';').pat()).pat(), ().map(|_, _| None).pat())
            ])
    ), |(_, name, _, _, body), loc| Func {
        name,
        args: vec![],
        ret: FullType(TypeT::Tuple(vec![]), loc.clone()),
        body,
        loc,
    });
    enum ModuleContent{
        Function(Func)
    }
    let module_content = Pattern::named("module content",
        While(
            GetNext.pat(),
            Match(vec![
                (Succeed(ExpectIdent("fn".to_string()).pat()).pat(), function.clone().map(|f, _| ModuleContent::Function(f)).pat())
            ]).pat()
        ).map_res(|content, loc| {
            let mut functions = HashMap::new();
            for c in content.into_iter() {
                match c {
                    ModuleContent::Function(f) => {
                        let l = f.name.1.clone();
                        if let Some(f) = functions.insert(f.name.0.clone(), f){
                            return Err(ParseET::AlreadyDefinedError("function".to_string(), f.name.0, f.name.1).at(l))
                        }
                    }
                };
            }
            Ok((functions,))
        }), |content, loc| (content, loc));
    Patterns {
        item,
        module_content
    }
}