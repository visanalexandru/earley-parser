use super::*;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::hash::Hash;
use std::rc::Rc;

mod parse_tree;
pub use parse_tree::render_tree;
pub use parse_tree::write_tree_to_dot;
pub use parse_tree::ParseNode;

/// Each state consists of:
/// - the production currently being matched
/// - the current position in that production
/// - the position in the input at witch the matching began.
#[derive(Eq, PartialEq, Hash)]
struct EarleyState<'a> {
    rule: &'a Rule<'a>,
    dot: usize,
    origin: usize,
    children: Vec<Rc<EarleyState<'a>>>,
}

impl<'a> EarleyState<'a> {
    fn new(rule: &'a Rule<'a>, dot: usize, origin: usize) -> Self {
        EarleyState {
            rule,
            dot,
            origin,
            children: Vec::new(),
        }
    }

    /// A state is finished if the dot is at the end of the production.
    fn is_finished(&self) -> bool {
        self.dot == self.rule.to.len()
    }

    fn current_token(&self) -> Token<'a> {
        self.rule.to[self.dot]
    }
}

/// The early table has k+1 sets, where k is the length
/// of the word to recognize.
/// Each set i holds the states at position i.
struct EarleyTable<'a> {
    sets: Vec<HashSet<Rc<EarleyState<'a>>>>,
}

impl<'a> EarleyTable<'a> {
    fn new(size: usize) -> Self {
        let mut sets = Vec::new();
        for _ in 0..size {
            sets.push(HashSet::new())
        }
        EarleyTable { sets }
    }
}

impl<'a> Grammar<'a> {
    /// For each state
    fn prediction<'g>(&'g self, early_table: &mut EarleyTable<'g>, k: usize) {
        let mut to_add = Vec::new();
        for state in early_table.sets[k].iter() {
            if state.is_finished() {
                continue;
            }

            let current_token = state.current_token();

            let nonterminal = match current_token {
                Token::T(_) => continue,
                Token::NT(n) => n,
            };

            for rule in self.rules.iter() {
                if rule.from == nonterminal {
                    to_add.push(Rc::new(EarleyState::new(rule, 0, k)));
                }
            }
        }

        for state in to_add {
            early_table.sets[k].insert(state);
        }
    }

    fn scan(&self, early_table: &mut EarleyTable<'a>, k: usize, next_char: char) {
        let mut to_add = Vec::new();

        for state in early_table.sets[k].iter() {
            if state.is_finished() {
                continue;
            }

            let current_token = state.current_token();

            let terminal = match current_token {
                Token::NT(_) => continue,
                Token::T(t) => t,
            };

            if terminal.content != next_char {
                continue;
            }

            to_add.push(Rc::new(EarleyState {
                rule: state.rule,
                dot: state.dot + 1,
                origin: state.origin,
                children: state.children.clone(),
            }));
        }

        for state in to_add {
            early_table.sets[k + 1].insert(state);
        }
    }

    fn complete(&self, early_table: &mut EarleyTable<'a>, k: usize) {
        let mut to_add = Vec::new();

        for state in early_table.sets[k].iter() {
            // We only look at finished states.
            if !state.is_finished() {
                continue;
            }

            let current_nonterminal = state.rule.from;
            let origin = state.origin;

            for old_state in early_table.sets[origin].iter() {
                // Find old states that are waiting for the current_nonterminal to be matched.
                if old_state.is_finished() {
                    continue;
                }

                let current_token = old_state.current_token();
                let nonterminal = match current_token {
                    Token::T(_) => continue,
                    Token::NT(n) => n,
                };

                if nonterminal == current_nonterminal {
                    let mut new_children_list = old_state.children.clone();
                    new_children_list.push(state.clone());

                    to_add.push(Rc::new(EarleyState {
                        rule: old_state.rule,
                        dot: old_state.dot + 1,
                        origin: old_state.origin,
                        children: new_children_list,
                    }));
                }
            }
        }
        for state in to_add {
            early_table.sets[k].insert(state);
        }
    }

    pub fn parse(&self, s: &str) -> Vec<Rc<ParseNode>> {
        let mut table = EarleyTable::new(s.len() + 1);

        // Add the starting rules.
        for rule in self.rules.iter() {
            if rule.from == self.start {
                table.sets[0].insert(Rc::new(EarleyState::new(rule, 0, 0)));
            }
        }

        for (position, c) in s.chars().enumerate() {
            // Repeat prediction, scan, completion until no new states
            // can be added to the current set.
            loop {
                let old_size = table.sets[position].len();
                self.prediction(&mut table, position);
                self.scan(&mut table, position, c);
                self.complete(&mut table, position);

                if table.sets[position].len() == old_size {
                    break;
                }
            }
        }

        let last = s.len();
        loop {
            let old_size = table.sets[last].len();
            self.prediction(&mut table, last);
            self.complete(&mut table, last);

            if table.sets[last].len() == old_size {
                break;
            }
        }

        println!("Earley table:");
        println!("{}", table);

        let mut result = Vec::new();

        for state in table.sets[last].iter() {
            if state.rule.from == self.start && state.is_finished() && state.origin == 0 {
                let tree = parse_tree::build_parse_tree(&state);
                result.push(tree)
            }
        }
        result
    }
}

impl fmt::Display for EarleyState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rule: {} -> ", self.rule.from)?;
        for (index, token) in self.rule.to.iter().enumerate() {
            if index == self.dot {
                write!(f, ".")?;
            }
            write!(f, "{}", token)?;
        }
        if self.dot == self.rule.to.len() {
            write!(f, ".")?;
        }
        write!(f, "  Origin: {}, Dot: {}", self.origin, self.dot)
    }
}

