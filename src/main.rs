use earley_parser::grammar::Grammar;
use std::fs;

fn main() {
    let rules = fs::read_to_string("grammar3").unwrap();
    let grammar = Grammar::from_rules(&rules).unwrap();
    println!("{}", grammar);
    println!();
    // grammar.parse("n+n*n");
    // grammar.parse("n+n"); //1
    // grammar.parse("acb"); //2
    // grammar.parse("aacbb"); //2
    grammar.parse("1+(2*3-4)"); //3
}
