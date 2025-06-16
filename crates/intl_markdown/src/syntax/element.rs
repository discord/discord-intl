use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxToken, TextSize};
use std::fmt::{Debug, Formatter};

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
    /// An Empty state allows an easy way to represent missing optional (or even required) nodes
    /// in a tree that preserves monotonicity of all types. If an optional field for a node is
    /// missing, its slot will still be taken up by this empty element, and calling code can know
    /// to skip, return None in its place, or error when a value is required.
    Empty,
}

impl<N: Debug, T: Debug> Debug for NodeOrToken<N, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeOrToken::Node(node) => node.fmt(f),
            NodeOrToken::Token(token) => token.fmt(f),
            NodeOrToken::Empty => f.debug_tuple("Empty").finish(),
        }
    }
}

impl<N, T> NodeOrToken<N, T> {
    pub fn token(&self) -> &T {
        match &self {
            NodeOrToken::Token(token) => token,
            NodeOrToken::Node(_) => panic!("Called `NodeOrToken::token` on a Node value"),
            NodeOrToken::Empty => panic!("Called `NodeOrToken::token` on an empty value"),
        }
    }

    pub fn node(&self) -> &N {
        match &self {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!("Called `NodeOrToken::node` on a Token value"),
            NodeOrToken::Empty => panic!("Called `NodeOrToken::node` on an empty value"),
        }
    }

    pub fn as_token(&self) -> Option<&T> {
        match &self {
            NodeOrToken::Token(token) => Some(token),
            _ => None,
        }
    }

    pub fn as_node(&self) -> Option<&N> {
        match &self {
            NodeOrToken::Node(node) => Some(node),
            _ => None,
        }
    }

    pub fn into_token(self) -> T {
        match self {
            NodeOrToken::Token(token) => token,
            NodeOrToken::Node(_) => panic!("Called `NodeOrToken::into_token` on a Node value"),
            NodeOrToken::Empty => panic!("Called `NodeOrToken::into_token` on an empty value"),
        }
    }

    pub fn into_node(self) -> N {
        match self {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!("Called `NodeOrToken::into_node` on a Token value"),
            NodeOrToken::Empty => panic!("Called `NodeOrToken::into_node` on an empty  value"),
        }
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, NodeOrToken::Empty)
    }
}

pub type SyntaxElement = NodeOrToken<SyntaxNode, SyntaxToken>;
impl SyntaxElement {
    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
            NodeOrToken::Empty => SyntaxKind::TOMBSTONE,
        }
    }

    pub fn text_len(&self) -> TextSize {
        match self {
            NodeOrToken::Node(node) => node.text_len(),
            NodeOrToken::Token(token) => token.text_len(),
            NodeOrToken::Empty => 0,
        }
    }

    pub fn first_token(&self) -> Option<&SyntaxToken> {
        match self {
            NodeOrToken::Node(node) => node.first_token(),
            NodeOrToken::Token(token) => Some(token),
            NodeOrToken::Empty => None,
        }
    }

    pub fn last_token(&self) -> Option<&SyntaxToken> {
        match self {
            NodeOrToken::Node(node) => node.last_token(),
            NodeOrToken::Token(token) => Some(token),
            NodeOrToken::Empty => None,
        }
    }
}

impl From<SyntaxNode> for SyntaxElement {
    fn from(node: SyntaxNode) -> Self {
        NodeOrToken::Node(node)
    }
}

impl From<SyntaxToken> for SyntaxElement {
    fn from(token: SyntaxToken) -> Self {
        NodeOrToken::Token(token)
    }
}

pub struct SyntaxElementChildren<'a> {
    elements: &'a [SyntaxElement],
    cursor: usize,
}

impl<'a> SyntaxElementChildren<'a> {
    pub fn new(elements: &'a [SyntaxElement]) -> Self {
        Self {
            elements,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for SyntaxElementChildren<'a> {
    type Item = SyntaxElement;
    fn next(&mut self) -> Option<Self::Item> {
        let element = self.elements.get(self.cursor).cloned();
        self.cursor += 1;
        element
    }
}

pub struct SyntaxNodeChildren<'a> {
    elements: &'a [SyntaxElement],
    cursor: usize,
}

impl<'a> SyntaxNodeChildren<'a> {
    pub fn new(elements: &'a [SyntaxElement]) -> Self {
        Self {
            elements,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for SyntaxNodeChildren<'a> {
    type Item = SyntaxNode;
    fn next(&mut self) -> Option<Self::Item> {
        let element = self
            .elements
            .get(self.cursor)
            .map(|element| element.node().to_owned());
        self.cursor += 1;
        element
    }
}

pub struct SyntaxTokenChildren<'a> {
    elements: &'a [SyntaxElement],
    cursor: usize,
}

impl<'a> SyntaxTokenChildren<'a> {
    pub fn new(elements: &'a [SyntaxElement]) -> Self {
        Self {
            elements,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for SyntaxTokenChildren<'a> {
    type Item = SyntaxToken;
    fn next(&mut self) -> Option<Self::Item> {
        let element = self
            .elements
            .get(self.cursor)
            .map(|element| element.token().to_owned());
        self.cursor += 1;
        element
    }
}
