use std::fmt;

pub struct Equation<'a> {
    lhs: Box<Expr<'a>>,
    op: Opcode,
    rhs: Box<Expr<'a>>,
}

type EqTuple<'a> = (Box<Expr<'a>>,Opcode,Box<Expr<'a>>);
impl<'a> From<EqTuple<'a>> for Equation<'a> {
    fn from(v: EqTuple<'a>) -> Self {
        Self { lhs: v.0, op: v.1, rhs: v.2 }
    }
}

pub enum ExprKind<'a> {
    Name(&'a str),
    Op(Box<Expr<'a>>, Opcode, Box<Expr<'a>>),
}

pub struct Expr<'a> {
    pub v: ExprKind<'a>,
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

impl<'a> fmt::Display for Expr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::ExprKind::*;
        match &self.v {
            Name(x) => write!(f, "{}",
                    if self.l > 0 { raise_expr(x) }
                    else { (*x).to_string() }
                ),
            Op(x,y,z) => {
                let res = format!("{} {} {}", x, y, z);
                write!(f, "{}",
                    if self.l > 0 { raise_expr(&res) }
                    else { res }
                )
            },
        }
    }
}

impl<'a> fmt::Display for Equation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", &self.lhs, &self.op, &self.rhs)
    }
}

fn raise_expr(s: &str) -> String {
    format!("{{ {} }}", s)
}
