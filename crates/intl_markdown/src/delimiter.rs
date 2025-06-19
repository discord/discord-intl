use super::syntax::{SyntaxKind, TreeMarker};

pub(crate) trait Delimiter {
    fn syntax_kind(&self) -> SyntaxKind;
    fn count(&self) -> usize;

    fn is_active(&self) -> bool;
    fn deactivate(&mut self);

    fn can_open(&self) -> bool;
    fn can_close(&self) -> bool;

    /// Returns a tuple of two event indices where the first represents the
    /// event to use as the opening marker for the semantic item, and the
    /// second represents the event to use as the opening marker for the
    /// content _within_ the item.
    fn consume_opening(&mut self, count: usize) -> (TreeMarker, TreeMarker);
    /// Returns a tuple of two event indices where the first represents the
    /// event to use as the closing marker for the semantic item, and the
    /// second represents the event to use as the closing marker for the
    /// content _within_ the item.
    fn consume_closing(&mut self, count: usize) -> (TreeMarker, TreeMarker);

    fn can_open_and_close(&self) -> bool {
        self.can_open() && self.can_close()
    }
}

/// Emphasis delimiters represent a run of tokens that can each be used to
/// possibly start or end some form of emphasis (e.g., strong or regular).
///
/// The implementation of this struct uses the assumption that all delimiters
/// are adjacent tokens in the parser's token list to select the appropriate
/// indices for each delimiter to use for its `first_token` and `length`.
#[derive(Debug)]
pub struct EmphasisDelimiter {
    kind: SyntaxKind,
    count: usize,
    end_offset: usize,
    start_offset: usize,
    can_open: bool,
    can_close: bool,
    active: bool,
    /// Emphasis delimiter runs are always a flat list of tokens, so the cursor can be tracked as a
    /// single marker where we move the cursor itself forward when consuming _closing_ delimiters,
    /// and use the cursor + remaining count for openers.
    cursor: TreeMarker,
}

impl EmphasisDelimiter {
    pub fn new(
        kind: SyntaxKind,
        count: usize,
        can_open: bool,
        can_close: bool,
        cursor: TreeMarker,
    ) -> Self {
        Self {
            kind,
            count,
            can_open,
            can_close,
            active: true,
            cursor,
            end_offset: 0,
            start_offset: 0,
        }
    }
}

impl Delimiter for EmphasisDelimiter {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind
    }

    fn count(&self) -> usize {
        self.count - self.end_offset - self.start_offset
    }

    fn is_active(&self) -> bool {
        self.active && self.count() > 0
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn can_open(&self) -> bool {
        self.can_open
    }

    fn can_close(&self) -> bool {
        self.can_close
    }

    fn consume_opening(&mut self, count: usize) -> (TreeMarker, TreeMarker) {
        //  | * | * | * | * |
        //  |       |       |-- content open = cursor + self.count - end_offset
        //  |       | -- item open = cursor + self.count - end_offset - count
        //  |-- cursor
        let item_open = self
            .cursor
            .clone()
            .add_child_offset(self.count - self.end_offset - count);
        let content_open = self
            .cursor
            .clone()
            .add_child_offset(self.count - self.end_offset);
        self.end_offset += count;
        (item_open, content_open)
    }

    fn consume_closing(&mut self, count: usize) -> (TreeMarker, TreeMarker) {
        // | * | * | * | * |
        // |       |-- item close = cursor + start_offset + count
        // |-- content close = cursor + start_offset
        // An additional 1 has to be removed
        let content_close = self.cursor.clone().add_token_offset(self.start_offset);
        let item_close = self
            .cursor
            .clone()
            .add_token_offset(self.start_offset + count);
        self.start_offset += count;
        (item_close, content_close)
    }
}

#[derive(Debug)]
pub struct LinkDelimiter {
    kind: SyntaxKind,
    is_closing: bool,
    active: bool,
    consumed: bool,
    /// Cursor to the marker for the link as a whole, including the resource.
    link_cursor: TreeMarker,
    /// Cursor to the marker for the link content, within the square brackets.
    content_cursor: TreeMarker,
}

impl LinkDelimiter {
    pub fn new(
        kind: SyntaxKind,
        is_closing: bool,
        link_marker: TreeMarker,
        content_marker: TreeMarker,
    ) -> Self {
        Self {
            kind,
            is_closing,
            active: true,
            consumed: false,
            link_cursor: link_marker,
            content_cursor: content_marker,
        }
    }

    pub fn link_cursor(&self) -> &TreeMarker {
        &self.link_cursor
    }

    pub fn content_cursor(&self) -> &TreeMarker {
        &self.content_cursor
    }
}

impl Delimiter for LinkDelimiter {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind
    }

    fn count(&self) -> usize {
        if self.consumed {
            0
        } else {
            1
        }
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn can_open(&self) -> bool {
        !self.is_closing
    }

    fn can_close(&self) -> bool {
        self.is_closing
    }

    fn consume_opening(&mut self, _count: usize) -> (TreeMarker, TreeMarker) {
        self.consumed = true;
        // These values aren't used for link delimiters
        Default::default()
    }

    fn consume_closing(&mut self, _count: usize) -> (TreeMarker, TreeMarker) {
        self.consumed = true;
        // These values aren't used for link delimiters
        Default::default()
    }
}

#[derive(Debug)]
pub struct StrikethroughDelimiter {
    kind: SyntaxKind,
    count: usize,
    can_open: bool,
    can_close: bool,
    active: bool,
    cursor: TreeMarker,
}