impl fmt::Display for EarleyTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, states) in self.sets.iter().enumerate() {
            writeln!(f, "S{}", i)?;
            for state in states {
                writeln!(f, "{}", state)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Gets the string from the derivation by collecting the leaf nodes.
    fn evaluate_parse_tree<'a>(root: &ParseNode<'a>) -> String {
        if let Token::T(_) = root.token {
            format!("{}", root.token)
        } else {
            let mut result = String::new();
            for child in root.children.iter() {
                result.push_str(&evaluate_parse_tree(&child));
            }
            result
        }
    }

    #[test]
    fn test_expression_grammar() {
        let grammar_string = "EXP
        EXP -> EXP + EXP
        EXP -> EXP * EXP
        EXP -> EXP - EXP
        EXP -> EXP / EXP
        EXP -> ( EXP )
        EXP -> n";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("(n+n+(n*n)-n/n)");
        assert_eq!(trees.len(), 14);
        trees
            .iter()
            .for_each(|root| assert_eq!(evaluate_parse_tree(root), "(n+n+(n*n)-n/n)"));

        let trees = grammar.parse("n*n+n+(n+(n*n+(n)-n-(n-((n)))))");
        assert_eq!(trees.len(), 70);
        trees.iter().for_each(|root| {
            assert_eq!(evaluate_parse_tree(root), "n*n+n+(n+(n*n+(n)-n-(n-((n)))))")
        });

        let trees = grammar.parse("((n)+n-)");
        assert_eq!(trees.len(), 0);

        let trees = grammar.parse("(((n)*(((n)+(((n)))))))");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "(((n)*(((n)+(((n)))))))");
    }

    #[test]
    fn test_palindrome_grammar() {
        let grammar_string = "S
        S -> a S a
        S -> b S b 
        S ->
        S -> a
        S -> b";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("abba");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "abba");

        let trees = grammar.parse("aabab");
        assert_eq!(trees.len(), 0);

        let trees = grammar.parse("aabaa");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "aabaa");
    }

    #[test]
    fn test_paranthesis_grammar() {
        let grammar_string = "S
        S -> ( S ) S
        S -> ";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("(()()((()())))");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "(()()((()())))");

        let trees = grammar.parse("(()(())()((()())))()()");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "(()(())()((()())))()()");

        let trees = grammar.parse("(()(()))((()())))()()");
        assert_eq!(trees.len(), 0);
    }

    #[test]
    fn test_grammar_ab() {
        let grammar_string = "S
        S -> A B 
        S -> B 
        A -> B A 
        A -> a 
        B -> A
        B -> b";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("bab");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "bab");
    }

    #[test]
    fn test_grammar_aa() {
        let grammar_string = "S
        S -> A A
        A -> a A
        A -> b
        A -> a A
        B -> b";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("bab");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "bab");
    }

    #[test]
    fn test_grammar_lambda() {
        let grammar_string = "S
        S -> a b C d e
        C -> D
        D -> E
        E -> ";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("abde");
        assert_eq!(trees.len(), 1);
        assert_eq!(evaluate_parse_tree(&trees[0]), "abde");
    }

    #[test]
    fn test_grammar_many_derivations() {
        let grammar_string = "S
        S -> S S 
        S -> a";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("aaaaaa");
        assert_eq!(trees.len(), 42);
        trees
            .iter()
            .for_each(|root| assert_eq!(evaluate_parse_tree(root), "aaaaaa"));

        let trees = grammar.parse("aaaaaaa");
        assert_eq!(trees.len(), 132);
        trees
            .iter()
            .for_each(|root| assert_eq!(evaluate_parse_tree(root), "aaaaaaa"));
    }

    #[test]
    fn test_grammar_empty() {
        let grammar_string = "S";
        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let trees = grammar.parse("aaaaaa");
        assert_eq!(trees.len(), 0);
    }

    #[test]
    fn test_grammar_nlp() {
        let grammar_string = "S
        S -> NP VP
        VP -> VP PP 
        VP -> V NP 
        VP -> V
        PP -> P NP
        NP -> DET N 
        NP -> N 
        NP -> PN 
        NP -> DET A N
        NP -> A NP
        A -> ADV A 
        A -> A A
        ADV -> t o o 
        ADV -> v e r y 
        ADV -> q u i t e 
        PN -> s h e
        PN -> h e 
        A -> f r e s h
        A -> t a s t y
        A -> s i l v e r
        N -> f i s h
        N -> f o r k 
        N -> a p p l e
        V -> e a t s 
        DET -> a 
        DET -> a n 
        DET -> t h e 
        P -> w i t h";

        let grammar = Grammar::from_rules(&grammar_string).unwrap();

        let sentences = vec![
            "sheeats",
            "sheeatsanapple",
            "sheeatsfreshtastyapple",
            "sheeatsafish",
            "sheeatsafishwithafork",
            "sheeatsafishwithasilverfork",
            "sheeatsaquitefreshfishwithasilverfork",
        ];

        let num_trees = vec![1, 1, 2, 1, 1, 1, 1];

        for (&sentence, &num_trees) in sentences.iter().zip(num_trees.iter()) {
            let trees = grammar.parse(sentence);
            assert_eq!(trees.len(), num_trees);
            trees
                .iter()
                .for_each(|tree| assert_eq!(evaluate_parse_tree(tree), sentence));
        }
    }
}
