use super::*;
use std::collections::HashSet;

/// Each state consists of:
/// - the production currently being matched
/// - the current position in that production
/// - the position in the input at witch the matching began.
#[derive(Hash, Eq, PartialEq)]
struct EarleyState<'a> {
    rule: Rule<'a>,
    dot: usize,
    origin: usize,
}

/// The early table has k+1 sets, where k is the length
/// of the word to recognize.
/// Each set i holds the states at position i.
struct EarleyTable<'a> {
    sets: Vec<HashSet<EarleyState<'a>>>,
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
    fn prediction(&self, early_table: &mut EarleyTable<'a>, position: usize) {
        loop {
            let mut to_add = Vec::new();
            for state in early_table.sets[position].iter() {
                let dot = state.dot;
                let current_token = state.rule.to[dot];

                if let Token::NT(nonterminal) = current_token {
                    for rule in self.rules.iter() {
                        if rule.from == nonterminal {
                            let new_state = EarleyState {
                                rule: rule.clone(),
                                dot: 0,
                                origin: position,
                            };
                            to_add.push(new_state);
                        }
                    }
                }
            }
            let old_size = early_table.sets[position].len();
            for state in to_add {
                early_table.sets[position].insert(state);
            }
            if early_table.sets[position].len() == old_size {
                break;
            }
        }
    }
    pub fn parse(&self, s: &str) {
        let mut table = EarleyTable::new(s.len() + 1);

        // Add the starting rules.
        for rule in self.rules.iter() {
            if rule.from == self.start {
                let new_state = EarleyState {
                    rule: rule.clone(),
                    origin: 0,
                    dot: 0,
                };
                table.sets[0].insert(new_state);
            }
        }

        for position in 0..s.len() {
            self.prediction(&mut table, position);
            break;
        }
        println!("Early table:");
        println!("{}", table);
    }
}

impl fmt::Display for EarleyState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rule: {}, Dot: {}, Origin: {}",
            self.rule, self.dot, self.origin
        )
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
