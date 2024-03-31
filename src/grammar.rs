use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::io;

#[derive(Copy, Clone)]
struct NonTerminal<'a> {
    name: &'a str,
}

#[derive(Copy, Clone, Debug)]
struct Terminal<'a> {
    name: &'a str,
}

#[derive(Copy, Clone)]
enum Token<'a> {
    NT(NonTerminal<'a>),
    T(Terminal<'a>),
}

struct Rule<'a> {
    from: NonTerminal<'a>,
    to: Vec<Token<'a>>,
}

pub struct Grammar<'a> {
    nonterminals: HashMap<&'a str, NonTerminal<'a>>,
    terminals: HashMap<&'a str, Terminal<'a>>,
    rules: Vec<Rule<'a>>,
    start: NonTerminal<'a>,
}

#[derive(Debug)]
pub enum ParseError {
    IoError(io::Error),
    InvalidRule { line_num: usize },
    MissingStart,
    InvalidStart,
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
    pub fn from_rules(grammar: &'a str) -> Result<Self, ParseError> {
        let rule_regex = Regex::new(RULE_REGEX).unwrap();
        let terminal_regex = Regex::new(TERMINAL_REGEX).unwrap();
        let nonterminal_regex = Regex::new(NONTERMINAL_REGEX).unwrap();

        let mut terminals = HashMap::new();
        let mut nonterminals = HashMap::new();
        let mut rules = Vec::new();

        // Read the first line to get the start nonterminal.
        let mut lines = grammar.lines();
        let first_line = lines.next().ok_or(ParseError::MissingStart)?.trim();
        if !nonterminal_regex.is_match(first_line) {
            return Err(ParseError::InvalidStart);
        }
        let start = NonTerminal { name: first_line };
        nonterminals.insert(first_line, start);

        // Then build the rules.
        for (line_num, line) in lines.enumerate() {
            if !rule_regex.is_match(&line) {
                return Err(ParseError::InvalidRule { line_num });
            }
            let words: Vec<&str> = line.split_whitespace().collect();

            // Build the rule by iterating over the words.
            // Create nonterminals/terminals while doing so.
            let word = words[0];
            let from = NonTerminal { name: word };
            nonterminals.entry(word).or_insert(from);

            let mut to = Vec::new();
            for &word in &words[2..] {
                if terminal_regex.is_match(word) {
                    let terminal = Terminal { name: word };
                    terminals.entry(word).or_insert(terminal);
                    to.push(Token::T(terminal));
                } else {
                    let nonterminal = NonTerminal { name: word };
                    nonterminals.entry(word).or_insert(nonterminal);
                    to.push(Token::NT(nonterminal));
                }
            }
            rules.push(Rule { from, to });
        }

        Ok(Grammar {
            nonterminals,
            terminals,
            rules,
            start,
        })
    }
}

impl fmt::Display for NonTerminal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Terminal<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::NT(x) => write!(f, "{}", x),
            Token::T(x) => write!(f, "{}", x),
        }
    }
}

impl fmt::Display for Rule<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> ", self.from)?;
        for token in self.to.iter() {
            write!(f, "{} ", token)?
        }
        Ok(())
    }
}

impl fmt::Display for Grammar<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nonterminals: ")?;
        for (_, v) in self.nonterminals.iter() {
            write!(f, "{}, ", v)?;
        }
        writeln!(f)?;

        write!(f, "Terminals: ")?;
        for (_, v) in self.terminals.iter() {
            write!(f, "{}, ", v)?;
        }

        writeln!(f)?;
        writeln!(f, "Rules: ")?;

        for r in self.rules.iter() {
            writeln!(f, "{}", r)?;
        }

        write!(f, "Start: {}", self.start)?;

        Ok(())
    }
}
