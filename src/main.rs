#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub pyseud2eqn);

mod ast;

fn print_expr(parser: &pyseud2eqn::ExprParser, v: &str) {
    match parser.parse(v) {
        Ok(x) => println!("{}", x),
        Err(x) => println!("{}", x),
    }
}

fn main() {
    let parser = pyseud2eqn::ExprParser::new();
    print_expr(&parser, "22__12");
    print_expr(&parser, "mamma_mia__superb1232");
}


#[test]
fn pyseud2eqn() {
    let p = pyseud2eqn::ExprParser::new();
    assert!(p.parse("22").unwrap().to_string() == "22");
    assert!(p.parse("26__2").unwrap().to_string() == "26 sup 2");
    assert!(p.parse("Vimp_i__8tell").unwrap().to_string() == "Vimp sub i sup 8tell");
    assert!(p.parse("Shungalung_integ").unwrap().to_string() == "Shungalung sub integ");
}
