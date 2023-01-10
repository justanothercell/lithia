use std::fmt::{Display, Formatter};
use crate::source::span::Span;
use crate::tokens::{Literal, NumLit};

#[derive(Debug)]
pub(crate) struct ParseError {
    et: ParseET,
    locs: Vec<Span>,
    context: Vec<String>
}

impl ParseError {
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

impl From<std::io::Error> for ParseError {
    fn from(error: std::io::Error) -> Self {
        ParseET::IOError(error).error().when("doing IO operation")
    }
}

#[derive(Debug)]
pub(crate) enum ParseET {
    EOF,
    EmptyInput,
    IOError(std::io::Error),
    TokenizationError(String),
    LiteralError(Literal, String),
    ParsingError(String),
    CompilationError(String),
    AlreadyDefinedError(String, String),
    VariableNotFound(String),
    TypeError(String, String),
    TagError(String),
    UnsafeError(String)
}

impl ParseET {
    pub(crate) fn error(self) -> ParseError {
        ParseError {
            et: self,
            locs: vec![],
            context: vec![]
        }
    }
    pub(crate) fn at(self, loc: Span) -> ParseError {
        ParseError {
            et: self,
            locs: vec![loc],
            context: vec![]
        }
    }
    pub(crate) fn ats(self, locs: Vec<Span>) -> ParseError {
        ParseError {
            et: self,
            locs,
            context: vec![]
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}",
           match &self.et {
               ParseET::EOF => format!("Input Error:\n    reached end of file"),
               ParseET::EmptyInput => format!("Input Error:\n    input was empty"),
               ParseET::IOError(e) => format!("IO Error:\n    {}", e),
               ParseET::TokenizationError(e) => format!("Tokenization Error:\n    {}", e),
               ParseET::LiteralError(lit, e) => format!("{} literal Error:\n    {}", match lit {
                   Literal::String(_) => "String",
                   Literal::Char(_) => "Char",
                   Literal::Number(NumLit::Integer(_), _) => "Integer",
                   Literal::Number(NumLit::Float(_), _) => "Float",
                   Literal::Bool(_) => "Float",
                   Literal::Array(..) => "Array"
               }, e),
               ParseET::ParsingError(e) => format!("Parsing Error:\n    {}", e),
               ParseET::CompilationError(e) => format!("Compilation Error:\n    {}", e),
               ParseET::AlreadyDefinedError(what, name) =>
                   format!("Multiple definitions Error:\n    {} {} was already defined",
                   what, name),
               ParseET::VariableNotFound(ident) => format!("Name Error:\n    could not find variable {ident}"),
               ParseET::TypeError(expected, found) => format!("Type Error:\n    expected {expected} found {found}"),
               ParseET::TagError(err) => format!("Compiler Flag Error:\n    {err}"),
               ParseET::UnsafeError(thing) => format!("Unsafe Context Error:\n    cannot use {thing} in safe context.\n    tag the expr or func with #[unsafe]"),
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

impl<T> OnParseErr for Result<T, ParseError> {
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