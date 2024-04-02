use super::*;
use std::cmp::{Eq, PartialEq};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

/// Each state consists of:
/// - the production currently being matched
/// - the current position in that production
/// - the position in the input at witch the matching began.
#[derive(Eq)]
struct EarleyState<'a> {
    rule: &'a Rule<'a>,
    dot: usize,
    origin: usize,
    children: Vec<Rc<EarleyState<'a>>>,
}

/// Implement the PartialEq for the EarleyState.
/// Ignore children.
impl<'a> PartialEq for EarleyState<'a> {
    fn eq(&self, other: &Self) -> bool {
        return self.rule == other.rule && self.dot == other.dot && self.origin == other.origin;
    }
}

/// Implement the Hash trait for the EarleyState.
/// Ignore children.
impl<'a> Hash for EarleyState<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rule.hash(state);
        self.dot.hash(state);
        self.origin.hash(state);
    }
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

    pub fn parse(&self, s: &str) {
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

        for state in table.sets[last].iter() {
            if state.rule.from == self.start && state.is_finished() && state.origin == 0 {
                println!("Solution: {}", state);
            }
        }
    }
}

impl fmt::Display for EarleyState<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Rule: {}, Dot: {}, Origin: {}",
            self.rule, self.dot, self.origin
        )?;

        if !self.children.is_empty() {
            write!(f, ", Children: ")?;
            for child in self.children.iter() {
                write!(f, "[{}]", child)?;
            }
        }

        Ok(())
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
