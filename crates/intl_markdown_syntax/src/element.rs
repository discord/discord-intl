use crate::iterators::PositionalIterator;
use crate::traits::EqIgnoreSpan;
use crate::{SyntaxKind, SyntaxNode, SyntaxToken, TextSize};
use std::fmt::{Debug, Formatter};

#[derive(Eq, Clone, PartialEq, Hash)]
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
pub type SyntaxElementRef<'a> = NodeOrToken<&'a SyntaxNode, &'a SyntaxToken>;

impl SyntaxElement {
    pub fn as_ref(&self) -> SyntaxElementRef {
        self.into()
    }

    pub fn kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(node) => node.kind(),
            NodeOrToken::Token(token) => token.kind(),
            NodeOrToken::Empty => SyntaxKind::TOMBSTONE,
        }
    }

    pub fn text_offset(&self) -> TextSize {
        match self {
            NodeOrToken::Node(node) => node.text_offset(),
            NodeOrToken::Token(token) => token.text_offset(),
            NodeOrToken::Empty => 0,
        }
    }

    pub fn text_len(&self) -> TextSize {
        match self {
            NodeOrToken::Node(node) => node.text_len(),
            NodeOrToken::Token(token) => token.len(),
            NodeOrToken::Empty => 0,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            NodeOrToken::Node(node) => node.len(),
            NodeOrToken::Token(_) => 1,
            NodeOrToken::Empty => 0,
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

impl From<&SyntaxNode> for SyntaxElement {
    fn from(node: &SyntaxNode) -> Self {
        NodeOrToken::Node(node.clone())
    }
}

impl From<&SyntaxToken> for SyntaxElement {
    fn from(token: &SyntaxToken) -> Self {
        NodeOrToken::Token(token.clone())
    }
}

impl EqIgnoreSpan for SyntaxElement {
    fn eq_ignore_span(&self, rhs: &Self) -> bool {
        match rhs {
            SyntaxElement::Token(rhs_token) => self
                .as_token()
                .is_some_and(|token| token.eq_ignore_span(rhs_token)),
            SyntaxElement::Node(rhs_node) => self
                .as_node()
                .is_some_and(|node| node.eq_ignore_span(rhs_node)),
            SyntaxElement::Empty => matches!(self, SyntaxElement::Empty),
        }
    }
}

impl SyntaxElementRef<'_> {
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

    pub fn len(&self) -> usize {
        match self {
            NodeOrToken::Node(node) => node.len(),
            NodeOrToken::Token(_) => 1,
            NodeOrToken::Empty => 0,
        }
    }
}

impl<'a> From<&'a SyntaxNode> for SyntaxElementRef<'a> {
    fn from(node: &'a SyntaxNode) -> Self {
        SyntaxElementRef::Node(node)
    }
}
impl<'a> From<&'a SyntaxToken> for SyntaxElementRef<'a> {
    fn from(node: &'a SyntaxToken) -> Self {
        SyntaxElementRef::Token(node)
    }
}
impl<'a> From<&'a SyntaxElement> for SyntaxElementRef<'a> {
    fn from(node: &'a SyntaxElement) -> Self {
        match node {
            SyntaxElement::Token(token) => SyntaxElementRef::Token(token),
            SyntaxElement::Node(node) => SyntaxElementRef::Node(node),
            SyntaxElement::Empty => SyntaxElementRef::Empty,
        }
    }
}

#[derive(Debug, Clone)]
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

impl ExactSizeIterator for SyntaxNodeChildren<'_> {
    fn len(&self) -> usize {
        self.elements.len()
    }
}
impl PositionalIterator for SyntaxNodeChildren<'_> {}

#[derive(Debug, Clone)]
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

impl ExactSizeIterator for SyntaxTokenChildren<'_> {
    fn len(&self) -> usize {
        self.elements.len()
    }
}
impl PositionalIterator for SyntaxTokenChildren<'_> {}
