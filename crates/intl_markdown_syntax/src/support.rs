use crate::FromSyntax;
use crate::{SyntaxNode, SyntaxToken};

pub fn required_node<N: FromSyntax>(node: &SyntaxNode, slot: usize) -> N {
    N::from_syntax(node[slot].node().clone())
}

pub fn optional_node<N: FromSyntax>(node: &SyntaxNode, slot: usize) -> Option<N> {
    node[slot]
        .as_node()
        .map(|syntax| N::from_syntax(syntax.clone()))
}

pub fn required_token(node: &SyntaxNode, slot: usize) -> SyntaxToken {
    node[slot].token().clone()
}

pub fn optional_token(node: &SyntaxNode, slot: usize) -> Option<SyntaxToken> {
    node[slot].as_token().cloned()
}
