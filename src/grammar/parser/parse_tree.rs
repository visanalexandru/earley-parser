use super::*;
use std::fmt;

/// A parse node consists of a token and a list of child nodes.
/// For leaf nodes, the token is a Terminal.
/// For non-leaf nodes, the token is a Nonterminal.
pub struct ParseNode<'a> {
    pub token: Token<'a>,
    pub children: Vec<Rc<ParseNode<'a>>>,
}

pub fn build_parse_tree<'a>(state: &EarleyState<'a>) -> Rc<ParseNode<'a>> {
    let node_token = Token::NT(state.rule.from);

    let mut node_children = Vec::new();
    let mut state_children = state.children.iter();

    // For each terminal symbol in the production, create a new leaf parse node.
    // For each nonterminal symbol in the production, get the corresponding state
    // by advancing the state_children iterator.
    for token in state.rule.to.iter() {
        let node_child = match token {
            terminal @ Token::T(_) => Rc::new(ParseNode {
                token: *terminal,
                children: Vec::new(),
            }),

            Token::NT(_) => build_parse_tree(state_children.next().unwrap()),
        };

        node_children.push(node_child);
    }

    Rc::new(ParseNode {
        token: node_token,
        children: node_children,
    })
}

fn write_subtree_to_dot<'a, W>(
    to: &mut W,
    node: &ParseNode<'a>,
    current_id: &mut usize,
) -> Result<usize, fmt::Error>
where
    W: fmt::Write,
{
    let mut children_ids = Vec::new();
    for child in node.children.iter() {
        children_ids.push(write_subtree_to_dot(to, child, current_id)?);
    }
    *current_id += 1;

    to.write_str(&format!("{} [label=\"{}\"]\n", *current_id, node.token))?;

    for id in children_ids {
        to.write_str(&format!("{} -> {}\n", *current_id, id))?;
    }

    Ok(*current_id)
}

pub fn write_tree_to_dot<'a, W>(to: &mut W, root: &ParseNode<'a>) -> Result<(), fmt::Error>
where
    W: fmt::Write,
{
    to.write_str("digraph G{\n")?;
    let mut curr_id = 0;
    write_subtree_to_dot(to, root, &mut curr_id)?;
    to.write_str("}")
}