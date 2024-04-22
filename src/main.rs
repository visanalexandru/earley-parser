use earley_parser::grammar::{render_tree, write_tree_to_dot, Grammar};
use std::fs;
fn main() {
    let rules = fs::read_to_string("grammar-expression").unwrap();
    let grammar = Grammar::from_rules(&rules).unwrap();
    println!("{}", grammar);
    println!();

    let trees = grammar.parse("n+n+n");
    println!("Got {} derivation trees", trees.len());

    for (index, tree) in trees.iter().enumerate() {
        let mut to = String::new();
        write_tree_to_dot(&mut to, &tree).unwrap();
        let path = format!("tree_{}.svg", index);
        render_tree(&tree, &path).unwrap();
    }
}
