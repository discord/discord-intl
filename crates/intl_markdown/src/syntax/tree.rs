use crate::syntax::token::SyntaxTokenData;
pub(crate) use crate::syntax::TextSize;
use crate::syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken, TextPointer};
use std::fmt::Debug;
use std::ops::Deref;
use std::ptr;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TreeIndex(u32);

impl From<u32> for TreeIndex {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Deref for TreeIndex {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct TreeCheckpoint {
    parents_idx: usize,
    children_idx: usize,
}

impl TreeCheckpoint {
    pub fn get(&self) -> usize {
        self.children_idx
    }
}

// Converting a number to a tree checkpoint is not _fully_ valid, since it won't be able to
// reference an accurate parent_idx. But for cases where a marker wants to wrap an arbitrary span
// of tokens, this is useful to transparently convert them.
impl From<TextSize> for TreeCheckpoint {
    fn from(value: TextSize) -> Self {
        TreeCheckpoint {
            parents_idx: 0,
            children_idx: value as usize,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord)]
struct DeferredNode {
    kind: SyntaxKind,
    start: usize,
    end: usize,
}

impl PartialOrd for DeferredNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.start
                .cmp(&other.start)
                .then(self.end.cmp(&other.end).reverse()),
        )
    }
}

#[derive(Debug)]
pub struct TreeBuilder {
    /// Stack of node kinds to their starting point in the children list
    parents: Vec<(SyntaxKind, usize)>,
    /// List of children that have not yet been grouped into a parent node. At the end of parsing,
    /// this list should be length 1 and contain the root node of the tree.
    children: Vec<SyntaxElement>,
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
    last_token_data: *mut SyntaxTokenData,
    pending_leading_trivia: TextPointer,
}

impl TreeBuilder {
    pub fn new() -> Self {
        TreeBuilder {
            parents: Vec::with_capacity(8),
            children: Vec::with_capacity(4),
            deferred_nodes: vec![],
            scratch: Vec::with_capacity(4),
            last_token_data: ptr::null_mut(),
            pending_leading_trivia: TextPointer::default(),
        }
    }

    pub fn push_token(&mut self, token: SyntaxToken) {
        // SAFETY: We should be the only ones mutating this data, meaning we can safely
        self.last_token_data = Rc::as_ptr(&token.raw_data()) as *mut SyntaxTokenData;
        self.children.push(token.into());

        // Add any unconsumed leading trivia to the token after it's been added.
        if !self.pending_leading_trivia.is_empty() {
            // SAFETY: We only do this while building the tree, meaning we know there can and should
            // only be a single other reference to this token (in whatever list or node contains it),
            // and that it won't be mutated by anything else.
            unsafe { &mut *self.last_token_data }
                .prepend_leading_trivia(&self.pending_leading_trivia);
            self.pending_leading_trivia = TextPointer::default();
        }
    }

    pub fn prepend_leading_trivia(&mut self, trivia_text: &str) {
        if trivia_text.is_empty() {
            return;
        }
        // `extend_back` here is used because we're still working left-to-right, meaning the leading
        // trivia is built by tacking on the new text until we reach the start of the actual token.
        self.pending_leading_trivia = self.pending_leading_trivia.extend_back(trivia_text);
    }

    pub fn add_trivia(&mut self, trivia_text: &str) {
        if trivia_text.is_empty() {
            return;
        }
        // SAFETY: We only do this while building the tree, meaning we know there can and should
        // only be a single other reference to this token (in whatever list or node contains it),
        // and that it won't be mutated by anything else.
        match unsafe { self.last_token_data.as_mut() } {
            Some(data) => data.append_trailing_trivia(trivia_text),
            None => self.prepend_leading_trivia(trivia_text),
        }
    }

    pub fn push_missing(&mut self) {
        self.children.push(SyntaxElement::Empty);
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.parents.push((kind, self.children.len()));
    }

    pub fn start_node_at(&mut self, kind: SyntaxKind, checkpoint: TreeCheckpoint) {
        assert!(
            checkpoint.parents_idx <= self.parents.len(),
            "Tree checkpoint is no longer valid, referencing parents that no longer exist"
        );
        assert!(
            checkpoint.children_idx <= self.children.len(),
            "Tree checkpoint is no longer valid, referencing children that no longer exist"
        );
        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(
                checkpoint.children_idx >= first_child,
                "Tree checkpoint is not valid while in the process of building another node within it. Finish that node before attempting to rewind"
            );
        }
        self.parents.push((kind, checkpoint.children_idx));
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
            let mut collapsed_count = 0;
            let mut collapsed_index = usize::MAX;
            self.deferred_nodes.sort();
            for node in self.deferred_nodes.drain(..).rev() {
                let start = node.start - first_child;
                let mut end = node.end - first_child;
                if end > collapsed_index {
                    end -= collapsed_count;
                }
                let new_node = SyntaxNode::new(node.kind, self.scratch.drain(start..end)).into();
                self.scratch.insert(start, new_node);
                collapsed_count = node.end - node.start - 1;
                collapsed_index = node.start;
            }
            SyntaxNode::new(kind, self.scratch.drain(..)).into()
        };

        self.children.push(node);
    }

    /// Create a new [DeferredNode] that will collect all children between the two checkpoints and
    /// move them into a new node before the next parent node is finished.
    pub fn wrap_with_node(&mut self, kind: SyntaxKind, start: TreeCheckpoint, end: TreeCheckpoint) {
        assert_eq!(
            start.parents_idx, end.parents_idx,
            "Tree checkpoint parents do not match and cannot be compared"
        );
        self.deferred_nodes.push(DeferredNode {
            kind,
            start: start.children_idx,
            end: end.children_idx,
        });
    }

    pub fn last_element(&self) -> Option<&SyntaxElement> {
        self.children.last()
    }

    /// Returns the current index in the children list as a single number (as opposed to a
    /// checkpoint that includes both a child and a parent index).
    pub fn index(&self) -> u32 {
        self.children.len() as u32
    }

    pub fn checkpoint(&self) -> TreeCheckpoint {
        TreeCheckpoint {
            parents_idx: self.parents.len(),
            children_idx: self.children.len(),
        }
    }

    pub fn rewind(&mut self, checkpoint: TreeCheckpoint) {
        assert!(
            checkpoint.parents_idx <= self.parents.len(),
            "Tree checkpoint is no longer valid, referencing parents that no longer exist"
        );
        assert!(
            checkpoint.children_idx <= self.children.len(),
            "Tree checkpoint is no longer valid, referencing children that no longer exist"
        );
        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(
                checkpoint.children_idx >= first_child,
                "Tree checkpoint is not valid while in the process of building another node within it. Finish that node before attempting to rewind"
            );
        }
        self.parents.truncate(checkpoint.parents_idx);
        self.children.truncate(checkpoint.children_idx);
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
