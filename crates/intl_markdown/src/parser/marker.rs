use crate::syntax::TreeCheckpoint;
use crate::syntax::{SyntaxKind, TextSize};
use crate::ICUMarkdownParser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Marker {
    checkpoint: TreeCheckpoint,
}

impl Marker {
    pub(crate) fn new(checkpoint: TreeCheckpoint) -> Self {
        Self { checkpoint }
    }

    pub(crate) fn get(&self) -> TextSize {
        self.checkpoint.get() as u32
    }

    pub(crate) fn span_to(self, close: Marker) -> MarkerSpan {
        MarkerSpan::from_markers(self, close)
    }

    pub(crate) fn complete(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) -> Option<()> {
        p.start_node_at(kind, self.checkpoint);
        p.finish_node();
        Some(())
    }

    pub(crate) fn complete_missing(self, p: &mut ICUMarkdownParser) -> Option<()> {
        p.builder.push_missing();
        Some(())
    }
}

/// An expanded Marker representing two points, a beginning and an end, that
/// can be completed as a matching pair in a single go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarkerSpan(Marker, Marker);

impl MarkerSpan {
    pub(crate) fn from_markers(open: Marker, close: Marker) -> Self {
        Self(open, close)
    }

    pub(crate) fn new(open_index: TreeCheckpoint, close_index: TreeCheckpoint) -> Self {
        Self(Marker::new(open_index), Marker::new(close_index))
    }

    #[inline(always)]
    pub(crate) fn complete(self, p: &mut ICUMarkdownParser, kind: SyntaxKind) {
        p.wrap_with_node(kind, self.0.checkpoint, self.1.checkpoint);
    }
}
