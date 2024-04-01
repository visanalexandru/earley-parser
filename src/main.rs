use earley_parser::grammar::Grammar;
use std::fs;

fn main() {
    let rules = fs::read_to_string("grammar").unwrap();
    let grammar = Grammar::from_rules(&rules).unwrap();
    println!("{}", grammar);
    println!();
    grammar.parse("n+n*n");
}
