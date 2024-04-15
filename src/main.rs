use earley_parser::grammar::{write_tree_to_dot, Grammar};
use std::fs;

fn main() {
    let rules = fs::read_to_string("grammar").unwrap();
    let grammar = Grammar::from_rules(&rules).unwrap();
    println!("{}", grammar);
    println!();

    let trees = grammar.parse("(n+n+(n*n)-n/n)");
    println!("Got {} derivation trees", trees.len());

    for tree in trees {
        let mut to = String::new();
        write_tree_to_dot(&mut to, &tree).unwrap();
        println!("{}", to);
        println!();
    }
}
