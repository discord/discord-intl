use crate::syntax::token::SyntaxTokenData;
use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, TextPointer};
use crate::SourceText;
use std::fmt::Debug;
use std::ptr;
use std::sync::Arc;

/// A transparent representation of an index and state of the tree, used for creating markers to
/// indicate where nodes should begin and end. `child_idx` is used as a starting marker, indicating
/// an index in the in-progress tree's child list that can be immediately jumped to when starting a
/// node. `text_offset` is the length of text parsed into the tree when the marker was made.
///
/// These values are _not_ interchangeable or pairable in any other direction, because the tree
/// builder's list is mutable and will change child indices for _current_ nodes arbitrarily using
/// [`DeferredNode`]. The only guarantees the parser and the builder make are that deferred nodes
/// can never reach beyond the start of the current child. As such, `child_idx` will always be
/// valid as a starting point for a node, but the ending point can only be known confidently by
/// using a token offset.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TreeMarker {
    parent_idx: usize,
    child_idx: usize,
    text_offset: usize,
}

impl TreeMarker {
    pub fn new(parent_idx: usize, child_idx: usize, text_offset: usize) -> Self {
        Self {
            parent_idx,
            child_idx,
            text_offset,
        }
    }

    /// NOTE: This only works when each child element is a single byte. Resolving multi-byte tokens
    /// and text offsets would require calculating from the content of the tree.
    pub fn add_child_offset(mut self, offset: usize) -> Self {
        self.child_idx += offset;
        self
    }

    pub fn add_text_offset(mut self, offset: usize) -> Self {
        self.text_offset += offset;
        self
    }

    /// NOTE: This only works when each child element is a single byte. Resolving multi-byte tokens
    /// and text offsets would require calculating from the content of the tree.
    pub fn sub_child_offset(mut self, offset: usize) -> Self {
        self.child_idx -= offset;
        self
    }

    pub fn sub_text_offset(mut self, offset: usize) -> Self {
        self.child_idx -= offset;
        self.text_offset -= offset;
        self
    }

    pub fn child_index(&self) -> usize {
        self.child_idx
    }

    pub fn text_offset(&self) -> usize {
        self.text_offset
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord)]
struct DeferredNode {
    kind: SyntaxKind,
    start: TreeMarker,
    end: TreeMarker,
    order: usize,
}

impl PartialOrd for DeferredNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            // Order by outermost starting position first (lower number first),
            // then outermost ending point (greater number first),
            // then outward order of insertion (inserted later first)
            self.start
                .cmp(&other.start)
                .then(self.end.cmp(&other.end).reverse())
                .then(self.order.cmp(&other.order).reverse()),
        )
    }
}

#[derive(Debug)]
pub struct TreeBuilder {
    /// Stack of node kinds to their starting point in the children list.
    parents: Vec<(SyntaxKind, usize)>,
    /// List of children that have not yet been grouped into a parent node. At the end of parsing,
    /// this list should be length 1 and contain the root node of the tree.
    children: Vec<SyntaxElement>,
    /// Count of how much text has been parsed into the tree. Markers are created using the current
    /// text offset as a persistent position marker in the tree, no matter how nodes may get
    /// restructured.
    text_offset: usize,
    /// List of child nodes to create only when finishing the current node. Deferred nodes are used
    /// to wrap earlier sections of the child list with new nodes without disturbing the indices of
    /// any other checkpoints or building actions in the meantime.
    ///
    /// When calling `finish_node`, this list will be drained and the deferred nodes will be created
    /// before wrapping them all in the newly finished node.
    ///
    /// Deferred nodes cannot be created until all children of the current node have been parsed.
    deferred_nodes: Vec<DeferredNode>,
    /// Reusable scratch for mutating node lists without extra heap allocation. This list is only
    /// valid to be used within a single method call. Each usage should first clear the list and
    /// then perform work.
    scratch: Vec<SyntaxElement>,
    /// Raw pointer to the last token's data to be able to append trailing trivia in O(1) and avoid
    /// iterating and cloning finished nodes.
    last_token_data: *mut SyntaxTokenData,
    /// Any leading trivia that has no token to attach itself to yet. Since pointers can be merged
    /// cheaply with contiguous text, it's just as efficient to save this and then apply it to the
    /// next token after it's pushed into the tree.
    pending_leading_trivia: TextPointer,
}

impl TreeBuilder {
    pub fn new(source: SourceText) -> Self {
        TreeBuilder {
            parents: Vec::with_capacity(8),
            children: Vec::with_capacity(4),
            deferred_nodes: vec![],
            text_offset: 0,
            scratch: Vec::with_capacity(4),
            last_token_data: ptr::null_mut(),
            pending_leading_trivia: TextPointer::new(source, 0, 0),
        }
    }

    pub fn push_token(&mut self, token: SyntaxToken) {
        // SAFETY: We should be the only ones mutating this data, meaning we can safely take a
        // mutable reference to it to amend leading trivia to later on.
        self.last_token_data = Arc::as_ptr(&token.raw_data()) as *mut SyntaxTokenData;
        self.text_offset += token.len() as usize;
        // Add any unconsumed leading trivia to the token before adding it to the tree.
        if !self.pending_leading_trivia.is_empty() {
            // Only add the pending trivia length to the current text offset once it's been
            // applied to the tree. If it's pushed before being applied, the computed offsets
            // of markers taken on tokens with leading trivia will point to the wrong index,
            // using the start of the token itself _without_ the leading trivia instead.
            // See test `tests::spec_regression::regression_1` for an example.
            self.text_offset += self.pending_leading_trivia.len();
            // SAFETY: We only do this while building the tree, meaning we know there can and
            // should only be a single other reference to this token (in whatever list or node
            // contains it), and that it won't be mutated by anything else.
            unsafe { &mut *self.last_token_data }
                .prepend_leading_trivia(&self.pending_leading_trivia);
            self.pending_leading_trivia.clear();
        }

        self.children.push(token.into());
    }

