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
    pub known: HashMap<String, f64>,
    pub repstyle: RepStyle,
    pub autocalc_ident: String,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
            repstyle: RepStyle::SiSuffix,
            autocalc_ident: "?".to_string(),
        }
    }
    pub fn eval(&self, e: &Expr) -> Option<f64> {
        use { ExprKind::*, Opcode::* };
        match &e.v {
            Constant(v) => Some(*v),
            Ident(name) => {
                self.known.get(name).map(|x| {
                    eprintln!("load  {} = {}", name, x);
                    *x
                })
            },
            // TODO, FIXME: Add the basic trig functions to a hardcoded hashmap for now
            Function(_name, _arg) => None,
            UnaryOp(op, e) => {
                match op {
                    Add => self.eval(e).map(|a| { a.abs() }),
                    Sub => self.eval(e).map(|a| { -a.abs() }),
                    _ => None,
                }
            },
            ExprKind::BinaryOp(lhs, op, rhs) => {
                match op {
                    At | NotEquals | GreaterThan | LesserThan | GtEquals | LtEquals => None,
                    Equals | ApproxEquals => self.eval(rhs).or_else(|| { self.eval(lhs) }),
                    _ => {
                        self.eval(rhs).map(|b| {
                            self.eval(lhs).map(|a| {
                                match op {
                                    Add => a + b,
                                    Sub => a - b,
                                    Mul => a * b,
                                    Div => a / b,
                                    Pow => a.powf(b),
                                    _ => f64::NAN,
                                }
                            })
                        }).flatten()
                    }
                }
            }
        }
    }
    pub fn store_op(&mut self, e: &Expr) {
        use ExprKind::*;
        if let BinaryOp(l,_o,r) = &e.v {
            if let Some(val) = self.eval(e) {
                self.store(l, val);
                self.store(r, val);
            }
        } else {
            panic!("Tried storing {} as a BinaryOp!", e);
        }
    }
    pub fn store(&mut self, e: &Expr, val: f64) {
        if let ExprKind::Ident(name) = &e.v {
            self.known.insert(name.to_string(), val);
            eprintln!("store {} = {}", name, val);
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

#[derive(Clone)]
pub enum ExprKind {
    Constant(f64),
    Ident(String),
    Function(String, Box<Expr>),
    UnaryOp(Opcode, Box<Expr>),
    BinaryOp(Box<Expr>, Opcode, Box<Expr>),
}

#[derive(Clone)]
pub struct Expr {
    pub v: ExprKind,
    pub unit: Option<String>,
}

impl Expr {
    pub fn new(v: ExprKind) -> Self {
        Expr { v, unit: None }
    }
    pub fn with_unit(v: ExprKind, unit: String) -> Self {
        Expr { v, unit: Some(unit) }
    }

    pub fn process(mut self, scope: &mut Scope) -> Self {
        use { ExprKind::*, Opcode::* };
        match &mut self.v {
            BinaryOp(l, o, r) => {
                if let Ident(lid) = &l.v {
                    if lid.to_string() == scope.autocalc_ident.as_str() {
                        match scope.eval(r) {
                            Some(val) => l.v = Constant(val),
                            None => l.v = Ident("?".to_string()),
                        }
                    }
                }
                if let Ident(rid) = &r.v {
                    if rid.to_string() == scope.autocalc_ident.as_str() {
                        match scope.eval(l) {
                            Some(val) => r.v = Constant(val),
                            None => r.v = Ident("?".to_string()),
                        }
                    }
                }
                match o {
                    Equals | ApproxEquals => scope.store_op(&self),
                    _ => (),
                }
            },
            _ => (),
        }
        self
    }
}
impl From<f64> for Expr {
    fn from(val: f64) -> Self {
        Expr::new(ExprKind::Constant(val))
    }
}

// TODO: add more eqn ops
#[derive(Clone)]
pub enum Opcode {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
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
        use Opcode::*;
        let name: &'static str = match self {
            Add => "+",
            Sub => "-",
            Mul => "times",
            Div => "over",
            At => "@",
            Subscript => "sub",
            Superscript | Pow => "sup",
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
            Constant(v) => write!(f, "{}", v),
            Ident(name) => write!(f, "{}", name),
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
        write!(f, "{}", self.iter().map(|e| e.to_string()).collect::<Vec<String>>().join("; ~~~ "))
    }
}
