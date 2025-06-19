use crate::syntax::{SyntaxKind, SyntaxNode, SyntaxToken, TextPointer, TextSize};
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

    pub fn token_len(&self) -> TextSize {
        match self {
            NodeOrToken::Node(node) => node.token_len(),
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

#[derive(Debug, Clone)]
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

impl ExactSizeIterator for SyntaxElementChildren<'_> {
    fn len(&self) -> usize {
        self.elements.len()
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

    /// Return a new iterator of string slices where each slice spans as long as possible across
    /// multiple tokens, so long as they appear continuously in the source text. So, if the children
    /// are made up of tokens directly adjacent to each other in the source, then this iterator will
    /// only have a single chunk with the entire text collected together. If any token in the middle
    /// of the text is detached, then the tokens before it will be grouped together, the token will
    /// be in its own chunk (or connected with following adjacent tokens), and then chunking
    /// continues afterward.
    ///
    /// Each chunk is returns as a TextPointer to simplify working with the text and not worrying
    /// about lifetimes.
    ///
    /// If `trim_trailing` is set, the trailing trivia of the entire chunk will be excluded from the
    /// resulting text.
    pub fn contiguous_text_chunks(
        &self,
        options: ContiguousTokenChunksIteratorOptions,
    ) -> ContiguousTokenChunksIterator<'a> {
        ContiguousTokenChunksIterator::new(self.elements).with_options(options)
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

pub enum TrimKind {
    TrimNone,
    TrimLeading,
    TrimTrailing,
    TrimAll,
}

#[derive(Debug, Clone, Copy)]
pub struct ContiguousTokenChunksIteratorOptions {
    pub trim_leading: bool,
    pub trim_all_leading: bool,
    pub trim_trailing: bool,
    pub unescape: bool,
    pub html_entities: bool,
}

impl ContiguousTokenChunksIteratorOptions {
    pub fn include_all() -> Self {
        Self {
            trim_leading: false,
            trim_all_leading: false,
            trim_trailing: false,
            unescape: true,
            html_entities: true,
        }
    }

    pub fn trim_leading() -> Self {
        Self {
            trim_leading: true,
            trim_all_leading: false,
            trim_trailing: false,
            unescape: true,
            html_entities: true,
        }
    }

    pub fn trim_all_leading() -> Self {
        Self {
            trim_leading: true,
            trim_all_leading: true,
            trim_trailing: false,
            unescape: true,
            html_entities: true,
        }
    }

    pub fn trim_trailing() -> Self {
        Self {
            trim_leading: false,
            trim_all_leading: false,
            trim_trailing: true,
            unescape: true,
            html_entities: true,
        }
    }

    pub fn trim_all() -> Self {
        Self {
            trim_leading: true,
            trim_all_leading: true,
            trim_trailing: true,
            unescape: true,
            html_entities: true,
        }
    }

    pub fn with_unescape(mut self, unescape: bool) -> Self {
        self.unescape = unescape;
        self
    }

    pub fn unescape(&self) -> bool {
        self.unescape
    }

    pub fn with_html_entities(mut self, html_entities: bool) -> Self {
        self.html_entities = html_entities;
        self
    }

    pub fn html_entities(&self) -> bool {
        self.html_entities
    }

    pub fn trim_kind(&self, index: usize) -> TrimKind {
        let trim_leading = if index == 0 {
            self.trim_leading
        } else {
            self.trim_all_leading
        };
        if trim_leading && self.trim_trailing {
            TrimKind::TrimAll
        } else if trim_leading {
            TrimKind::TrimLeading
        } else if self.trim_trailing {
            TrimKind::TrimTrailing
        } else {
            TrimKind::TrimNone
        }
    }
}

impl Default for ContiguousTokenChunksIteratorOptions {
    fn default() -> Self {
        Self::trim_leading()
    }
}

#[derive(Debug)]
pub struct ContiguousTokenChunksIterator<'a> {
    elements: &'a [SyntaxElement],
    cursor: usize,
    options: ContiguousTokenChunksIteratorOptions,
}

impl<'a> ContiguousTokenChunksIterator<'a> {
    pub fn new(elements: &'a [SyntaxElement]) -> Self {
        Self {
            elements,
            cursor: 0,
            options: Default::default(),
        }
    }

    pub fn with_options(mut self, options: ContiguousTokenChunksIteratorOptions) -> Self {
        self.options = options;
        self
    }
}

impl Iterator for ContiguousTokenChunksIterator<'_> {
    type Item = TextPointer;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.elements.len() {
            return None;
        }

        let first_token = self.elements[self.cursor].token();
        let mut pointer = first_token.text_pointer().clone();
        if (self.options.trim_leading && self.cursor == 0) || self.options.trim_all_leading {
            pointer = first_token.text_pointer_with_no_leading_trivia()
        }

        self.cursor += 1;
        while self.cursor < self.elements.len() {
            let next = if self.options.trim_all_leading {
                &self.elements[self.cursor]
                    .token()
                    .text_pointer_with_no_leading_trivia()
            } else {
                self.elements[self.cursor].token().text_pointer()
            };
            if !pointer.is_adjacent_before(next) {
                break;
            }
            pointer = pointer.extend_back(next);
            self.cursor += 1;
        }

        if self.options.trim_trailing && self.cursor == self.elements.len() {
            pointer =
                pointer.trim_back(self.elements[self.cursor - 1].token().trailing_trivia_len());
        }

        Some(pointer)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.elements.len()))
    }
}
