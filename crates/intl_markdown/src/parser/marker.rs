use crate::ICUMarkdownParser;
use intl_markdown_syntax::{SyntaxKind, TreeMarker};
use std::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct Marker(TreeMarker);

impl Marker {
    pub(crate) fn new(tree_marker: TreeMarker) -> Self {
        Self(tree_marker)
    }

    pub(crate) fn span_to(self, close: Marker) -> MarkerSpan {
        MarkerSpan::from_markers(self, close)
    }

    pub(crate) fn complete(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
        p.start_node_at(kind, self.0);
        p.finish_node();
        Some(())
    }
}

impl From<Marker> for TreeMarker {
    fn from(value: Marker) -> Self {
        value.0
    }
}

impl Deref for Marker {
    type Target = TreeMarker;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// An expanded Marker representing two points, a beginning and an end, that
/// can be completed as a matching pair in a single go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkerSpan(TreeMarker, TreeMarker);

impl MarkerSpan {
    pub(crate) fn from_markers(open: Marker, close: Marker) -> Self {
        Self(open.0, close.0)
    }

    pub(crate) fn new(open_index: TreeMarker, close_index: TreeMarker) -> Self {
        Self(open_index, close_index)
    }

    #[inline(always)]
    pub(crate) fn complete(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) {
        p.wrap_with_node(kind, self.0, self.1);
    }
}
