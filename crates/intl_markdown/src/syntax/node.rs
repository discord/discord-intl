use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxToken, TextSize};
use slice_dst::SliceWithHeader;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub struct SyntaxNodeHeader {
    pub kind: SyntaxKind,
    pub text_len: TextSize,
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
        let mut text_len = 0;
        // Adding an inspection into the iterator means we can collect the text_len from all the
        // children when it gets read into the node header. Then, after the node is constructed,
        // we can assume the iterator has been exhausted and use the resulting `text_len` to set
        // the data for this node's header directly.
        let children = children
            .into_iter()
            .inspect(|child| text_len += child.text_len());
        let mut data =
            SliceWithHeader::new::<Rc<_>, _>(SyntaxNodeHeader { kind, text_len: 0 }, children);

        let header = &mut Rc::get_mut(&mut data).unwrap().header;
        header.text_len = text_len;
        Self(data)
    }

    pub fn is_tombstone(&self) -> bool {
        self.0.header.kind == SyntaxKind::TOMBSTONE
    }

    pub fn text_len(&self) -> TextSize {
        self.0.header.text_len
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

    pub fn first_token(&self) -> Option<&SyntaxToken> {
        self.0
            .slice
            .iter()
            .find(|element| !matches!(element, SyntaxElement::Empty))?
            .first_token()
    }

    pub fn last_token(&self) -> Option<&SyntaxToken> {
        self.0
            .slice
            .iter()
            .rfind(|element| !matches!(element, SyntaxElement::Empty))?
            .last_token()
    }
}

impl Debug for SyntaxNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}(+{:?}) ", self.kind(), self.text_len()))?;
        if self.len() > 0 {
            f.debug_list().entries(self.children()).finish()
        } else {
            Ok(())
        }
    }
}
