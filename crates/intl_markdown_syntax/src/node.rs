use crate::iterators::SyntaxNodeTokenIter;
use crate::traits::EqIgnoreSpan;
use crate::{SyntaxElement, SyntaxKind, TextSize};
use slice_dst::SliceWithHeader;
use std::fmt::{Debug, Formatter};
use std::ops::{Index, Range};
use std::sync::Arc;

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct SyntaxNodeHeader {
    pub kind: SyntaxKind,
    /// Position of this node in the original source text.
    pub text_offset: TextSize,
    /// Byte length of all text contained by this node.
    pub text_len: TextSize,
}

#[derive(Clone, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SyntaxNode(Arc<SliceWithHeader<SyntaxNodeHeader, SyntaxElement>>);

impl SyntaxNode {
    pub fn new<I>(kind: SyntaxKind, text_offset: TextSize, children: I) -> Self
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
            let (text_len, children) = fixup_inline_content_children(text_offset, children);
            Self(SliceWithHeader::new(
                SyntaxNodeHeader {
                    kind,
                    text_offset,
                    text_len,
                },
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
            let mut data = SliceWithHeader::new::<Arc<_>, _>(
                SyntaxNodeHeader {
                    kind,
                    text_offset,
                    text_len: 0,
                },
                children,
            );
            let header = &mut Arc::get_mut(&mut data).unwrap().header;
            header.text_len = text_len;
            Self(data)
        }
    }

    pub fn text_offset(&self) -> TextSize {
        self.0.header.text_offset
    }

    pub fn text_len(&self) -> TextSize {
        self.0.header.text_len
    }

    pub fn source_position(&self) -> (usize, usize) {
        let offset = self.text_offset() as usize;
        let len = self.text_len() as usize;
        (offset, offset + len)
    }

    pub fn kind(&self) -> SyntaxKind {
        self.0.header.kind
    }

    pub fn children(&self) -> &[SyntaxElement] {
        self.0.slice.as_ref()
    }

    /// Returns the number of child elements contained by this node.
    pub fn len(&self) -> usize {
        self.0.slice.len()
    }

    pub fn get(&self, index: usize) -> Option<&SyntaxElement> {
        self.0.slice.get(index)
    }

    pub fn iter_tokens(&self) -> SyntaxNodeTokenIter {
        SyntaxNodeTokenIter::new(&self)
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

impl EqIgnoreSpan for SyntaxNode {
    fn eq_ignore_span(&self, rhs: &SyntaxNode) -> bool {
        let children = self.children();
        let rhs_children = rhs.children();

        if rhs_children.len() != children.len() {
            return false;
        }

        if self.kind() != rhs.kind() {
            return false;
        }

        children
            .iter()
            .zip(rhs_children.iter())
            .all(|(a, b)| a.eq_ignore_span(b))
    }
}

fn fixup_inline_content_children<I>(
    initial_offset: TextSize,
    children: I,
) -> (TextSize, Vec<SyntaxElement>)
where
    I: IntoIterator<Item = SyntaxElement>,
    I::IntoIter: ExactSizeIterator,
{
    let children = children.into_iter();
    let mut collected = Vec::with_capacity(children.len());
    let mut text_len = 0;
    let mut text_span_start_index: Option<usize> = None;
    for child in children {
        let child_len = child.text_len();
        match &child {
            SyntaxElement::Token(_) => {
                // Store the index of this token in the children list.
                text_span_start_index.get_or_insert(collected.len());
                collected.push(child);
            }
            _ => {
                if let Some(span_start) = text_span_start_index.take() {
                    let new_node = SyntaxNode::new(
                        SyntaxKind::TEXT_SPAN,
                        initial_offset + text_len,
                        collected.drain(span_start..),
                    )
                    .into();
                    collected.push(new_node);
                }
                collected.push(child)
            }
        }
        text_len += child_len;
    }

    if let Some(span_start) = text_span_start_index.take() {
        let new_node = SyntaxNode::new(
            SyntaxKind::TEXT_SPAN,
            initial_offset + text_len,
            collected.drain(span_start..),
        )
        .into();
        collected.push(new_node);
    }

    (text_len, collected)
}
