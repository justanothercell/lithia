use std::fmt::{Display, Formatter};
use crate::source::span::Span;

#[derive(Debug)]
pub(crate) struct ParseError {
    et: ParseET,
    loc: Option<Span>,
    context: Vec<String>
}

impl ParseError {
    pub(crate) fn when<T: Into<String>>(mut self, reason: T) -> Self{
        self.context.push(reason.into());
        self
    }
    pub(crate) fn at(mut self, loc: Span) -> Self{
        self.loc = Some(loc);
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
    ParseLiteralError(Literal, String),
    ParsingError(String)
}

impl ParseET {
    pub(crate) fn error(self) -> ParseError {
        ParseError {
            et: self,
            loc: None,
            context: vec![]
        }
    }
    pub(crate) fn at(self, loc: Span) -> ParseError {
        ParseError {
            et: self,
            loc: Some(loc),
            context: vec![]
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}",
               match &self.et {
                   ParseET::EOF => format!("Input error:\n    reached end of file"),
                   ParseET::EmptyInput => format!("Input error:\n    input was empty"),
                   ParseET::IOError(e) => format!("IO error:\n    {}", e),
                   ParseET::TokenizationError(e) => format!("Tokenization error:\n    {}", e),
                   ParseET::ParseLiteralError(lit, e) => format!("{} literal parsing error:\n    {}", match lit {
                       Literal::String(_) => "String",
                       Literal::Char(_) => "Char",
                       Literal::Number(NumLit::Integer(_), _) => "Integer",
                       Literal::Number(NumLit::Float(_), _) => "Float",
                       Literal::Bool(_) => "Float",
                   }, e),
                   ParseET::ParsingError(e) => format!("Parsing error:\n    {}", e)
               },
               if self.context.len() > 0 {
                   format!("\n    while {}", self.context.join("\n    while "))
               } else {
                   String::new()
               },
               if let Some(loc) = &self.loc {
                   format!("{}\n{}",
                           if loc.start == loc.end {
                               let (l, p) = loc.start().pos();
                               format!("\n\nat: {}: {}:{}", loc.source.st, l, p)
                           } else {
                               let (sl, sp) = loc.start().pos();
                               let (el, ep) = loc.end().pos();
                               format!("\n\nat: {}: {}:{}..{}:{}", loc.source.st, sl, sp, el, ep)
                           },
                           loc.render_span_code(2)
                   )
               } else {
                   String::new()
               },
        )
    }
}

pub(crate) trait OnParseErr{
    fn e_when<T: Into<String>>(self, reason: String) -> Self;
    fn e_at(self, loc: Span) -> Self;
}

impl<T> OnParseErr for Result<T, ParseError> {
    fn e_when<T: Into<String>>(self, reason: T) -> Self {
        self.map_err(|err| err.when(reason))
    }

    fn e_at(self, loc: Span) -> Self {
        self.map_err(|err| err.at(loc))
    }
}