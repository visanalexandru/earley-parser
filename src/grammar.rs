use const_format;
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::io;

mod parser;
pub use parser::render_tree;
pub use parser::write_tree_to_dot;
pub use parser::ParseNode;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct NonTerminal<'a> {
    name: &'a str,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Terminal {
    content: char,
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum Token<'a> {
    NT(NonTerminal<'a>),
    T(Terminal),
}

/// A production rule is a pair (from, to) where from is a nonterminal
/// and to is a string of terminals/nonterminals.
#[derive(Clone, Hash, Eq, PartialEq)]
struct Rule<'a> {
    from: NonTerminal<'a>,
    to: Vec<Token<'a>>,
}

/// A context free grammar.
pub struct Grammar<'a> {
    nonterminals: HashMap<&'a str, NonTerminal<'a>>,
    terminals: HashMap<&'a str, Terminal>,
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

const TERMINAL_REGEX: &'static str = r"[a-z+\-\*0-9\(\)/]";
const NONTERMINAL_REGEX: &'static str = r"[A-Z]+";
const RULE_REGEX: &'static str = const_format::formatcp!(
    r"^{}\s+->(\s+({}|{}))*$",
    NONTERMINAL_REGEX,
    NONTERMINAL_REGEX,
    TERMINAL_REGEX
);

impl<'a> Grammar<'a> {
    /// Reads the grammar rules and constructs the grammar.
    pub fn from_rules(grammar: &'a str) -> Result<Self, ParseError> {
        let rule_regex = Regex::new(RULE_REGEX).unwrap();
        let terminal_regex = Regex::new(TERMINAL_REGEX).unwrap();
        let first_line_regex = Regex::new(&format!(r"^{}$", NONTERMINAL_REGEX)).unwrap();

        let mut terminals = HashMap::new();
        let mut nonterminals = HashMap::new();
        let mut rules = Vec::new();

        // Read the first line to get the start nonterminal.
        let mut lines = grammar.lines();
        let first_line = lines.next().ok_or(ParseError::MissingStart)?.trim();
        if !first_line_regex.is_match(first_line) {
            return Err(ParseError::InvalidStart);
        }
        let start = NonTerminal { name: first_line };
        nonterminals.insert(first_line, start);

        // Then build the rules.
        for (line_num, line) in lines.enumerate() {
            let line = line.trim();
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
                    let terminal = Terminal {
                        content: word.chars().next().unwrap(),
                    };
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

impl fmt::Display for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
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
