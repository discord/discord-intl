use super::SyntaxKind;
use crate::syntax::text::TextPointer;
use std::fmt::{Debug, Formatter};
use std::ops::{Deref, Range};
use std::rc::Rc;

/// An opaque type representing a reference to the source text of the parser.
pub type SourceText = Rc<str>;
pub type TextSize = u32;
pub type TextSpan = Range<usize>;

/// A singular token entity, including both the kind of the token and it's
/// span in the underlying text. The actual text that the token represents is
/// stored as a reference-counted pointer to the original text, allowing tokens
/// to be cheaply cloned and passed around without worry of the lifetime of the
/// underlying string.
#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct SyntaxTokenData {
    /// The kind of token present here.
    kind: SyntaxKind,
    text: TextPointer,
    /// The start position of the actual token text
    text_start: TextSize,
    /// The start position of the trailing trivia attached to the token
    trailing_start: TextSize,
}

impl SyntaxTokenData {
    // NOTE: Internal-only methods for efficiently constructing the tree with trivia that may only
    // be added after a token has been pushed elsewhere into the tree structure.
    // See [TreeBuilder::append_token_trivia] for context on the usage.

    pub(super) fn append_trailing_trivia(&mut self, trivia_text: &str) {
        self.text = self.text.extend_back(trivia_text);
    }

    pub(super) fn prepend_leading_trivia(&mut self, trivia_text: &str) {
        self.text = self.text.extend_front(trivia_text);
        self.text_start = self.text_start + trivia_text.len() as TextSize;
        self.trailing_start = self.trailing_start + trivia_text.len() as TextSize;
    }
}

#[derive(Clone, Default, Eq, PartialEq, Hash)]
pub struct SyntaxToken(Rc<SyntaxTokenData>);

impl SyntaxToken {
    pub fn new(kind: SyntaxKind, text: TextPointer) -> Self {
        let len = text.len_size();
        Self(Rc::new(SyntaxTokenData {
            kind,
            text,
            text_start: 0,
            trailing_start: len,
        }))
    }

    pub fn from_str(kind: SyntaxKind, text: &str) -> Self {
        Self(Rc::new(SyntaxTokenData {
            kind,
            text: TextPointer::from_str(text),
            text_start: TextSize::default(),
            trailing_start: text.len() as TextSize,
        }))
    }

    /// Returns the kind of this token.
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    pub fn has_trivia(&self) -> bool {
        self.trailing_start != self.text.len_size()
    }

    /// Returns the positional range this token represents in the source.
    ///
    /// This method does _not_ include the range of any trivia attached to this token.
    pub fn text_span(&self) -> TextSpan {
        self.text_start as usize..self.trailing_start as usize
    }

    /// Returns the position range of _only_ the trivia attached to this token.
    pub fn trailing_trivia_span(&self) -> TextSpan {
        self.trailing_start as usize..self.text.len()
    }

    /// Return the entire positional range this token represents in the source, including the length
    /// of the trivia attached to the token.
    pub fn span(&self) -> TextSpan {
        self.text.range()
    }

    pub fn start(&self) -> TextSize {
        self.text.start()
    }

    pub fn end(&self) -> TextSize {
        self.text.end()
    }

    /// Returns the starting character position of this token in the source.
    pub fn text_start(&self) -> TextSize {
        self.text_start
    }

    /// Returns the ending character position of this token in the source.
    pub fn text_end(&self) -> TextSize {
        self.trailing_start
    }

    pub fn trivia_start(&self) -> TextSize {
        self.trailing_start
    }

    pub fn trivia_end(&self) -> TextSize {
        self.text.end()
    }

    pub fn len(&self) -> TextSize {
        self.text.len_size()
    }

    pub fn text_len(&self) -> TextSize {
        self.trailing_start - self.text_start
    }

    pub fn trailing_trivia_len(&self) -> TextSize {
        self.end() - self.trivia_start()
    }

    /// Returns the text of this token excluding all attached trivia.
    pub fn text(&self) -> &str {
        &self.text[self.text_span()]
    }

    /// Returns only the text of the trivia attached to this token.
    pub fn trailing_trivia_text(&self) -> &str {
        &self.text[self.trailing_trivia_span()]
    }

    /// Returns the complete text of this token, including the token itself and any attached trivia.
    pub fn text_with_trivia(&self) -> &str {
        &self.text
    }

    // NOTE: Internal-only methods for efficiently constructing the tree with trivia that may only
    // be added after a token has been pushed elsewhere into the tree structure.
    // See [TreeBuilder::append_token_trivia] for context on the usage.

    pub(super) fn raw_data(&self) -> Rc<SyntaxTokenData> {
        self.0.clone()
    }
}

impl Deref for SyntaxToken {
    type Target = SyntaxTokenData;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl Debug for SyntaxToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:?}@{}..{}{:?}",
            self.kind(),
            self.text_start(),
            self.text_end(),
            self.text()
        ))?;

        if self.has_trivia() {
            f.write_fmt(format_args!(
                " [..{} {:?}]",
                self.trivia_end(),
                self.trailing_trivia_text()
            ))?;
        }
        Ok(())
    }
}
