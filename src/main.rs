#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub pyseud2eqn);
use regex::Regex;
use std::io::{self, BufRead, BufReader};
mod ast;

#[allow(unused)]
fn print_target(s: &mut ast::Scope, parser: &pyseud2eqn::TargetParser, v: &str) {
    println!("{}", parser.parse(s, v).unwrap());
}

#[allow(unused)]
fn print_expr(s: &mut ast::Scope, parser: &pyseud2eqn::ExprParser, v: &str) {
    println!("{}", parser.parse(s, v).unwrap());
}

fn main() -> std::io::Result<()> {
    //let args: Vec<String> = std::env::args().collect();
    let reader = Box::new(BufReader::new(io::stdin()));

    let beg_txt = r"^[.](?i)EQPY";
    let end_txt = r"^[.](?i)ENPY";
    let beg = Regex::new(beg_txt).unwrap();
    let end = Regex::new(end_txt).unwrap();
    let mut on = false;
    let parser = pyseud2eqn::TargetParser::new();
    let mut s = ast::Scope::new();
    for line in reader.lines() {
        if let Ok(line) = line {
            if end.is_match(&line) {
                on = false;
            } else if beg.is_match(&line) {
                on = true;
            } else if on {
                match parser.parse(&mut s, &line.to_string()) {
                    Ok(r) => println!(".EQ\n{}\n.EN", r),
                    Err(e) => {
                        println!(".LP\nInvalid EQ, {}", e);
                    }
                }
            } else {
                println!("{}", line);
            }
        }
    }

    eprintln!("known at exit: {:?}", s.known);
    Ok(())
}

#[test]
fn sanity() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse(&mut s, "çæ").is_err());
    assert!(p.parse(&mut s, "\\|/").is_err());
    assert!(p.parse(&mut s, ",;?").is_err());
    assert!(p.parse(&mut s, "66").unwrap().to_string() == "66");
    assert!(p.parse(&mut s, "-12").unwrap().to_string() == "-12");
}

#[test]
fn factor_ops() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse(&mut s, "26__2").unwrap().to_string() == "{ 26 } sup { 2 }");
    assert!(
        p.parse(&mut s, "Imp_i__-8:(m/s)").unwrap().to_string()
            == "{ { Vimp } sub { i } } sup { -8 ~ tell }"
    );
    assert!(
        p.parse(&mut s, "Shungalung_integ").unwrap().to_string() == "{ Shungalung } sub { integ }"
    );
    assert!(
        p.parse(&mut s, "dododo_final**plus").unwrap().to_string() == "dododo sub final sup plus"
    );
    assert!(p.parse(&mut s, "2 / 4 * 12").unwrap().to_string() == "2 over 4 times 12");
    assert!(p.parse(&mut s, "98*0.2/pi").unwrap().to_string() == "98 times 0.2 over pi");
}

#[test]
fn expr_ops() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse(&mut s, "1 + ae").unwrap().to_string() == "1 + ae");
    assert!(p.parse(&mut s, "000 - ooo").unwrap().to_string() == "000 - ooo");
}

#[test]
fn paren() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse(&mut s, "(45)").unwrap().to_string() == "{ 45 }");
    assert!(p.parse(&mut s, "301 / (pi/tau)").unwrap().to_string() == "301 over { pi over tau }");
    assert!(
        p.parse(&mut s, "((-87+78)**(1/omega))/(alpha_1)__45")
            .unwrap()
            .to_string()
            == "{ { -87 + 78 } sup { 1 over omega } } over { alpha sub 1 } sup 45"
    );
    // FIXME cover a(b), (a)b and other variations on parenthesis inside terms/idents
}

#[test]
fn visible_paren() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse(&mut s, "((2_2))").unwrap().to_string() == "{ ( 2 sub 2 ) }");
    assert!(p.parse(&mut s, "((((2**2))))").unwrap().to_string() == "{ ( 2 sup 2 ) }");
    // FIXME wrong behaviour
    assert!(p.parse(&mut s, "((2) 8)").unwrap().to_string() == "{ { 2 } 8 }");
}

#[test]
fn equations() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::EquationParser::new();
    assert!(p.parse(&mut s, "0 = 0").unwrap().to_string() == "{ 0 = 0 }");
    assert!(p.parse(&mut s, "X__'= Y__'").unwrap().to_string() == "{ X sup ' = Y sup ' }");
    assert!(p.parse(&mut s, "pi != tau/4").unwrap().to_string() == "{ pi != tau over 4 }");
    assert!(p.parse(&mut s, "0 <= 1").unwrap().to_string() == "{ 0 <= 1 }");
    assert!(
        p.parse(&mut s, "0 <= 1 > -2 ~= -1.9").unwrap().to_string()
            == "{ 0 <= 1 > -2 approx~ -1.9 }"
    );
    assert!(
        p.parse(&mut s, "0 = 0 = (0) != 1 != 12")
            .unwrap()
            .to_string()
            == "{ 0 = 0 = { 0 } != 1 != 12 }"
    );
}

#[test]
fn equation_sets() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::ExprSetParser::new();

    assert!(
        p.parse(&mut s, "0 = 0; 1 ~= 0").unwrap().to_string()
            == "{ 0 = 0 }; ~~~ { 1 approx~ 0 }; ~~~ "
    );
    assert!(
        p.parse(&mut s, "2 < 6; abc ~= bcd; 12; (e)e != E0")
            .unwrap()
            .to_string()
            == "{ 2 < 6 }; ~~~ { abc approx~ bcd }; ~~~ 12; ~~~ { { e e } != E0 }; ~~~ "
    );
    // FIXME cover multiple expression equations in a set
}

#[test]
fn targets() {
    let mut s = ast::Scope::new();
    let p = pyseud2eqn::TargetParser::new();
    print_target(&mut s, &p, ".EQPY 12 EQPY");
    print_target(&mut s, &p, ".EQPY 12 = alpha / 2 .EQPY");
}
