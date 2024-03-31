use regex::Regex;
use std::collections::HashMap;
use std::io;

#[derive(Copy, Clone, Debug)]
struct NonTerminal<'a> {
    name: &'a str,
}

#[derive(Copy, Clone, Debug)]
struct Terminal<'a> {
    name: &'a str,
}

#[derive(Copy, Clone, Debug)]
enum Token<'a> {
    NT(NonTerminal<'a>),
    T(Terminal<'a>),
}

#[derive(Debug)]
struct Rule<'a> {
    from: NonTerminal<'a>,
    to: Vec<Token<'a>>,
}

#[derive(Debug)]
pub struct Grammar<'a> {
    non_terminals: HashMap<&'a str, NonTerminal<'a>>,
    terminals: HashMap<&'a str, Terminal<'a>>,
    rules: Vec<Rule<'a>>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    InvalidRule,
}

impl From<io::Error> for ParseError {
    fn from(value: io::Error) -> Self {
        ParseError::IoError(value)
    }
}

const TERMINAL_REGEX: &'static str = r"^[a-z+\-\*0-9]$";
const NONTERMINAL_REGEX: &'static str = r"^[A-Z]+$";
const RULE_REGEX: &'static str = r"^([A-Z]+)\s+->(\s+([A-Z]+|[a-z+\-\*0-9]))*$";

impl<'a> Grammar<'a> {
    fn is_terminal(s: &str) -> bool {
        Regex::new(TERMINAL_REGEX).unwrap().is_match(s)
    }
    fn is_nonterminal(s: &str) -> bool {
        Regex::new(NONTERMINAL_REGEX).unwrap().is_match(s)
    }

    pub fn from_rules(grammar: &'a str) -> Result<Self, ParseError> {
        let rule_regex = Regex::new(RULE_REGEX).unwrap();

        let mut terminals = HashMap::new();
        let mut non_terminals = HashMap::new();
        let mut rules = Vec::new();

        for line in grammar.lines() {
            let line = line.trim();
            if !rule_regex.is_match(&line) {
                return Err(ParseError::InvalidRule);
            }

            let words: Vec<&str> = line.split_whitespace().collect();

            // Insert all new terminal and nonterminals into the grammar.
            for &word in words.iter() {
                if Grammar::is_terminal(word) {
                    terminals.entry(word).or_insert(Terminal { name: word });
                } else if Grammar::is_nonterminal(word) {
                    non_terminals
                        .entry(word)
                        .or_insert(NonTerminal { name: word });
                }
            }

            // Create the rule.
            let from = *non_terminals.get(words[0]).unwrap();
            let mut to = Vec::new();

            for &word in &words[2..] {
                if Grammar::is_terminal(word) {
                    to.push(Token::T(*terminals.get(word).unwrap()));
                } else {
                    to.push(Token::NT(*non_terminals.get(word).unwrap()));
                }
            }
            rules.push(Rule { from, to });
        }

        Ok(Grammar {
            non_terminals,
            terminals,
            rules,
        })
    }
}
