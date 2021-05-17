#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub pyseud2eqn);

mod ast;

fn print_expr(parser: &pyseud2eqn::ExprParser, v: &str) {
    println!("{}\n{}", v, parser.parse(v).unwrap());
}

fn main() {
    let p = pyseud2eqn::ExprParser::new();
    //print_expr(&p, "301 / (pi/tau)");
    print_expr(&p, "((-87+78)**(1/omega))/((alpha_1)__45)");
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
}

