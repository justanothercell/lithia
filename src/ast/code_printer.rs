use crate::ast::{AstLiteral, Block, Const, Expr, Expression, Func, Ident, Item, Module, Op, Operator, Statement, Ty};
use crate::tokens::{Literal, NumLit};

pub(crate) trait CodePrinter{
    fn print(&self) -> String;
    fn print_indented(&self) -> String {
        String::from("    ") + &self.print().replace("\n", "\n    ")
    }
}

impl CodePrinter for Ty {
    fn print(&self) -> String {
        match self {
            Ty::Pointer(ty) => format!("*{}", ty.0.print()),
            Ty::Array(ty) => format!("[{}]", ty.0.print()),
            Ty::Single { generics, base_type, .. } =>
                if generics.len() == 0{
                    format!("{}", base_type.print())
                }
                else {
                    format!("{}<{}>", base_type.print(), generics.iter().map(|g|g.0.print()).collect::<Vec<_>>().join(", "))
                },
            Ty::Tuple(types) => format!("({})", types.iter().map(|t|t.0.print()).collect::<Vec<_>>().join(", ")),
            Ty::Signature(args, ret) => format!("fn({}) -> {}", args.iter().map(|t|t.0.print()).collect::<Vec<_>>().join(", "), ret.0.print()),
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
        }
    }
}

impl CodePrinter for AstLiteral {
    fn print(&self) -> String {
        self.0.print()
    }
}

impl CodePrinter for Expression {
    fn print(&self) -> String {
        match &self.0 {
            Expr::FuncCall(ident, args) => format!("{}({})", ident.print(), args.iter().map(|e|e.print()).collect::<Vec<_>>().join(", ")),
            Expr::Literal(lit) => lit.print(),
            Expr::Variable(var) => var.print(),
            Expr::UnaryOp(op, box expr) => format!("{}{}", op.print(), expr.print()),
            Expr::BinaryOp(op, box left, box right) => format!("({} {} {})", left.print(), op.print(), right.print()),
            Expr::VarCreate(ident, mutable, ty, expr) =>
            format!("let {}{}{} = {};",
                    if *mutable { "mut "} else {""}.to_string(),
                    ident.print(),
                    ty.as_ref().map(|t|format!(": {}", t.0.print())).unwrap_or("".to_string()),
                    expr.print()
            ),
            Expr::VarAssign(ident, Some(op), expr) => format!("{} {}= {};", ident.print(), op.print(), expr.print()),
            Expr::VarAssign(ident, None, expr) => format!("{} = {};", ident.print(), expr.print()),
            Expr::Block(block) => block.print()
        }
    }
}

impl CodePrinter for Operator {
    fn print(&self) -> String {
        match self.0 {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
            Op::And => "&&",
            Op::Or => "||",
            Op::Not => "!",
            Op::LShift => "<<",
            Op::RShift => ">>",
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
        format!("fn {}({}){}{}",
                self.name.print(),
                self.args.iter().map(|(ident, ty)| format!("{}: {}", ident.print(), ty.0.print())).collect::<Vec<_>>().join(", "),
                if self.ret.0.is_empty() {
                    String::new()
                } else {
                    format!(" -> {}", self.ret.0.print())
                },
                if let Some(body) = &self.body {
                    body.print()
                } else {String::from(";")}
        )
    }
}

impl CodePrinter for Const {
    fn print(&self) -> String {
        format!("const {}: {} = {};", self.name.print(), self.ty.0.print(), self.val.print())
    }
}

impl CodePrinter for Block {
    fn print(&self) -> String {
        if self.0.is_empty() {
            String::from(" {}")
        } else {
            format!(" {{\n{}\n}}", self.0.iter().map(|t| t.print_indented()).collect::<Vec<_>>().join("\n"))
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
        format!("{}\n{}",
                self.constants.values().map(|c| c.print()).collect::<Vec<_>>().join("\n"),
                self.functions.values().map(|t| t.print()).collect::<Vec<_>>().join("\n"))
    }
}