use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Peekable;
use std::str::{Chars, FromStr};
use std::str::pattern::Pattern;
use crate::compiler::compiler::{Loc, ParseError};
use crate::variable::Value;

pub(crate) struct Tokens(Vec<Token>);

impl Tokens {
    pub(crate) fn get_tokens(self) -> Vec<Token> {
        self.0
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Token {
    EndStmt(Loc),
    Ident(String, Loc),
    Bracket(Bracket, Loc),
    String(String, Loc),
    NumLiteral(String, String, Loc),
    Assign(Loc),
    TypeSep(Loc),
    PathSep(Loc),
    ArgSep(Loc),
    Op(Op, Loc),
    EOF(Loc)
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Op{
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    TrueMod,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Lshift,
    Rshift
}

impl Display for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::Mod => "%",
            Op::TrueMod => "%%",
            Op::Pow => "**",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
            Op::Eq => "==",
            Op::Ne => "==",
            Op::And => "&&",
            Op::Or => "||",
            Op::BitAnd => "&",
            Op::BitOr => "|",
            Op::BitXor => "^",
            Op::Lshift => "<<",
            Op::Rshift => ">>"
        })
    }
}

impl Op {
    pub(crate) fn group(&self) -> OpGroup {
        match self {
            Op::Add => OpGroup::Dash,
            Op::Sub => OpGroup::Dash,
            Op::Mul => OpGroup::Dot,
            Op::Div => OpGroup::Dot,
            Op::Mod => OpGroup::Dot,
            Op::TrueMod => OpGroup::Pow,
            Op::Pow => OpGroup::Pow,
            Op::Lt => OpGroup::Comp,
            Op::Le => OpGroup::Comp,
            Op::Gt => OpGroup::Comp,
            Op::Ge => OpGroup::Comp,
            Op::Eq => OpGroup::Comp,
            Op::Ne => OpGroup::Comp,
            Op::And => OpGroup::Bool,
            Op::Or => OpGroup::Bool,
            Op::BitAnd => OpGroup::Bit,
            Op::BitOr => OpGroup::Bit,
            Op::BitXor => OpGroup::Bit,
            Op::Lshift => OpGroup::Bit,
            Op::Rshift => OpGroup::Bit
        }
    }

    pub(crate) fn fn_op(&self) -> String{
        match self {
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::Div => "div",
            Op::Mod => "rem",
            Op::TrueMod => "rem_euclid",
            Op::Pow => "pow",
            Op::Lt => "lt",
            Op::Le => "le",
            Op::Gt => "gt",
            Op::Ge => "ge",
            Op::Eq => "eq",
            Op::Ne => "ne",
            Op::And => "and",
            Op::Or => "or",
            Op::BitAnd => "bitand",
            Op::BitOr => "bitor",
            Op::BitXor => "bitxor",
            Op::Lshift => "shl",
            Op::Rshift => "shr"
        }.to_string()
    }
}

#[derive(PartialEq)]
pub(crate) enum OpGroup {
    Pow,
    Dot,
    Dash,
    Bit,
    Comp,
    Bool
}

