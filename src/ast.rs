use std::fmt;
use std::vec::Vec;
use std::ops;

pub enum Target {
    ExprSet(ExprSet),
    Expr(Box<Expr>),
}

pub struct ExprSet(pub Vec<Box<Expr>>);
impl ops::Deref for ExprSet {
    type Target = Vec<Box<Expr>>;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl ops::DerefMut for ExprSet {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

pub enum ExprKind {
    Ident(String),
    UnaryOp(Opcode, Box<Expr>),
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
}

pub struct Expr {
    pub v: ExprKind,
    pub prio: usize,
}

impl Expr {
    pub fn new(v: ExprKind, prio: usize) -> Self {
        Expr { v, prio }
    }
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
    AboutEquals,
    NotEquals,
    GreaterThan,
    LesserThan,
    GtEquals,
    LtEquals,
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Target::*;
        match &self {
            ExprSet(set) => write!(f, "{}", set),
            Expr(e) => write!(f, "{}", e),
        }
    }
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
            AboutEquals => "~=",
            NotEquals => "!=",
            GreaterThan => ">",
            LesserThan => "<",
            GtEquals => ">=",
            LtEquals => "<=",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for ExprKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ExprKind::*;
        match &self {
            Ident(x) => write!(f, "{}", x),
            UnaryOp(o,v) => write!(f, "{}{}", o,v),
            BinaryOp(l,o,r) => write!(f, "{} {} {}", l,o,r),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let inner = self.v.to_string();
        match self.prio {
            0 => write!(f, "{}", inner),
            1 => write!(f, "{{ {} }}", inner),
            _ => write!(f, "{{ ( {} ) }}", inner),
        }
    }
}

impl fmt::Display for ExprSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter().fold(Ok(()), |res, e| {
            res.and_then(|_| write!(f, "{}; ~~~ ", e))
        })
    }
}

