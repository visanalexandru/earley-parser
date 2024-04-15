use earley_parser::grammar::{write_tree_to_dot, Grammar};
use std::fs;

fn main() {
    let rules = fs::read_to_string("grammar").unwrap();
    let grammar = Grammar::from_rules(&rules).unwrap();
    println!("{}", grammar);
    println!();
    let parse_tree = grammar.parse("n+n*n+n+n*n").unwrap(); //3
    let mut to = String::new();
    write_tree_to_dot(&mut to, &parse_tree).unwrap();
    println!("{}", to);
}