impl StrikethroughDelimiter {
    pub fn new(
        kind: SyntaxKind,
        count: usize,
        can_open: bool,
        can_close: bool,
        cursor: TreeMarker,
    ) -> Self {
        Self {
            kind,
            count,
            can_open,
            can_close,
            active: true,
            cursor,
        }
    }
}

impl Delimiter for StrikethroughDelimiter {
    fn syntax_kind(&self) -> SyntaxKind {
        self.kind
    }

    fn count(&self) -> usize {
        self.count
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn deactivate(&mut self) {
        self.active = false;
    }

    fn can_open(&self) -> bool {
        self.can_open
    }

    fn can_close(&self) -> bool {
        self.can_close
    }

    fn consume_opening(&mut self, count: usize) -> (TreeMarker, TreeMarker) {
        self.active = false;
        let content_open = self.cursor.clone().add_token_offset(self.count + 1);
        self.count -= count;
        let item_open = self.cursor.clone().add_token_offset(self.count);
        (item_open, content_open)
    }

    fn consume_closing(&mut self, count: usize) -> (TreeMarker, TreeMarker) {
        self.active = false;
        let content_close = self.cursor.clone();
        self.count -= count;
        let item_close = self.cursor.clone().add_token_offset(count + 1);
        (item_close, content_close)
    }
}

#[derive(Debug)]
pub enum AnyDelimiter {
    Emphasis(EmphasisDelimiter),
    Link(LinkDelimiter),
    Strikethrough(StrikethroughDelimiter),
}

impl AnyDelimiter {
    pub fn is_strikethrough(&self) -> bool {
        matches!(self, AnyDelimiter::Strikethrough(_))
    }

    pub fn as_link_delimiter(&self) -> Option<&LinkDelimiter> {
        match self {
            AnyDelimiter::Link(link) => Some(link),
            _ => None,
        }
    }
    pub fn as_emphasis_delimiter(&self) -> Option<&EmphasisDelimiter> {
        match self {
            AnyDelimiter::Emphasis(emphasis) => Some(emphasis),
            _ => None,
        }
    }
    pub fn as_strikethrough_delimiter(&self) -> Option<&StrikethroughDelimiter> {
        match self {
            AnyDelimiter::Strikethrough(strikethrough) => Some(strikethrough),
            _ => None,
        }
    }

    pub fn as_link_delimiter_mut(&mut self) -> Option<&mut LinkDelimiter> {
        match self {
            AnyDelimiter::Link(link) => Some(link),
            _ => None,
        }
    }
    pub fn as_emphasis_delimiter_mut(&mut self) -> Option<&mut EmphasisDelimiter> {
        match self {
            AnyDelimiter::Emphasis(emphasis) => Some(emphasis),
            _ => None,
        }
    }
    pub fn as_strikethrough_delimiter_mut(&mut self) -> Option<&mut StrikethroughDelimiter> {
        match self {
            AnyDelimiter::Strikethrough(strikethrough) => Some(strikethrough),
            _ => None,
        }
    }
}

impl Delimiter for AnyDelimiter {
    fn syntax_kind(&self) -> SyntaxKind {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.syntax_kind(),
            AnyDelimiter::Link(link) => link.syntax_kind(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.syntax_kind(),
        }
    }

    fn count(&self) -> usize {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.count(),
            AnyDelimiter::Link(link) => link.count(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.count(),
        }
    }

    fn is_active(&self) -> bool {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.is_active(),
            AnyDelimiter::Link(link) => link.is_active(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.is_active(),
        }
    }

    fn deactivate(&mut self) {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.deactivate(),
            AnyDelimiter::Link(link) => link.deactivate(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.deactivate(),
        }
    }

    fn can_open(&self) -> bool {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.can_open(),
            AnyDelimiter::Link(link) => link.can_open(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.can_open(),
        }
    }

    fn can_close(&self) -> bool {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.can_close(),
            AnyDelimiter::Link(link) => link.can_close(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.can_close(),
        }
    }

    /// Returns a tuple two event indices where the first represents the event
    /// event to use as the opening marker for the semantic item, and the
    /// second represents the event to use as the opening marker for the
    /// content _within_ the item.
    fn consume_opening(&mut self, count: usize) -> (TreeMarker, TreeMarker) {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.consume_opening(count),
            AnyDelimiter::Link(link) => link.consume_opening(count),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.consume_opening(count),
        }
    }

    /// Returns a tuple two event indices where the first represents the event
    /// event to use as the closing marker for the semantic item, and the
    /// second represents the event to use as the closing marker for the
    /// content _within_ the item.
    fn consume_closing(&mut self, count: usize) -> (TreeMarker, TreeMarker) {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.consume_closing(count),
            AnyDelimiter::Link(link) => link.consume_closing(count),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.consume_closing(count),
        }
    }
}

impl From<LinkDelimiter> for AnyDelimiter {
    fn from(value: LinkDelimiter) -> Self {
        AnyDelimiter::Link(value)
    }
}

impl From<EmphasisDelimiter> for AnyDelimiter {
    fn from(value: EmphasisDelimiter) -> Self {
        AnyDelimiter::Emphasis(value)
    }
}

impl From<StrikethroughDelimiter> for AnyDelimiter {
    fn from(value: StrikethroughDelimiter) -> Self {
        AnyDelimiter::Strikethrough(value)
    }
}
