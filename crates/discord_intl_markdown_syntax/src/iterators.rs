use crate::html_entities::get_html_entity;
use crate::{SyntaxElement, TextPointer, TrimKind};
use crate::{SyntaxNode, SyntaxToken};
use std::iter::Peekable;

pub trait PositionalIterator: Iterator + ExactSizeIterator {
    fn with_positions(self) -> PositionalIter<Self>
    where
        Self: Sized,
    {
        PositionalIter::new(self)
    }
}

impl<'a, T> PositionalIterator for std::slice::Iter<'a, T> {}

/// Iterator for collecting the complete token text contained by a node. Handling of trivia
/// is controlled by [`TokenTextIterOptions`], created by [`SyntaxNodeTokenIter::with_options`].
pub struct SyntaxNodeTokenIter<'a> {
    parents: Vec<(&'a SyntaxNode, usize)>,
}

// Public API
impl<'a> SyntaxNodeTokenIter<'a> {
    pub fn new(node: &'a SyntaxNode) -> Self {
        let mut parents = Vec::with_capacity(4);
        parents.push((node, 0));
        Self { parents }
    }

    pub fn into_text_iter(self) -> TokenTextIter<'a, Self> {
        TokenTextIter::new(self)
    }
}

impl SyntaxNodeTokenIter<'_> {
    pub(self) fn bump_cursor(&mut self) {
        self.parents.last_mut().map(|(_, index)| *index += 1);
    }
}

impl<'a> Iterator for SyntaxNodeTokenIter<'a> {
    type Item = &'a SyntaxToken;

    fn next(&mut self) -> Option<Self::Item> {
        // Depth-first search for the next token contained by the element.
        let token = loop {
            let &(node, index) = self.parents.last()?;
            self.bump_cursor();
            if index >= node.len() {
                self.parents.pop();
                continue;
            }

            match &node[index] {
                SyntaxElement::Token(token) => break token,
                SyntaxElement::Node(node) => self.parents.push((node, 0)),
                SyntaxElement::Empty => {}
            }
        };
        Some(token)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TokenTextIterOptions {
    /// When true, the first encountered token will have its leading trivia trimmed.
    trim_first_leading: bool,
    /// Controls the trimming of leading trivia for all tokens _except_ the first.
    trim_leading: bool,
    /// Controls the trimming of trailing trivia for all tokens _except_ for the last.
    trim_trailing: bool,
    /// When true, the last encountered token will have its leading trivia trimmed.
    trim_last_trailing: bool,
    /// When true, `HTML_ENTITY`, `HEX_CHAR_REF`, and `DEC_CHAR_REF` will have their text replaced
    /// with the referenced value. Otherwise, the entity text will be preserved as-is.
    ///
    /// Because entities can currently still have trivia attached to them, this setting will also
    /// cause the leading and trailing trivia to be emitted separately. When combined with
    /// [`MinimalTextIter`], this will attach those trivia to the adjacent pointers when possible,
    /// but if the token's text was already detached, then they will become two separate pointers.
    replace_entity_references: bool,
}

impl TokenTextIterOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_trim_ends(mut self) -> Self {
        self.trim_first_leading = true;
        self.trim_last_trailing = true;
        self
    }

    pub fn with_trim_first_leading(mut self) -> Self {
        self.trim_first_leading = true;
        self
    }

    pub fn with_trim_leading(mut self) -> Self {
        self.trim_leading = true;
        self
    }

    pub fn with_trim_trailing(mut self) -> Self {
        self.trim_trailing = true;
        self
    }

    pub fn with_trim_last_trailing(mut self) -> Self {
        self.trim_last_trailing = true;
        self
    }

    pub fn with_replace_entity_references(mut self, replace_entity_references: bool) -> Self {
        self.replace_entity_references = replace_entity_references;
        self
    }

    pub fn trim_kind(&self, is_first: bool, is_last: bool) -> TrimKind {
        let trim_leading = if is_first {
            self.trim_first_leading
        } else {
            self.trim_leading
        };
        let trim_trailing = if is_last {
            self.trim_last_trailing
        } else {
            self.trim_trailing
        };

        match (trim_leading, trim_trailing) {
            (true, true) => TrimKind::TrimAll,
            (true, false) => TrimKind::TrimLeading,
            (false, true) => TrimKind::TrimTrailing,
            (false, false) => TrimKind::TrimNone,
        }
    }
}

impl Default for TokenTextIterOptions {
    /// By default, tokens trim all leading trivia and preserve all trailing trivia.
    fn default() -> Self {
        Self {
            trim_first_leading: true,
            trim_leading: true,
            trim_trailing: false,
            trim_last_trailing: false,
            replace_entity_references: true,
        }
    }
}

pub struct TokenTextIter<'a, I: Iterator<Item = &'a SyntaxToken>> {
    inner: Peekable<I>,
    options: TokenTextIterOptions,
    is_first: bool,
}

impl<'a, I: Iterator<Item = &'a SyntaxToken>> TokenTextIter<'a, I> {
    pub fn new(inner: I) -> Self {
        Self {
            inner: inner.peekable(),
            options: Default::default(),
            is_first: true,
        }
    }

