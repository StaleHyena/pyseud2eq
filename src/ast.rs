use std::collections::HashMap;
use std::fmt;
use std::ops;
use std::vec::Vec;
use rug::{ self, Float, ops::Pow };

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RepStyle {
    SiSuffix,
    TenExp,
    Scientific,
    Verbatim,
}

pub struct Scope {
    pub known: HashMap<String, Float>,
    pub repstyle: RepStyle,
    pub autocalc_ident: String,
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
                    eprintln!("load  {} = {}", name, &x.render(self));
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
            BinaryOp(lhs, op, rhs) => {
                match op {
                    At | NotEquals | GreaterThan | LesserThan | GtEquals | LtEquals => None,
                    o if ASSIGNING_OPS.contains(o) => self.eval(rhs).or_else(|| { self.eval(lhs) }),
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
    fn process_inner(&mut self, e: &mut Expr, val: Option<&Float>, assign: bool) {
        use ExprKind::*;
        match &mut e.v {
            BinaryOp(l, o, r) => {
                let lhv = self.eval(l);
                let rhv = self.eval(r);
                let sval = rhv.as_ref().or_else(|| lhv.as_ref().or_else(|| val));
                let assign = ASSIGNING_OPS.contains(o);
                self.process_inner(l, sval, assign);
                self.process_inner(r, sval, assign);
            },
            Ident(name) => {
                if let Some(v) = val {
                    if name.to_string() == self.autocalc_ident {
                        e.v = Constant(v.clone());
                    }
                    if assign { self.store(&e, &v); }
                }
            },
            _ => (),
        }
    }
    pub fn process(&mut self, e: &mut Expr) {
        self.process_inner(e, None, false);
    }
    pub fn store(&mut self, e: &Expr, val: &Float) {
        if let ExprKind::Ident(name) = &e.v {
            self.known.insert(name.to_string(), val.clone());
            eprintln!("store {} = {}", name, val.render(self));
        }
    }
}

pub enum Target {
    ExprSet(ExprSet),
    Expr(Box<Expr>),
    Config,
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
}
impl From<Float> for Expr {
    fn from(val: Float) -> Self {
        Expr::new(ExprKind::Constant(val))
    }
}

// TODO: add more eqn ops
#[derive(Clone, Copy, PartialEq, Eq)]
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

static ASSIGNING_OPS: &[Opcode] = &[
    Opcode::Equals,
    Opcode::ApproxEquals,
];

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
        eprintln!("render target");
        use Target::*;
        match self {
            ExprSet(set) => format!("{}", set.render(scope)),
            Expr(e) => format!("{}", e.render(scope)),
            Config => "".to_string(),
        }
    }
}
impl Render for ExprKind {
    fn render(&self, scope: &Scope) -> String {
        use ExprKind::*;
        match self {
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
        eprintln!("render expr");
        if let Some(unit) = &self.unit {
            format!("{} {{ {} }}", self.v.render(scope), unit)
        } else {
            format!("{}", self.v.render(scope))
        }
    }
}
impl Render for ExprSet {
    fn render(&self, scope: &Scope) -> String {
        eprintln!("render exprset");
        format!("{}", self.iter().map(|e| e.render(scope)).collect::<Vec<String>>().join("; ~~~ "))
    }
}
impl Render for Float {
    fn render(&self, scope: &Scope) -> String {
        eprintln!("render float {}", self);
        let (val,s,n) = style_suffix(self.clone(), scope.repstyle);
        let mut vstr = val.to_string_radix(10, Some(3*8));

        if let Some(dotidx) = vstr.find('.') {
            // this is a mess
            if let RepStyle::Verbatim = scope.repstyle {
                vstr.remove(dotidx);
                let new_dotidx: isize = (dotidx as isize) + (n as isize);
                match new_dotidx {
                    n if n <= 0 => {
                        vstr.insert_str(0, &"0".repeat(n.abs() as usize + 1));
                        vstr.insert(1, '.');
                    },
                    n if n > 0 => {
                        vstr.insert(n as usize, '.');
                    },
                    _ => { unreachable!(); }
                }
            } else {
                let maxidx = dotidx + scope.max_digits_after_zero;
                if let Some(s) = vstr.get(..=maxidx) {
                    vstr = s.to_string();
                }
            };
            vstr = vstr.trim_end_matches('0').trim_end_matches('.').to_string();
        }
        format!("{}{}", vstr, s.unwrap_or("".to_string()))
    }
}

fn closest_common_exp(val_og: Float, e: u32) -> (Float, isize) {
    if val_og == 0 { return (val_og, 0) }
    let snum = Float::with_val(val_og.prec(), val_og.signum_ref());
    let mut val = val_og.clone().abs();
    let mut n = 0_isize;
    let one = Float::with_val(val.prec(), 1_u32);
    let e = Float::with_val(val.prec(), e);
    let c = Float::with_val(val.prec(), e.exp10_ref());
    let mut a = Float::with_val(val.prec(), &e - &one);
    a = a.exp10();
    while val > a {
        val = val / &c;
        n += 1;
    }
    while val < one {
        val = val * &c;
        n -= 1;
    }
    let rr = (snum * val, n);
    eprintln!("closest common exp for {} is ({}, {})", val_og, rr.0, rr.1);
    rr
}

fn style_suffix(val: Float, style: RepStyle) -> (Float, Option<String>, isize) {
    use RepStyle::*;
    match style {
        SiSuffix => {
            let (nval, n) = closest_common_exp(val.clone(), 3);
            if n == 0 { return (nval, None, n) }
            else if n > 8 || n < -8 {
                return style_suffix(val, TenExp);
            }
            (nval, SI_SUFF_LUT[(n + 8) as usize].map(|x| x.1.to_string()), n)
        },
        TenExp => {
            let (val, n) = closest_common_exp(val, 1);
            if n == 0 { return (val, None, n) }
            (val, Some(format!(" times 10 sup {{ {} }}", n)), n)
        },
        Scientific | Verbatim => {
            let (val, n) = closest_common_exp(val, 1);
            if n == 0 { return (val, None, n) }
            (val, if style == Verbatim { None } else { Some(format!("e{}", n)) }, n)
        },
    }
}

static SI_SUFF_LUT: &[Option<(&'static str, &'static str)>] =
&[
   Some(("yocto", "y")),
   Some(("zepto", "z")),
   Some(("atto" , "a")),
   Some(("femto", "f")),
   Some(("pico" , "p")),
   Some(("nano" , "n")),
   Some(("micro", "µ")),
   Some(("milli", "m")),
   None,
   Some(("kilo" , "k")),
   Some(("mega" , "M")),
   Some(("giga" , "G")),
   Some(("tera" , "T")),
   Some(("peta" , "P")),
   Some(("exa"  , "E")),
   Some(("zetta", "Z")),
   Some(("yotta", "Y")),
];

