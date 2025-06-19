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
        if !trivia_text.is_empty() {
            self.text = self.text.extend_back(trivia_text);
        }
    }

    pub(super) fn prepend_leading_trivia(&mut self, trivia_text: &str) {
        if !trivia_text.is_empty() {
            self.text = self.text.extend_front(trivia_text);
            self.text_start += trivia_text.len() as TextSize;
            self.trailing_start += trivia_text.len() as TextSize;
        }
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
        self.text_start > 0 || self.trailing_start != self.text.len_size()
    }

    /// Returns the positional range this token represents in the source.
    ///
    /// This method does _not_ include the range of any trivia attached to this token.
    pub fn text_span(&self) -> TextSpan {
        self.text_start as usize..self.trailing_start as usize
    }

    /// Returns the position range of _only_ the leading trivia attached to this token.
    pub fn leading_trivia_span(&self) -> TextSpan {
        0..self.text_start as usize
    }

    /// Returns the position range of _only_ the trailing trivia attached to this token.
    pub fn trailing_trivia_span(&self) -> TextSpan {
        self.trailing_start as usize..self.text.len()
    }

    /// Return the entire positional range this token represents in the source, including the length
    /// of the trivia attached to the token.
    pub fn span(&self) -> TextSpan {
        self.text.range()
    }

    /// Returns the starting character position of this token's main text.
    pub fn text_start(&self) -> TextSize {
        self.text_start
    }

    /// Returns the ending character position of this token's main text.
    pub fn text_end(&self) -> TextSize {
        self.trailing_start
    }
    /// Returns the starting character position of this token's leading trivia.
    pub fn leading_trivia_start(&self) -> TextSize {
        0
    }

    /// Returns the ending character position of this token's leading trivia.
    pub fn leading_trivia_end(&self) -> TextSize {
        self.text_start
    }

    /// Returns the starting character position of this token's trailing trivia.
    pub fn trailing_trivia_start(&self) -> TextSize {
        self.trailing_start
    }

    /// Returns the ending character position of this token's trailing trivia.
    pub fn trailing_trivia_end(&self) -> TextSize {
        self.text.len_size()
    }

    /// Returns the total length of this token, including trivia.
    pub fn len(&self) -> TextSize {
        self.text.len_size()
    }

    /// Returns the length of just this token's main text.
    pub fn text_len(&self) -> TextSize {
        self.trailing_start - self.text_start
    }

    /// Returns the length of just this token's leading trivia.
    pub fn leading_trivia_len(&self) -> TextSize {
        self.text_start
    }

    /// Returns the length of just this token's trailing trivia.
    pub fn trailing_trivia_len(&self) -> TextSize {
        self.text.len() as u32 - self.trailing_start
    }

    pub fn text_pointer(&self) -> &TextPointer {
        &self.text
    }

    /// Returns a new TextPointer to the text of this token _excluding_ its leading trivia.
    // TODO: Rename/change this to be more normal
    pub fn text_pointer_with_no_leading_trivia(&self) -> TextPointer {
        self.text.clone().trim_front(self.leading_trivia_len())
    }

    /// Returns the text of this token excluding all attached trivia.
    pub fn text(&self) -> &str {
        &self.text[self.text_span()]
    }

    /// Returns only the text of the trailing trivia attached to this token.
    pub fn leading_trivia_text(&self) -> &str {
        &self.text[self.leading_trivia_span()]
    }

    /// Returns only the text of the leading trivia attached to this token.
    pub fn trailing_trivia_text(&self) -> &str {
        &self.text[self.trailing_trivia_span()]
    }

    /// Returns the complete text of this token, including the token itself and any attached trivia.
    pub fn text_with_trailing_trivia(&self) -> &str {
        &self.text[self.text_start() as usize..self.trailing_trivia_end() as usize]
    }

    pub fn text_with_leading_trivia(&self) -> &str {
        &self.text[self.leading_trivia_start() as usize..self.text_end() as usize]
    }

    pub fn full_text(&self) -> &str {
        &self.text
    }
}

impl SyntaxToken {
    // NOTE: Internal-only methods for efficiently constructing the tree with trivia that may only
    // be added after a token has been pushed elsewhere into the tree structure.
    // See [TreeBuilder::add_trivia] for context on the usage.

    pub(super) fn raw_data(&self) -> Rc<SyntaxTokenData> {
        self.0.clone()
    }

    /// Creates a clone of this token by fully copying the underlying syntax data. The result is a
    /// fully detached token that can be safely manipulated without any effect on other token
    /// instances that may be referencing the same data.
    pub(super) fn deep_clone(&self) -> SyntaxToken {
        let data = (*self.0).clone();
        SyntaxToken(Rc::from(data))
    }

    /// Append the other token to the back of this token. Trailing and leading trivia become part
    /// of the token's actual text. This method should _only_ be used when intentionally merging
    /// tokens while building the initial tree. All downstream usages should prefer creating new
    /// tokens instead.
    ///
    /// ## Safety
    ///
    /// This method should _only_ be used on a newly created token, to ensure that no other tokens
    /// reference the same data and end up incorrectly affected by the change.
    pub(super) unsafe fn extend_back(&mut self, other: SyntaxToken) {
        let new_trailing_start = self.len() + other.trailing_start;
        let new_text_pointer = self.text.extend_back(other.full_text());
        let ptr = self.raw_ptr();
        ptr.text = new_text_pointer;
        ptr.trailing_start = new_trailing_start;
    }

    /// Intentionally override the `kind` of this token to be something else. Generally, kinds
    /// should only be determined by the lexer and parser. During tree building, though, there may
    /// be cases where the tree decides it's more efficient to represent a token differently, or to
    /// be able to merge insignificant text pieces.
    pub(super) fn set_kind(&mut self, kind: SyntaxKind) {
        self.raw_ptr().kind = kind;
    }

    fn raw_ptr(&mut self) -> &mut SyntaxTokenData {
        let ptr = Rc::as_ptr(&self.0) as *mut SyntaxTokenData;
        unsafe { &mut *ptr }
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
            "{:?}@{}{:?}",
            self.kind(),
            self.text.format_range(),
            self.text()
        ))?;

        if self.has_trivia() {
            f.write_fmt(format_args!(
                " [{:?}, {:?}]",
                self.leading_trivia_text(),
                self.trailing_trivia_text()
            ))?;
        }
        Ok(())
    }
}
