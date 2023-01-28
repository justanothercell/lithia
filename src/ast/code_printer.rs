use std::collections::HashMap;
use crate::ast::{AstLiteral, Block, Const, Expr, Expression, Func, Ident, Item, Module, Op, Operator, Statement, Tag, TagValue, Ty, Type};
use crate::tokens::{Literal, NumLit};

pub(crate) trait CodePrinter{
    fn print(&self) -> String;
    fn print_indented(&self) -> String {
        String::from("    ") + &self.print().replace("\n", "\n    ")
    }
}

impl CodePrinter for Type {
    fn print(&self) -> String {
        self.0.print()
    }
}

impl CodePrinter for Ty {
    fn print(&self) -> String {
        match self {
            Ty::Pointer(ty) => format!("&{}", ty.print()),
            Ty::RawPointer => "&".to_string(),
            Ty::Array(ty, c) => format!("[{};{c}]", ty.print()),
            Ty::Slice(ty) => format!("[{}]", ty.print()),
            Ty::Single(generics, base_type) =>
                if generics.len() == 0{
                    format!("{}", base_type.print())
                }
                else {
                    format!("{}<{}>", base_type.print(), generics.iter().map(|g|g.0.print()).collect::<Vec<_>>().join(", "))
                },
            Ty::Tuple(types) => format!("({})", types.iter().map(|t|t.0.print()).collect::<Vec<_>>().join(", ")),
            Ty::Signature(args, ret, unsafe_fn, vararg) => format!("{}fn({}{}) -> {}",
                                                                   if *unsafe_fn { "unsafe ".to_string() } else { String::new() },
                                                                   args.iter().map(|t|t.0.print()).collect::<Vec<_>>().join(", "),
                                                                   if *vararg { if args.len() > 0 { ", ...".to_string() } else { "...".to_string() } } else { String::new() },

                                                                   ret.0.print()),
        }
    }
}

impl CodePrinter for Ident {
    fn print(&self) -> String {
        self.0.clone()
    }
}

impl CodePrinter for Item {
    fn print(&self) -> String {
        self.0.iter().map(|i|i.print()).collect::<Vec<_>>().join("::")
    }
}

impl CodePrinter for Literal {
    fn print(&self) -> String {
        match self {
            Literal::String(s) => format!("{s:?}"),
            Literal::Char(c) => format!("{c:?}"),
            Literal::Number(NumLit::Integer(i), ty) => format!("{i}{}", ty.as_ref().map_or(String::new(), |t| format!("{t}"))),
            Literal::Number(NumLit::Float(f), ty) => format!("{f}{}", ty.as_ref().map_or(String::new(), |t| format!("{t}"))),
            Literal::Bool(b) => format!("{b}"),
            Literal::Array(v, _ty, _) => format!("[{}]", v.iter().map(|v|v.print()).collect::<Vec<_>>().join(", ")),
        }
    }
}

impl CodePrinter for AstLiteral {
    fn print(&self) -> String {
        self.0.print()
    }
}

impl CodePrinter for HashMap<String, Tag> {
    fn print(&self) -> String {
        self.iter().map(|(_name, tag)| format!("#[{}]", tag.print())).collect::<Vec<_>>().join("\n")
    }
}

impl CodePrinter for Tag {
    fn print(&self) -> String {
        format!("{}{}", self.0.0, if self.1.len() > 0 {
            format!("({})", self.1.iter().map(|v| v.print()).collect::<Vec<_>>().join(", "))
        } else { String::new() })
    }
}

impl CodePrinter for TagValue {
    fn print(&self) -> String {
        match self {
            TagValue::Lit(lit) => lit.print(),
            TagValue::Ident(ident) => ident.print(),
            TagValue::Tag(tag) => tag.print()
        }
    }
}

