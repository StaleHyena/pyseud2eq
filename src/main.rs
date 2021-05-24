#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub pyseud2eqn);
use std::io::{self, BufReader, BufRead};
use regex::Regex;

mod ast;

#[allow(unused)]
fn print_target(parser: &pyseud2eqn::TargetParser, v: &str) {
    println!("{}", parser.parse(v).unwrap());
}

#[allow(unused)]
fn print_expr(parser: &pyseud2eqn::ExprParser, v: &str) {
    println!("{}", parser.parse(v).unwrap());
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        use std::io::*;
        return Err(Error::new(ErrorKind::NotFound, "No input file!"));
    }
    
    let filename = args.get(1).unwrap();
    let reader: Box<dyn BufRead> = match filename.as_str() {
        "-" => Box::new(BufReader::new(io::stdin())),
        _ => Box::new(BufReader::new(std::fs::File::open(filename).unwrap())),
    };

    let parser = pyseud2eqn::TargetParser::new();
    let prefix = Regex::new(r"^\.(?i)EQPY").unwrap();
    for line in reader.lines() {
        if let Ok(line) = line {
            if prefix.is_match(line.as_str()) {
                match parser.parse(line.as_str()) {
                    Ok(r) => println!(".EQ\n{}\n.EN", r),
                    Err(e) => {
                        println!(".LP\nInvalid EQ");
                        eprintln!("{}", e);
                    }
                }
            } else {
                println!("{}", line);
            }
        }
    }

    Ok(())
}

#[test]
fn sanity() {
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse("çæ").is_err());
    assert!(p.parse("\\|/").is_err());
    assert!(p.parse(",;?").is_err());
    assert!(p.parse("66").unwrap().to_string()
                 == "66");
    assert!(p.parse("-12").unwrap().to_string()
                 == "-12");
}

#[test]
fn factor_ops() {
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse("26__2").unwrap().to_string()
                 == "26 sup 2");
    assert!(p.parse("Vimp_i__8tell").unwrap().to_string()
                 == "Vimp sub i sup 8tell");
    assert!(p.parse("Shungalung_integ").unwrap().to_string()
                 == "Shungalung sub integ");
    assert!(p.parse("dododo_final**plus").unwrap().to_string()
                 == "dododo sub final sup plus");
    assert!(p.parse("2 / 4 * 12").unwrap().to_string()
                 == "2 over 4 times 12");
    assert!(p.parse("98*0.2/pi").unwrap().to_string()
                 == "98 times 0.2 over pi");
}

#[test]
fn expr_ops() {
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse("1 + ae").unwrap().to_string()
                 == "1 + ae");
    assert!(p.parse("000 - ooo").unwrap().to_string()
                 == "000 - ooo");
}

#[test]
fn paren() {
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse("(45)").unwrap().to_string()
                 == "{ 45 }");
    assert!(p.parse("301 / (pi/tau)").unwrap().to_string()
                 == "301 over { pi over tau }");
    assert!(p.parse("((-87+78)**(1/omega))/(alpha_1)__45").unwrap().to_string()
                 == "{ { -87 + 78 } sup { 1 over omega } } over { alpha sub 1 } sup 45");
    // FIXME cover a(b), (a)b and other variations on parenthesis inside terms/idents
}

#[test]
fn visible_paren() {
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse("((2_2))").unwrap().to_string()
                 == "{ ( 2 sub 2 ) }");
    assert!(p.parse("((((2**2))))").unwrap().to_string()
                 == "{ ( 2 sup 2 ) }");
    // FIXME wrong behaviour
    assert!(p.parse("((2) 8)").unwrap().to_string()
                 == "{ { 2 } 8 }");
}

#[test]
fn equations() {
    let p = pyseud2eqn::EquationParser::new();
    assert!(p.parse("0 = 0").unwrap().to_string()
                 == "{ 0 = 0 }");
    assert!(p.parse("X__'= Y__'").unwrap().to_string()
                 == "{ X sup ' = Y sup ' }");
    assert!(p.parse("pi != tau/4").unwrap().to_string()
                 == "{ pi != tau over 4 }");
    assert!(p.parse("0 <= 1").unwrap().to_string()
                 == "{ 0 <= 1 }");
    assert!(p.parse("0 <= 1 > -2 ~= -1.9").unwrap().to_string()
                 == "{ 0 <= 1 > -2 approx~ -1.9 }");
    assert!(p.parse("0 = 0 = (0) != 1 != 12").unwrap().to_string()
                 == "{ 0 = 0 = { 0 } != 1 != 12 }");
}

#[test]
fn equation_sets() {
    let p = pyseud2eqn::ExprSetParser::new();

    assert!(p.parse("0 = 0; 1 ~= 0").unwrap().to_string()
                 == "{ 0 = 0 }; ~~~ { 1 approx~ 0 }; ~~~ ");
    assert!(p.parse("2 < 6; abc ~= bcd; 12; (e)e != E0").unwrap().to_string()
                 == "{ 2 < 6 }; ~~~ { abc approx~ bcd }; ~~~ 12; ~~~ { { e e } != E0 }; ~~~ ");
    // FIXME cover multiple expression equations in a set
}

#[test]
fn targets() {
    let p = pyseud2eqn::TargetParser::new();
    print_target(&p, ".EQPY 12 EQPY");
    print_target(&p, ".EQPY 12 = alpha / 2 .EQPY");
}

