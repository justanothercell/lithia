use crate::ast::{Block, Expr, Expression, FullType, Func, Ident, Item, Statement, TypeT};
use crate::ast::patterns::{Consumer, Pat, Pattern};
use crate::ast::patterns::conditional::{While, Match, Succeed, Fail, IsOk};
use crate::ast::patterns::dynamic::{Latent, Mapping};
use crate::ast::patterns::simple::{ExpectIdent, ExpectParticle, ExpectParticleExact, GetIdent, GetLiteral, GetNext};
use crate::source::span::Span;

pub(crate) struct Patterns{
    pub(crate) item: Pat<Item>,
    pub(crate) module_content: Pat<Vec<Func>>
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
        ), |content, loc| {
            let mut functions = vec![];
            for c in content.into_iter() {
                match c {
                    ModuleContent::Function(f) => functions.push(f)
                }
            }
            functions
        });
    Patterns {
        item,
        module_content
    }
}