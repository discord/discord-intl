use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxToken, TextSize};
use slice_dst::SliceWithHeader;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, Range};
use std::rc::Rc;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SyntaxNodeHeader {
    pub kind: SyntaxKind,
    pub token_len: TextSize,
}

#[derive(Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct SyntaxNode(Rc<SliceWithHeader<SyntaxNodeHeader, SyntaxElement>>);

impl SyntaxNode {
    pub fn new<I>(kind: SyntaxKind, children: I) -> Self
    where
        I: IntoIterator<Item = SyntaxElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut token_len = 0;
        // Adding an inspection into the iterator means we can collect the token_len from all the
        // children when it gets read into the node header. Then, after the node is constructed,
        // we can assume the iterator has been exhausted and use the resulting `token_len` to set
        // the data for this node's header directly.
        let children = children
            .into_iter()
            .inspect(|child| token_len += child.token_len());
        let mut data =
            SliceWithHeader::new::<Rc<_>, _>(SyntaxNodeHeader { kind, token_len: 0 }, children);

        let header = &mut Rc::get_mut(&mut data).unwrap().header;
        header.token_len = token_len;
        Self(data)
    }

    pub fn is_tombstone(&self) -> bool {
        self.0.header.kind == SyntaxKind::TOMBSTONE
    }

    pub fn token_len(&self) -> TextSize {
        self.0.header.token_len
    }

    pub fn kind(&self) -> SyntaxKind {
        self.0.header.kind
    }

    pub fn is_token(&self) -> bool {
        self.0.header.kind.is_token()
    }

    pub fn required_node(&self, slot: usize) -> SyntaxNode {
        self.0.slice[slot].node().to_owned()
    }

    pub fn optional_node(&self, slot: usize) -> Option<SyntaxNode> {
        self.0.slice[slot].as_node().cloned()
    }

    pub fn required_token(&self, slot: usize) -> SyntaxToken {
        self.0.slice[slot].token().to_owned()
    }

    pub fn optional_token(&self, slot: usize) -> Option<SyntaxToken> {
        self.0.slice[slot].as_token().cloned()
    }

    pub fn children(&self) -> &[SyntaxElement] {
        self.0.slice.as_ref()
    }

    pub fn len(&self) -> usize {
        self.0.slice.len()
    }

    pub fn first_child(&self) -> Option<&SyntaxElement> {
        self.0.slice.get(0)
    }

    pub fn last_child(&self) -> Option<&SyntaxElement> {
        self.0.slice.get(self.0.slice.len() - 1)
    }

    pub fn get(&self, index: usize) -> Option<&SyntaxElement> {
        self.0.slice.get(index)
    }
}

impl Debug for SyntaxNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}(+{:?}) ", self.kind(), self.token_len()))?;
        if self.len() > 0 {
            f.debug_list().entries(self.children()).finish()
        } else {
            Ok(())
        }
    }
}

impl Index<usize> for SyntaxNode {
    type Output = SyntaxElement;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0.slice[index]
    }
}

impl Index<Range<usize>> for SyntaxNode {
    type Output = [SyntaxElement];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0.slice[index]
    }
}
