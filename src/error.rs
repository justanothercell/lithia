use std::fmt::{Display, Formatter};
use crate::ast::code_printer::CodePrinter;
use crate::ast::Type;
use crate::ast::types_impl::TySat;
use crate::source::span::Span;
use crate::tokens::{Literal, NumLit};

#[derive(Debug)]
pub(crate) struct LithiaError {
    et: LithiaET,
    locs: Vec<Span>,
    context: Vec<String>
}

impl LithiaError {
    pub(crate) fn when<T: Into<String>>(mut self, reason: T) -> Self{
        self.context.push(reason.into());
        self
    }
    pub(crate) fn at(mut self, loc: Span) -> Self{
        self.locs = vec![loc];
        self
    }
    pub(crate) fn at_add(mut self, loc: Span) -> Self{
        self.locs.push(loc);
        self
    }
    pub(crate) fn ats(mut self, locs: Vec<Span>) -> Self{
        self.locs = locs;
        self
    }
}

impl From<std::io::Error> for LithiaError {
    fn from(error: std::io::Error) -> Self {
        LithiaET::IOError(error).error().when("doing IO operation")
    }
}

#[derive(Debug)]
pub(crate) enum LithiaET {
    EOF,
    IOError(std::io::Error),
    TokenizationError(String),
    LiteralError(Literal, String),
    ParsingError(String),
    CompilationError(String),
    AlreadyDefinedError(String, String),
    VariableNotFound(String),
    TypeError(Type, Type),
    CastError(Type, Type),
    TagError(String),
    UnsafeError(String),
}

impl LithiaET {
    pub(crate) fn error(self) -> LithiaError {
        LithiaError {
            et: self,
            locs: vec![],
            context: vec![]
        }
    }
    pub(crate) fn at(self, loc: Span) -> LithiaError {
        LithiaError {
            et: self,
            locs: vec![loc],
            context: vec![]
        }
    }
    pub(crate) fn ats(self, locs: Vec<Span>) -> LithiaError {
        LithiaError {
            et: self,
            locs,
            context: vec![]
        }
    }
}

impl Display for LithiaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}",
           match &self.et {
               LithiaET::EOF => format!("Input Error:\n    reached end of file"),
               LithiaET::IOError(e) => format!("IO Error:\n    {}", e),
               LithiaET::TokenizationError(e) => format!("Tokenization Error:\n    {}", e),
               LithiaET::LiteralError(lit, e) => format!("{} literal Error:\n    {}", match lit {
                   Literal::String(_) => "String",
                   Literal::Char(_) => "Char",
                   Literal::Number(NumLit::Integer(_), _) => "Integer",
                   Literal::Number(NumLit::Float(_), _) => "Float",
                   Literal::Bool(_) => "Float",
                   Literal::Array(..) => "Array"
               }, e),
               LithiaET::ParsingError(e) => format!("Parsing Error:\n    {}", e),
               LithiaET::CompilationError(e) => format!("Compilation Error:\n    {}", e),
               LithiaET::AlreadyDefinedError(what, name) =>
                   format!("Multiple definitions Error:\n    {} {} was already defined",
                   what, name),
               LithiaET::VariableNotFound(ident) => format!("Name Error:\n    could not find variable {ident}"),
               LithiaET::TypeError(expected, found) => format!("Type Error:\n    expected {} found {}:", expected.print(), found.print()),
               LithiaET::CastError(expected, found) => format!("Cast Error:\n    cannot cast from {} to {}:", expected.print(), found.print()),
               LithiaET::TagError(err) => format!("Compiler Flag Error:\n    {err}"),
               LithiaET::UnsafeError(thing) => format!("Unsafe Context Error:\n    cannot use {thing} in safe context.\n    tag the expr or func with #[unsafe]"),
           },
           if self.context.len() > 0 {
               format!("\n    while {}", self.context.join("\n    while "))
           } else {
               String::new()
           },
           {
               let mut locs = String::new();
               for loc in &self.locs {
                   locs.push_str(&format!("\n{:?}: {:?}\n{}",
                                          loc.source,
                                          loc,
                                          loc.render_span_code(2)
                   ))
               }
               locs
           }
        )
    }
}

pub(crate) trait OnParseErr{
    fn e_when<S: Into<String>>(self, reason: S) -> Self;
    fn e_at(self, loc: Span) -> Self;
    fn e_at_add(self, loc: Span) -> Self;
}

impl<T> OnParseErr for Result<T, LithiaError> {
    fn e_when<S: Into<String>>(self, reason: S) -> Self {
        self.map_err(|err| err.when(reason))
    }

    fn e_at(self, loc: Span) -> Self {
        self.map_err(|err| err.at(loc))
    }
    fn e_at_add(self, loc: Span) -> Self {
        self.map_err(|err| err.at_add(loc))
    }
}