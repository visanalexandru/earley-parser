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

impl<'a> EarleyState<'a> {
    fn new(rule: Rule<'a>, dot: usize, origin: usize) -> Self {
        EarleyState { rule, dot, origin }
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
    /// For each state
    fn prediction(&self, early_table: &mut EarleyTable<'a>, k: usize) {
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
                    to_add.push(EarleyState::new(rule.clone(), 0, k));
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

            to_add.push(EarleyState::new(
                state.rule.clone(),
                state.dot + 1,
                state.origin,
            ));
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
                    to_add.push(EarleyState::new(
                        old_state.rule.clone(),
                        old_state.dot + 1,
                        old_state.origin,
                    ));
                }
            }
        }
        for state in to_add {
            early_table.sets[k].insert(state);
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