impl CodePrinter for Expression {
    fn print(&self) -> String {
        format!("{}{}", if self.0.len() > 0 { format!("{}\n", self.0.print()) } else { String::new() },
                match &self.1 {
            Expr::FuncCall(ident, args) => format!("{}({})", ident.print(), args.iter().map(|e|e.print()).collect::<Vec<_>>().join(", ")),
            Expr::Expr(expr) => format!("({})", expr.print()),
            Expr::Point(expr) => format!("&{}", expr.print()),
            Expr::Deref(expr) => format!("*{}", expr.print()),
            Expr::Cast(expr, ty) => format!("{} as {}", expr.print(), ty.print()),
            Expr::Literal(lit) => lit.print(),
            Expr::Variable(var) => var.print(),
            Expr::UnaryOp(op, box expr) => format!("{}{}", op.print(), expr.print()),
            Expr::BinaryOp(op, box left, box right) => format!("({} {} {})", left.print(), op.print(), right.print()),
            Expr::VarCreate(ident, mutable, ty, expr) =>
            format!("let {}{}{} = {}",
                    if *mutable { "mut "} else {""}.to_string(),
                    ident.print(),
                    ty.as_ref().map(|t|format!(": {}", t.0.print())).unwrap_or("".to_string()),
                    expr.print()
            ),
            Expr::VarAssign(ident, Some(op), expr) => format!("{} {}= {}", ident.print(), op.print(), expr.print()),
            Expr::VarAssign(ident, None, expr) => format!("{} = {};", ident.print(), expr.print()),
            Expr::Block(block) => block.print_indented(),
            Expr::If(cond, body, else_body) => format!("if {} {} else {}", cond.print(), body.print(), else_body.print()),
            Expr::Return(expr) => match expr { Some(e) => format!("return {}", e.print()), None => format!("return") }
        })
    }
}

impl CodePrinter for Operator {
    fn print(&self) -> String {
        match self.0 {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::And => "&",
            Op::Or => "|",
            Op::Not => "!",
            Op::BinAnd => "&&",
            Op::BinOr => "||",
            Op::LShift => "<<",
            Op::RShift => ">>",
            Op::LT => "<",
            Op::LE => "<=",
            Op::GT => ">",
            Op::GE => ">=",
            Op::EQ => "==",
            Op::NE => "!=",
        }.to_string()
    }
}

impl CodePrinter for Statement {
    fn print(&self) -> String {
        format!("{}{}", self.0.print(), if self.1 {";"} else {""} )
    }
}

impl CodePrinter for Func {
    fn print(&self) -> String {
        format!("{}fn {}({}){}{}",
            if self.tags.len() > 0 { format!("{}\n", self.tags.print()) } else { String::new() },
            self.name.print(),
            self.args.iter().map(|(ident, ty)| format!("{}: {}", ident.print(), ty.print())).collect::<Vec<_>>().join(", "),
            if self.ret.0.is_empty() {
                String::new()
            } else {
                format!(" -> {}", self.ret.0.print())
            },
            if let Some(body) = &self.body {
                format!(" {}", body.print())
            } else {String::from(";")}
        )
    }
}

impl CodePrinter for Const {
    fn print(&self) -> String {
        format!("const {}: {} = {};", self.name.print(), self.ty.print(), self.val.print())
    }
}

impl CodePrinter for Block {
    fn print(&self) -> String {
        if self.0.is_empty() {
            String::from("{}")
        } else {
            format!("{{\n{}\n}}", self.0.iter().map(|t| t.print_indented()).collect::<Vec<_>>().join("\n"))
        }
    }
}

impl CodePrinter for Module {
    fn print(&self) -> String {
        format!("mod {} {{\n    {}\n}}", self.name.print(), self.print_content().replace("\n", "\n    "))
    }
}

impl Module {
    fn print_content(&self) -> String {
        format!("{}\n\n{}",
                self.constants.values().map(|c| c.print()).collect::<Vec<_>>().join("\n\n"),
                self.functions.values().map(|t| t.print()).collect::<Vec<_>>().join("\n\n"))
    }
}