pub(crate) fn value_from_numer_literal(tok: Token) -> Result<Value, ParseError> {
    if let Token::NumLiteral(val, typ, loc) = tok {
        let r: Result<Value, Box<dyn Error>> = try {
            match typ.as_str() {
                "u8" => Value::U8(u8::from_str(&val)?),
                "u16" => Value::U16(u16::from_str(&val)?),
                "u32" => Value::U32(u32::from_str(&val)?),
                "u64" => Value::U64(u64::from_str(&val)?),
                "u128" => Value::U128(u128::from_str(&val)?),

                "i8" => Value::I8(i8::from_str(&val)?),
                "i16" => Value::I16(i16::from_str(&val)?),
                "i32" => Value::I32(i32::from_str(&val)?),
                "i64" => Value::I64(i64::from_str(&val)?),
                "i128" => Value::I128(i128::from_str(&val)?),

                "f32" => Value::F32(f32::from_str(&val)?),
                "f64" => Value::F64(f64::from_str(&val)?),
                other => Err(loc.error(format!("Provided type '{}' is not valid as a number type", other)))?
            }
        };
        match r {
            Ok(v) => Ok(v),
            Err(e) => Err(loc.error(format!("Error while parsing number literal: {}", e)))
        }
    }
    else {
        Err(tok.loc().error(format!("Unexpected token {:?} '{}', expected NumberLiteral", tok, tok)))
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Side{
    Open,
    Close
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Bracket{
    Curly(Side),
    Pointy(Side),
    Square(Side),
    Round(Side)
}

pub(crate) fn tokenize(code: &str) -> Result<Tokens, ParseError> {
    let mut input_iter = ParserIter::new(code);

    let mut tokens = Vec::<Token>::new();

    while let Some(&c) = input_iter.peek() {
        if c.is_whitespace() {
            input_iter.next();
        }
        else if c == ';' {
            tokens.push(Token::EndStmt(input_iter.here()));
            input_iter.next();
        }
        else if c == ',' {
            tokens.push(Token::ArgSep(input_iter.here()));
            input_iter.next();
        }
        else if c == ':' {
            input_iter.next();
            if let Some(token) = tokens.pop() {
                if let Token::TypeSep(loc) = token {
                    tokens.push(Token::PathSep(loc.clone()))
                }
                else{
                    tokens.push(token);
                    tokens.push(Token::TypeSep(input_iter.here()))
                }
            }
            else{
                tokens.push(Token::TypeSep(input_iter.here()))
            }
        }
        // IMPORTANT: operators is before brackets so that < and > land in operators!
        // their parsing is still included in brackets for redundancy. when parsing brackets, also check for op < >!
        // Also takes care of Assign "="
        else if c.is_contained_in("+-*/%=&|^><!") {
            let loc = input_iter.here();
            input_iter.next();
            tokens.push(Token::Op(match c {
                '+' => Op::Add,
                '-' => Op::Sub,
                '*' => {
                    if let Some('*') = input_iter.peek() {
                        input_iter.next();
                        Op::Pow
                    } else {
                        Op::Mul
                    }
                },
                '/' => Op::Div,
                '%' => {
                    if let Some('%') = input_iter.peek() {
                        input_iter.next();
                        Op::TrueMod
                    } else {
                        Op::Mod
                    }
                },
                '<' => {
                    if let Some('=') = input_iter.peek() {
                        input_iter.next();
                        Op::Le
                    } else if let Some('<') = input_iter.peek() {
                        input_iter.next();
                        Op::Lshift
                    } else {
                        Op::Lt
                    }
                },
                '>' => {
                    if let Some('=') = input_iter.peek() {
                        input_iter.next();
                        Op::Ge
                    } else if let Some('>') = input_iter.peek() {
                        input_iter.next();
                        Op::Rshift
                    } else {
                        Op::Gt
                    }
                },
                '=' => {
                    if let Some('=') = input_iter.peek() {
                        input_iter.next();
                        Op::Eq
                    } else {
                        tokens.push(Token::Assign(input_iter.here()));
                        continue
                    }
                },
                '!' => {
                    if let Some('=') = input_iter.peek() {
                        input_iter.next();
                        Op::Ne
                    } else {
                        unimplemented!()
                    }
                }
                '&' => {
                    if let Some('&') = input_iter.peek() {
                        input_iter.next();
                        Op::And
                    }
                    else {
                        Op::BitAnd
                    }
                },
                '|' => {
                    if let Some('|') = input_iter.peek() {
                        input_iter.next();
                        Op::Or
                    }
                    else {
                        Op::BitOr
                    }
                },
                '^' => Op::BitXor,
                _ => unreachable!()
            }, loc));
        }
        else if c.is_contained_in("[]{}<>()") {
            tokens.push(Token::Bracket(match c {
                '{' => Bracket::Curly(Side::Open),
                '}' => Bracket::Curly(Side::Close),
                '<' => Bracket::Pointy(Side::Open),
                '>' => Bracket::Pointy(Side::Close),
                '[' => Bracket::Square(Side::Open),
                ']' => Bracket::Square(Side::Close),
                '(' => Bracket::Round(Side::Open),
                ')' => Bracket::Round(Side::Close),
                _ => unreachable!()
            }, input_iter.here()));
            input_iter.next();
        }
        else if c == '"' {
            tokens.push(tokenize_string(&mut input_iter))
        }
        else if c == '/' {
            input_iter.next();
            match input_iter.peek() {
                Some('*') => {
                    input_iter.next();
                    skip_block_comment(&mut input_iter)
                },
                Some('/') => {
                    input_iter.next();
                    skip_line_comment(&mut input_iter)
                }
                _ => ()
            }
        }
        else if c.is_ascii_digit() {
            tokens.push(tokenize_number_literal(&mut input_iter)?)
        }
        else if c.is_alphabetic() || c == '_' {
            tokens.push(tokenize_ident(&mut input_iter))
        }
        else {
            return Err(input_iter.here().error(format!("Unexpected char '{}'", c)));
        }
    }
    tokens.push(Token::EOF(input_iter.here()));

    Ok(Tokens(tokens))
}

fn skip_line_comment(input_iter: &mut ParserIter) {
    while let Some(c) = input_iter.next() {
        if c == '\n' {
            break;
        }
    }
}

fn skip_block_comment(input_iter: &mut ParserIter) {
    while let Some(c) = input_iter.next() {
        if c == '*' {
            if let Some('/') = input_iter.peek() {
                input_iter.next();
                break;
            }
        }
    }
}

fn tokenize_string(input_iter: &mut ParserIter) -> Token{
    let loc = input_iter.here();
    let mut ident = String::new();
    input_iter.next();
    while let Some(&c) = input_iter.peek() {
        if c != '"' {
            ident.push(c);
            input_iter.next();
        }
        else{
            input_iter.next();
            break;
        }
    }
    return Token::String(ident, loc);
}

fn tokenize_ident(input_iter: &mut ParserIter) -> Token{
    let loc = input_iter.here();
    let mut ident = String::new();
    while let Some(&c) = input_iter.peek() {
        if c.is_alphanumeric() || c == '_' {
            ident.push(c);
            input_iter.next();
        }
        else{
            break;
        }
    }
    return Token::Ident(ident, loc);
}

fn tokenize_number_literal(input_iter: &mut ParserIter) -> Result<Token, ParseError>{
    let loc = input_iter.here();
    let mut val = String::new();
    let mut ty = String::new();
    let mut had_dot = false;
    while let Some(&c) = input_iter.peek() {
        if c == '_' {
            input_iter.next();
        }
        else if c.is_ascii_digit() {
            val.push(c);
            input_iter.next();
        }
        else if c == '.' {
            if !had_dot {
                val.push(c);
                had_dot = true;
                input_iter.next();
            }
            else{
                input_iter.next();
                return Err(input_iter.here().error("Encountered decimal point for a second time in this number".to_string()));
            }
        }
        else{
            break;
        }
    }
    let here = input_iter.here();
    if let Some(c) = input_iter.peek() {
        if !c.is_alphabetic() {
            return Err(here.error(format!("Expected token after number to be alphabetic: '{}'", c)));
        }
    }
    else {
        return Err(input_iter.here().error("Expected Some token after number".to_string()));
    }
    while let Some(&c) = input_iter.peek() {
        if c.is_alphanumeric() || c == '_' {
            ty.push(c);
            input_iter.next();
        }
        else{
            break;
        }
    }
    Ok(Token::NumLiteral(val, ty, loc))
}

#[derive(Clone)]
struct ParserIter<'a> {
    original: String,
    iter: Peekable<Chars<'a>>,
    index: usize,
    line: usize,
    index_in_line: usize
}

impl ParserIter<'_> {
    fn new(input: &str) -> ParserIter {
        ParserIter {
            original: input.clone().to_string(),
            iter: input.chars().peekable(),
            index: 0,
            line: 1,
            index_in_line: 0
        }
    }

    fn next(&mut self) -> Option<char>{
        let oc = self.iter.next();
        if let Some(c) = oc {
            self.index += 1;
            if c == '\n' {
                self.line += 1;
                self.index_in_line = 0;
            }
            else {
                self.index_in_line += 1;
            }
        }
        oc
    }

    fn peek(&mut self) -> Option<&char>{
        self.iter.peek()
    }

    fn here(&self) -> Loc {
        Loc {
            index: self.index,
            line: self.line,
            index_in_line: self.index_in_line,
            is_dummy: false,
            source_code: self.original.clone()
        }
    }
}

impl Token {
    pub(crate) fn loc(&self) -> &Loc {
        match self {
            Token::EndStmt(loc) => loc,
            Token::Ident(_, loc) => loc,
            Token::Bracket(_, loc) => loc,
            Token::String(_, loc) => loc,
            Token::NumLiteral(_, _, loc) => loc,
            Token::Assign(loc) => loc,
            Token::TypeSep(loc) => loc,
            Token::PathSep(loc) => loc,
            Token::ArgSep(loc) => loc,
            Token::Op(_, loc) => loc,
            Token::EOF(loc) => loc
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Token::EndStmt(_) => ";".to_string(),
            Token::Ident(ident, _) => ident.to_string(),
            Token::Bracket(Bracket::Curly(side), _) => format!("{}\n", Bracket::Curly(side.clone())),
            Token::Bracket(bracket, _) => format!("{}", bracket),
            Token::String(string, _) => format!("\"{}\"", string),
            Token::NumLiteral(num, n_type, _) => format!("{}{}", num, n_type),
            Token::Assign(_) => "=".to_string(),
            Token::TypeSep(_) => ":".to_string(),
            Token::PathSep(_) => "::".to_string(),
            Token::ArgSep(_) => ",".to_string(),
            Token::Op(op, _) => format!("{}", op),
            Token::EOF(_) => "".to_string()
        })
    }
}

impl Display for Tokens {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for t in self.0.iter() {
            s.push_str(&format!("{} ", t))
        }
        write!(f, "{}", s.replace(";", ";\n"))
    }
}


impl Display for Bracket {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Bracket::Curly(Side::Open) => "{",
            Bracket::Curly(Side::Close) => "}",
            Bracket::Pointy(Side::Open) => "<",
            Bracket::Pointy(Side::Close) => ">",
            Bracket::Square(Side::Open) => "[",
            Bracket::Square(Side::Close) => "]",
            Bracket::Round(Side::Open) => "(",
            Bracket::Round(Side::Close) => ")"
        })
    }
}
