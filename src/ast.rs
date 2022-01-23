use std::collections::HashMap;
use std::fmt;
use std::ops;
use std::vec::Vec;
use rug::{Float, ops::Pow};

#[derive(Clone, Copy)]
pub enum RepStyle {
    SiSuffix,
    TenExp,
    Scientific,
}

pub struct Scope {
    pub known: HashMap<String, Float>,
    pub repstyle: RepStyle,
    pub autocalc_ident: String,
    pub si_suff_lut: HashMap<i32,(&'static str, &'static str)>,
    pub precision: u32,
    pub max_digits_after_zero: usize,
}

pub trait Render {
    fn render(&self, scope: &Scope) -> String;
}

impl Scope {
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
            repstyle: RepStyle::SiSuffix,
            autocalc_ident: "?".to_string(),
            si_suff_lut: HashMap::from([
                                       ( 8, ("yotta", "Y")),
                                       ( 7, ("zetta", "Z")),
                                       ( 6, ("exa"  , "E")),
                                       ( 5, ("peta" , "P")),
                                       ( 4, ("tera" , "T")),
                                       ( 3, ("giga" , "G")),
                                       ( 2, ("mega" , "M")),
                                       ( 1, ("kilo" , "k")),
                                       (-1, ("milli", "m")),
                                       (-2, ("micro", "µ")),
                                       (-3, ("nano" , "n")),
                                       (-4, ("pico" , "p")),
                                       (-5, ("femto", "f")),
                                       (-6, ("atto" , "a")),
                                       (-7, ("zepto", "z")),
                                       (-8, ("yocto", "y")),
            ]),
            precision: 256,
            max_digits_after_zero: 3,
        }
    }
    pub fn eval(&self, e: &Expr) -> Option<Float> {
        use { ExprKind::*, Opcode::* };
        match &e.v {
            Constant(v) => Some(v.clone()),
            Ident(name) => {
                self.known.get(name).map(|x| {
                    //eprintln!("load  {} = {}", name, &x.render(self));
                    x.clone()
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
                                    Pow => a.pow(b),
                                    _ => Float::with_val(rug::float::prec_min(),rug::float::Special::Nan),
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
                self.store(l, &val);
                self.store(r, &val);
            }
        } else {
            panic!("Tried storing {} as a BinaryOp!", e.render(&self));
        }
    }
    pub fn store(&mut self, e: &Expr, val: &Float) {
        if let ExprKind::Ident(name) = &e.v {
            self.known.insert(name.to_string(), val.clone());
            //eprintln!("store {} = {}", name, val.render(self));
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
    Constant(Float),
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
impl From<Float> for Expr {
    fn from(val: Float) -> Self {
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

impl Render for Target {
    fn render(&self, scope: &Scope) -> String {
        use Target::*;
        match &self {
            ExprSet(set) => format!("{}", set.render(scope)),
            Expr(e) => format!("{}", e.render(scope)),
        }
    }
}
impl Render for ExprKind {
    fn render(&self, scope: &Scope) -> String {
        use ExprKind::*;
        match &self {
            Constant(v) => v.render(scope),
            Ident(name) => format!("{}", name),
            Function(n, a) => format!("{} ( {} )", n, a.render(scope)),
            UnaryOp(o, v) => format!("{}{{ {} }}", o, v.render(scope)),
            BinaryOp(l, o, r) => format!("{{ {} }} {} {{ {} }}", l.render(scope), o, r.render(scope)),
        }
    }
}
impl Render for Expr {
    fn render(&self, scope: &Scope) -> String {
        if let Some(unit) = &self.unit {
            format!("{} {{ {} }}", self.v.render(scope), unit)
        } else {
            format!("{}", self.v.render(scope))
        }
    }
}
impl Render for ExprSet {
    fn render(&self, scope: &Scope) -> String {
        format!("{}", self.iter().map(|e| e.render(scope)).collect::<Vec<String>>().join("; ~~~ "))
    }
}
impl Render for Float {
    fn render(&self, scope: &Scope) -> String {
        let (nv,s) = style_suffix(&self, scope);
        let mut nvstr = nv.to_string_radix_round(10, Some(3*8 + scope.max_digits_after_zero), rug::float::Round::Nearest);
        if let Some(dotidx) = nvstr.find('.') {
            let maxidx = dotidx + scope.max_digits_after_zero;
            if let Some(s) = nvstr.get(..maxidx+1) {
                nvstr = s.to_string();
            }
            let trailch: &[_] = &['0','.'];
            nvstr = nvstr.trim_end_matches(trailch).to_string();
        }
        format!("{}{}", nvstr, s.unwrap_or("".to_string()))
    }
}

fn closest_common_exp(mut val: Float) -> (Float, i32) {
    let mut n = 0;
    while val > 100_f64 {
        val = val / 1000_f64;
        n += 1;
    }
    while val < 1_f64 {
        val = val * 1000_f64;
        n -= 1;
    }
    (val, n)
}

fn style_suffix(val: &Float, scope: &Scope) -> (Float, Option<String>) {
    use RepStyle::*;
    let (nval, n) = closest_common_exp(val.clone());
    if n == 0 { return (nval, None) }
    match scope.repstyle {
        SiSuffix => {
            (nval, scope.si_suff_lut.get(&n).map(|x| x.1.to_string()))
        },
        TenExp => {
            (nval, Some(format!(" times 10 sup {{ {} }}", n)))
        },
        Scientific => {
            (nval, Some(format!("e{}", n)))
        },
    }
}


