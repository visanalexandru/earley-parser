use earley_parser::grammar::{render_tree, write_tree_to_dot, Grammar};
use std::fs;
use std::io;

fn main() {
    let rules = fs::read_to_string("grammar").unwrap();
    let grammar = Grammar::from_rules(&rules).unwrap();
    println!("{}", grammar);
    println!("Enter your words:");

    loop {
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        let line = line.trim();
        let trees = grammar.parse(line);

        println!("Got {} derivation trees", trees.len());

        for (index, tree) in trees.iter().enumerate() {
            let mut to = String::new();
            write_tree_to_dot(&mut to, &tree).unwrap();
            let path = format!("tree_{}.svg", index);
            render_tree(&tree, &path).unwrap();
        }
    }
}
