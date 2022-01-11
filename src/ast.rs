use std::collections::hash_map::HashMap;
use std::fmt;
use std::ops;
use std::vec::Vec;

pub enum RepStyle {
    SiSuffix,
    TenExp,
    Scientific,
}

pub struct Scope {
    known: HashMap<String, f64>,
    repstyle: RepStyle,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
            repstyle: RepStyle::SiSuffix,
        }
    }
}

pub enum Target {
    ExprSet(ExprSet),
    Expr(Box<Expr>),
}

pub struct ExprSet(pub Vec<Box<Expr>>);
impl ops::Deref for ExprSet {
    type Target = Vec<Box<Expr>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ops::DerefMut for ExprSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub enum ExprKind {
    Value(Box<Value>),
    Function(String, Box<Expr>),
    UnaryOp(Opcode, Box<Expr>),
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
}

pub struct Expr {
    pub v: ExprKind,
    pub unit: Option<String>,
}

impl Expr {
    pub fn new(v: ExprKind, unit: Option<String>) -> Self {
        Expr { v, unit }
    }
    pub fn eval(&self) -> Option<f64> {
        match &self.v {
            ExprKind::Value(v) => v.num_val,
            // TODO, FIXME: Add the basic trig functions to a hardcoded hashmap for now
            ExprKind::Function(_name, _arg) => None,
            ExprKind::UnaryOp(op, e) => {
                match op {
                    Opcode::Add => e.eval().map(|v| { v.abs() }),
                    Opcode::Sub => e.eval().map(|v| { -v.abs() }),
                    _ => None,
                }
            },
            ExprKind::BinaryOp(lhs, op, rhs) => {
                match op {
                    Opcode::At => None,
                    Opcode::Subscript => None,
                    Opcode::Superscript => None,
                    Opcode::Equals => lhs.eval().or_else(|| { rhs.eval() }),
                    Opcode::ApproxEquals => lhs.eval().or_else(|| { rhs.eval() }),
                    Opcode::NotEquals => None,
                    Opcode::GreaterThan => None,
                    Opcode::LesserThan => None,
                    Opcode::GtEquals => None,
                    Opcode::LtEquals => None,
                    _ => lhs.eval().map(|a| {
                        rhs.eval().map(|b| {
                            match op {
                                Opcode::Add => a + b,
                                Opcode::Sub => a - b,
                                Opcode::Mul => a * b,
                                Opcode::Div => a / b,
                                _ => f64::NAN,
                            }
                        })
                    }).flatten()
                }
            }
        }
    }
}
impl From<f64> for Expr {
    fn from(val: f64) -> Self {
        Expr::new(ExprKind::Value(Box::new(Value::new(val.to_string(), Some(val)))), None)
    }
}

pub struct Value {
    pub text: String,
    pub num_val: Option<f64>,
}

impl Value {
    pub fn new(text: String, num_val: Option<f64>) -> Self {
        Value { text, num_val }
    }
}

// TODO: add more eqn ops
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    At,
    Subscript,
    Superscript,
    Equals,
    ApproxEquals,
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
            At => "@",
            Subscript => "sub",
            Superscript => "sup",
            Equals => "=",
            ApproxEquals => "~approx~",
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
            Value(v) => write!(f, "{}", v),
            Function(n, a) => write!(f, "{} ( {} )", n, a),
            UnaryOp(o, v) => write!(f, "{}{{ {} }}", o, v),
            BinaryOp(l, o, r) => write!(f, "{{ {} }} {} {{ {} }}", l, o, r),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(unit) = &self.unit {
            write!(f, "{} {{ {} }}", self.v, unit)
        } else {
            write!(f, "{}", self.v)
        }
    }
}

impl fmt::Display for ExprSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.iter()
            .fold(Ok(()), |res, e| res.and_then(|_| write!(f, "{}; ~~~ ", e)))
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
