use std::fmt;

pub enum ExprKind {
    Ident(String),
    UnaryOp(Opcode, Box<Expr>),
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
}

pub struct Expr {
    pub v: ExprKind,
    pub l: usize,
}

// TODO: add more eqn ops
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Subscript,
    Superscript,
    Equals,
    NotEquals,
    GreaterThan,
    LesserThan,
    GtEquals,
    LtEquals,
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::Opcode::*;
        let name: &'static str = match self {
            Add => "+",
            Sub => "-",
            Mul => "times",
            Div => "over",
            Subscript => "sub",
            Superscript => "sup",
            Equals => "=",
            NotEquals => "!=",
            GreaterThan => ">",
            LesserThan => "<",
            GtEquals => ">=",
            LtEquals => "<=",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ExprKind::*;
        match &self.v {
            Ident(x) => write!(f, "{}",
                    if self.l > 0 { raise_expr(x) }
                    else { (*x).to_string() }
                ),
            UnaryOp(o,r) => {
                let res = format!("{}{}", o, r);
                write!(f, "{}",
                    if self.l > 0 { raise_expr(&res) }
                    else { res }
                )
            },
            BinaryOp(l,o,r) => {
                let res = format!("{} {} {}", l, o, r);
                write!(f, "{}",
                    if self.l > 0 { raise_expr(&res) }
                    else { res }
                )
            },
        }
    }
}

fn raise_expr(s: &str) -> String {
    format!("{{ {} }}", s)
}
