use crate::ast::Ident;
use crate::ast::patterns::{Pat, Pattern};
use crate::ast::patterns::simple::{ExpectParticle, ExpectParticleExact, GetIdent};

pub(crate) struct Patterns{
    pub(crate) item: Pat<Ident>
}

pub(crate) fn build_patterns() -> Patterns {
    let item_pattern = Pattern::named("ident",
              (
                  GetIdent,
                  ExpectParticle(':'),
                  ExpectParticleExact(':', true)
              ),
        |(mut i, _, _), loc| {i.1.combine(loc); i});
    Patterns {
        item: item_pattern
    }
}