    pub fn prepend_leading_trivia(&mut self, trivia_text: &str) {
        if trivia_text.is_empty() {
            return;
        }
        // `extend_back` here is used because we're still working left-to-right, meaning the leading
        // trivia is built by tacking on the new text until we reach the start of the actual token.
        self.pending_leading_trivia = self.pending_leading_trivia.extend_back(trivia_text);
    }

    pub fn append_trailing_trivia(&mut self, trivia_text: &str) {
        if !trivia_text.is_empty() {
            // SAFETY: We only do this while building the tree, meaning we know there can and should
            // only be a single other reference to this token (in whatever list or node contains it),
            // and that it won't be mutated by anything else.
            match unsafe { self.last_token_data.as_mut() } {
                Some(data) => {
                    data.append_trailing_trivia(trivia_text);
                    self.text_offset += trivia_text.len()
                }
                None => self.prepend_leading_trivia(trivia_text),
            }
        }
    }

    pub fn add_trivia(&mut self, trailing_text: &str, next_leading_text: &str) {
        if !next_leading_text.is_empty() {
            self.prepend_leading_trivia(next_leading_text);
        }
        if !trailing_text.is_empty() {
            self.append_trailing_trivia(trailing_text);
        }
    }

    pub fn push_missing(&mut self) {
        self.children.push(SyntaxElement::Empty);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.parents.push((kind, self.children.len()));
    }

    pub fn start_node_at(&mut self, kind: SyntaxKind, marker: TreeMarker) {
        assert!(
            marker.parent_idx <= self.parents.len(),
            "Tree marker is no longer valid, referencing parents that no longer exist"
        );
        assert!(
            marker.child_idx <= self.children.len(),
            "Tree marker is no longer valid, referencing children that no longer exist"
        );
        assert!(
            marker.text_offset <= self.text_offset,
            "Tree marker is no longer valid, referencing tokens that no longer exist"
        );
        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(
                marker.child_idx >= first_child,
                "Tree checkpoint is not valid while in the process of building another node within it. Finish that node before attempting to start another node outside of it."
            );
        }
        self.parents.push((kind, marker.child_idx));
    }

    pub fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().unwrap();

        let children_iter = self.children.drain(first_child..);
        let node = if self.deferred_nodes.len() == 0 {
            // Most things won't have deferred nodes, so we can fast-path finishing the parent
            // directly from iterating the children.
            SyntaxNode::new(kind, children_iter).into()
        } else {
            // But if there are any deferred nodes, then we need to mutate the list accordingly,
            // which is most easily done by collecting them into a new vector and mutating that,
            // even if it's not the _most_ efficient way possible.
            children_iter.collect_into(&mut self.scratch);
            self.deferred_nodes.sort();
            for node in self.deferred_nodes.drain(..).rev() {
                let start = node.start.child_idx - first_child;
                let end = {
                    let mut offset = node.start.text_offset;
                    let target = node.end.text_offset;
                    let mut end = start;
                    for element in &self.scratch[start..] {
                        if offset == target {
                            break;
                        }
                        offset += element.text_len() as usize;
                        debug_assert!(
                            offset <= target,
                            "Deferred node of kind {:?} had mismatched ending boundary. Expected {target} total bytes, but this element spanned from {} to {}",
                            node.kind,
                            offset - element.text_len() as usize,
                            offset,
                        );
                        end += 1;
                    }
                    debug_assert!(
                        offset == target,
                        "Failed to find exact matching end boundary for deferred node of kind {:?}. Expected {target} bytes, read {offset}",
                        node.kind
                    );
                    end
                };
                let new_node = SyntaxNode::new(node.kind, self.scratch.drain(start..end)).into();
                self.scratch.insert(start, new_node);
            }
            SyntaxNode::new(kind, self.scratch.drain(..)).into()
        };

        self.children.push(node);
    }

    /// Create a new [DeferredNode] that will collect all children between the two checkpoints and
    /// move them into a new node before the next parent node is finished.
    pub fn wrap_with_node(&mut self, kind: SyntaxKind, start: TreeMarker, end: TreeMarker) {
        self.deferred_nodes.push(DeferredNode {
            kind,
            start,
            end,
            order: self.deferred_nodes.len(),
        });
    }

    pub fn last_element(&self) -> Option<&SyntaxElement> {
        self.children.last()
    }

    pub fn checkpoint(&self) -> TreeMarker {
        TreeMarker {
            parent_idx: self.parents.len(),
            child_idx: self.children.len(),
            text_offset: self.text_offset,
        }
    }

    pub fn rewind(&mut self, marker: TreeMarker) {
        assert!(
            marker.parent_idx <= self.parents.len(),
            "Tree marker is no longer valid, referencing parents that no longer exist"
        );
        assert!(
            marker.child_idx <= self.children.len(),
            "Tree marker is no longer valid, referencing children that no longer exist"
        );
        assert!(
            marker.text_offset <= self.text_offset,
            "Tree marker is no longer valid, referencing tokens that no longer exist"
        );
        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(
                marker.child_idx >= first_child,
                "Tree marker is not valid while in the process of building another node within it. Finish that node before attempting to rewind"
            );
        }
        self.parents.truncate(marker.parent_idx);
        self.children.truncate(marker.child_idx);
        self.text_offset = marker.text_offset;
    }

    pub fn finish(mut self) -> SyntaxElement {
        assert_eq!(
            self.children.len(),
            1,
            "Tree building should finish with only the root node left in the tree"
        );
        self.children.pop().unwrap()
    }
}
