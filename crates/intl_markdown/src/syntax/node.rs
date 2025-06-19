use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxToken, TextSize};
use slice_dst::SliceWithHeader;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, Range};
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
        // For cases where the node being built expects to contain a homogenous set of nodes, it's
        // possible that the tree builder will have ended up leaving a heterogeneous set of tokens
        // and nodes in `children`, which isn't allowed. Because inline content can be created both
        // from a regular node (like from `parse_inline`) _or_ from a deferred node (like in
        // `process_emphasis`), it's not straightforward to do this work earlier on. Instead, we
        // can at least try to limit how much extra work is done, but _every_ inline content node
        // will have to go through this process to ensure the final node structure is valid.
        if kind.expects_inline_node_children() {
            let (text_len, children) = fixup_inline_content_children(children);
            Self(SliceWithHeader::new(
                SyntaxNodeHeader { kind, text_len },
                children,
            ))
        } else {
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
        f.write_fmt(format_args!("{:?}(+{:?}) ", self.kind(), self.text_len()))?;
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

fn fixup_inline_content_children<I>(children: I) -> (TextSize, Vec<SyntaxElement>)
where
    I: IntoIterator<Item = SyntaxElement>,
    I::IntoIter: ExactSizeIterator,
{
    let children = children.into_iter();
    let mut collected = Vec::with_capacity(children.len());
    let mut text_len = 0;
    let mut text_span_start_index: Option<usize> = None;
    for child in children {
        text_len += child.text_len();
        match &child {
            SyntaxElement::Token(_) => {
                // Store the index of this token in the children list.
                text_span_start_index.get_or_insert(collected.len());
                collected.push(child);
            }
            _ => {
                if let Some(span_start) = text_span_start_index.take() {
                    let new_node =
                        SyntaxNode::new(SyntaxKind::TEXT_SPAN, collected.drain(span_start..))
                            .into();
                    collected.push(new_node);
                }
                collected.push(child)
            }
        }
    }

    if let Some(span_start) = text_span_start_index.take() {
        let new_node = SyntaxNode::new(SyntaxKind::TEXT_SPAN, collected.drain(span_start..)).into();
        collected.push(new_node);
    }

    (text_len, collected)
}
