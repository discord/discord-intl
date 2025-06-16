use super::syntax::{SyntaxKind, TextSize};

pub(crate) trait Delimiter {
    fn kind(&self) -> SyntaxKind;
    fn count(&self) -> usize;

    fn is_active(&self) -> bool;
    fn deactivate(&mut self);

    fn can_open(&self) -> bool;
    fn can_close(&self) -> bool;

    fn opening_cursor(&self) -> TextSize;
    fn closing_cursor(&self) -> TextSize;

    /// Returns a tuple of two event indices where the first represents the
    /// event to use as the opening marker for the semantic item, and the
    /// second represents the event to use as the opening marker for the
    /// content _within_ the item.
    fn consume_opening(&mut self, count: usize) -> (TextSize, TextSize);
    /// Returns a tuple of two event indices where the first represents the
    /// event to use as the closing marker for the semantic item, and the
    /// second represents the event to use as the closing marker for the
    /// content _within_ the item.
    fn consume_closing(&mut self, count: usize) -> (TextSize, TextSize);

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
    can_open: bool,
    can_close: bool,
    active: bool,
    start_cursor: TextSize,
    end_cursor: TextSize,
}

impl EmphasisDelimiter {
    pub fn new(
        kind: SyntaxKind,
        count: usize,
        can_open: bool,
        can_close: bool,
        first_index: TextSize,
    ) -> Self {
        Self {
            kind,
            count,
            can_open,
            can_close,
            active: true,
            start_cursor: first_index,
            end_cursor: first_index + count as u32,
        }
    }
}

impl Delimiter for EmphasisDelimiter {
    fn kind(&self) -> SyntaxKind {
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

    fn opening_cursor(&self) -> TextSize {
        self.start_cursor
    }

    fn closing_cursor(&self) -> TextSize {
        self.end_cursor
    }

    fn consume_opening(&mut self, count: usize) -> (TextSize, TextSize) {
        self.count -= count;
        self.end_cursor -= count as TextSize;
        self.active = self.count > 0;
        (self.end_cursor, self.end_cursor + count as TextSize)
    }

    fn consume_closing(&mut self, count: usize) -> (TextSize, TextSize) {
        self.count -= count;
        self.start_cursor += count as TextSize;
        self.active = self.count > 0;
        (self.start_cursor, self.start_cursor - count as TextSize)
    }
}

#[derive(Debug)]
pub struct LinkDelimiter {
    kind: SyntaxKind,
    is_closing: bool,
    active: bool,
    consumed: bool,
    /// Cursor to the marker for the link as a whole, including the resource.
    link_cursor: TextSize,
    /// Cursor to the marker for the link content, within the square brackets.
    content_cursor: TextSize,
}

impl LinkDelimiter {
    pub fn new(
        kind: SyntaxKind,
        is_closing: bool,
        link_index: TextSize,
        content_index: TextSize,
    ) -> Self {
        Self {
            kind,
            is_closing,
            active: true,
            consumed: false,
            link_cursor: link_index,
            content_cursor: content_index,
        }
    }
}

impl Delimiter for LinkDelimiter {
    fn kind(&self) -> SyntaxKind {
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

    fn opening_cursor(&self) -> TextSize {
        self.link_cursor
    }

    fn closing_cursor(&self) -> TextSize {
        self.content_cursor
    }

    fn consume_opening(&mut self, _count: usize) -> (TextSize, TextSize) {
        self.consumed = true;
        // These values aren't used for link delimiters
        Default::default()
    }

    fn consume_closing(&mut self, _count: usize) -> (TextSize, TextSize) {
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
    start_cursor: TextSize,
    end_cursor: TextSize,
}

impl StrikethroughDelimiter {
    pub fn new(
        kind: SyntaxKind,
        count: usize,
        can_open: bool,
        can_close: bool,
        open_index: TextSize,
    ) -> Self {
        Self {
            kind,
            count,
            can_open,
            can_close,
            active: true,
            start_cursor: open_index,
            end_cursor: open_index + (count as TextSize) + 1,
        }
    }
}

impl Delimiter for StrikethroughDelimiter {
    fn kind(&self) -> SyntaxKind {
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

    fn opening_cursor(&self) -> TextSize {
        self.start_cursor
    }

    fn closing_cursor(&self) -> TextSize {
        self.end_cursor
    }

    fn consume_opening(&mut self, _count: usize) -> (TextSize, TextSize) {
        self.active = false;
        self.count = 0;
        // These values aren't used for strikethrough delimiters
        (self.start_cursor, self.end_cursor)
    }

    fn consume_closing(&mut self, _count: usize) -> (TextSize, TextSize) {
        self.active = false;
        self.count = 0;
        (self.end_cursor, self.start_cursor)
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
}

impl Delimiter for AnyDelimiter {
    fn kind(&self) -> SyntaxKind {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.kind(),
            AnyDelimiter::Link(link) => link.kind(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.kind(),
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

    fn opening_cursor(&self) -> TextSize {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.opening_cursor(),
            AnyDelimiter::Link(link) => link.opening_cursor(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.opening_cursor(),
        }
    }

    fn closing_cursor(&self) -> TextSize {
        match self {
            AnyDelimiter::Emphasis(emph) => emph.closing_cursor(),
            AnyDelimiter::Link(link) => link.closing_cursor(),
            AnyDelimiter::Strikethrough(strikethrough) => strikethrough.closing_cursor(),
        }
    }

    /// Returns a tuple two event indices where the first represents the event
    /// event to use as the opening marker for the semantic item, and the
    /// second represents the event to use as the opening marker for the
    /// content _within_ the item.
    fn consume_opening(&mut self, count: usize) -> (TextSize, TextSize) {
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
    fn consume_closing(&mut self, count: usize) -> (TextSize, TextSize) {
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
