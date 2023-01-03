use crate::ast::{Item};
use crate::ast::patterns::{Consumer, Pat, Pattern};
use crate::ast::patterns::conditional::{While};
use crate::ast::patterns::dynamic::Mapping;
use crate::ast::patterns::simple::{ExpectParticle, ExpectParticleExact, GetIdent};

pub(crate) struct Patterns{
    pub(crate) item: Pat<Item>
}

pub(crate) fn build_patterns() -> Patterns {
    let item_pattern = Pattern::named("ident",
              (
                  GetIdent,
                  While(
                      (ExpectParticle(':'), ExpectParticleExact(':', true)).pat(),
                      (ExpectParticle(':'), ExpectParticleExact(':', true), GetIdent).map(|(_, _, i), _| i).pat()
                  ),
              ),
        |(ident, mut vec), loc| {vec.insert(0, ident); Item(vec, loc)});
    Patterns {
        item: item_pattern
    }
}