    pub fn with_options(mut self, options: TokenTextIterOptions) -> Self {
        self.options = options;
        self
    }

    pub fn minimal(self) -> MinimalTextIter<Self> {
        MinimalTextIter::new(self)
    }
}

impl<'a, I: Iterator<Item = &'a SyntaxToken>> Iterator for TokenTextIter<'a, I> {
    type Item = TextPointer;
    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner.next()?;
        let trim_kind = self
            .options
            .trim_kind(self.is_first, self.inner.peek().is_none());
        // TODO: handle trivia on entity references
        if self.options.replace_entity_references && token.kind().is_entity_reference() {
            return Some(
                get_html_entity(token.text().as_bytes())
                    .map(TextPointer::from_str)
                    .unwrap_or(token.trimmed_text_pointer(trim_kind)),
            );
        }
        let text = token.trimmed_text_pointer(trim_kind);
        self.is_first = false;
        Some(text)
    }
}

/// A wrapping iterator around [`TextPointer`]s that will merge pointers together so long as they
/// are adjacent (according to [`TextPointer`] itself, where the pointers reference the same source
/// text and `b` starts at the position immediately after `a`). Detached and non-adjacent pointers
/// will start a new text pointer, but the iterator will continue trying to merge adjacent pointers
/// using that one as the new base.
pub struct MinimalTextIter<I: Iterator<Item = TextPointer>> {
    inner: Peekable<I>,
}

impl<I: Iterator<Item = TextPointer>> MinimalTextIter<I> {
    pub fn new(iter: I) -> Self {
        Self {
            inner: iter.peekable(),
        }
    }
}

impl<I: Iterator<Item = TextPointer>> Iterator for MinimalTextIter<I> {
    type Item = TextPointer;
    fn next(&mut self) -> Option<Self::Item> {
        let mut pointer = self.inner.next()?;
        loop {
            let Some(next) = self.inner.next_if(|next| pointer.is_adjacent_before(&next)) else {
                break;
            };
            pointer = pointer.extend_back(&next);
        }

        Some(pointer)
    }
}

pub enum FirstLastPosition {
    Neither,
    First,
    Last,
    Both,
}

impl FirstLastPosition {
    pub fn from_cursor_and_size(cursor: usize, size: usize) -> Self {
        match (cursor <= 0, cursor >= size - 1) {
            (true, true) => FirstLastPosition::Both,
            (true, false) => FirstLastPosition::First,
            (false, true) => FirstLastPosition::Last,
            (false, false) => FirstLastPosition::Neither,
        }
    }
    pub fn is_first(&self) -> bool {
        matches!(self, FirstLastPosition::First | FirstLastPosition::Both)
    }

    pub fn is_last(&self) -> bool {
        matches!(self, FirstLastPosition::Last | FirstLastPosition::Both)
    }

    #[allow(unused)]
    pub fn is_middle(&self) -> bool {
        matches!(self, FirstLastPosition::Neither)
    }
}

pub struct PositionalIter<I: Iterator + ExactSizeIterator> {
    inner: I,
    cursor: usize,
    len: usize,
}

impl<I: Iterator + ExactSizeIterator> PositionalIter<I> {
    pub fn new(inner: I) -> Self {
        Self {
            len: inner.len(),
            cursor: 0,
            inner,
        }
    }
}

impl<I: Iterator + ExactSizeIterator> Iterator for PositionalIter<I> {
    type Item = (FirstLastPosition, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;
        self.cursor += 1;
        Some((
            FirstLastPosition::from_cursor_and_size(self.cursor - 1, self.len),
            next,
        ))
    }
}
