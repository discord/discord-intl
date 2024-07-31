use std::fmt::Formatter;
use std::ops::{Deref, DerefMut, Range};
use std::rc::Rc;

use arcstr::{ArcStr, Substr};
use bitflags::bitflags;

use crate::tree_builder::TokenSpan;

use super::syntax::SyntaxKind;

bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TokenFlags: u8 {
        const HAS_PRECEDING_PUNCTUATION = 1;
        const HAS_FOLLOWING_PUNCTUATION = 1 << 1;
        const HAS_PRECEDING_WHITESPACE = 1 << 2;
        const HAS_FOLLOWING_WHITESPACE = 1 << 3;

        // Only used for some delimiters currently. `ESCAPED` kinds will also
        // always have this set.
        const IS_ESCAPED = 1 << 6;
    }
}

impl TokenFlags {
    pub fn has_preceding_punctuation(&self) -> bool {
        self.contains(TokenFlags::HAS_PRECEDING_PUNCTUATION)
    }
    pub fn has_preceding_whitespace(&self) -> bool {
        self.contains(TokenFlags::HAS_PRECEDING_WHITESPACE)
    }
    pub fn has_following_punctuation(&self) -> bool {
        self.contains(TokenFlags::HAS_FOLLOWING_PUNCTUATION)
    }
    pub fn has_following_whitespace(&self) -> bool {
        self.contains(TokenFlags::HAS_FOLLOWING_WHITESPACE)
    }

    pub fn is_escaped(&self) -> bool {
        self.contains(TokenFlags::IS_ESCAPED)
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Trivia {
    kind: SyntaxKind,
    text: Substr,
    /// A Trivia piece is `trailing` on the token in all cases _except_ for
    /// leading whitespace, which is the first whitespace on a line that also
    /// contains non-trivial tokens. This is reverse from how trivia is often
    /// treated (trailing until the next newline, everything else is leading),
    /// but provides a lot more flexibility in the context of Markdown
    /// rendering, where the newlines appearing _after_ a token are significant
    /// and preserved, but depend on whether that token is the last child of
    /// its parent (e.g., hard line breaks can't appear at the end of a
    /// paragraph, and blank lines after a paragraph are ignored).
    is_trailing: bool,
}

impl Trivia {
    pub fn new(kind: SyntaxKind, text: Substr, is_trailing: bool) -> Self {
        Self {
            kind,
            text,
            is_trailing,
        }
    }

    /// Returns the kind of this trivia.
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the text of this trivia.
    pub fn text(&self) -> &str {
        &self.text.as_str()
    }

    /// Returns true if this trivia is leading.
    pub fn is_trailing(&self) -> bool {
        self.is_trailing
    }

    pub fn span_start(&self) -> usize {
        self.text.range().start
    }

    pub fn span_end(&self) -> usize {
        self.text.range().end
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }
}

impl std::fmt::Debug for Trivia {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{:?}@{}..{}\"{}\"",
            self.kind,
            self.text.range().start,
            self.text.range().end,
            self.text.escape_debug(),
        ))?;

        if f.alternate() && !self.is_trailing {
            f.write_str(" leading")?;
        }

        Ok(())
    }
}

/// A representation of a list of Trivia, allowing for a consistent interface to
/// push and pull trivia from the list with the appropriate semantics applied.
#[derive(Clone, Default, Debug)]
pub struct TriviaList {
    trivia_list: Vec<Trivia>,
}

impl TriviaList {
    pub fn new() -> Self {
        Self {
            trivia_list: vec![],
        }
    }

    /// Returns a cursor position, a leading count, and a trailing count for trivia relating to the
    /// given token.
    pub fn get_pointer_data(
        &self,
        token: &SyntaxToken,
        starting_cursor: &mut usize,
    ) -> (u32, u16, u16) {
        let available_trivia = &self.trivia_list[*starting_cursor..];
        let token_start = token.span_start();
        let leading_count = available_trivia
            .iter()
            .position(|trivia| trivia.is_trailing() || trivia.span_start() >= token_start)
            .unwrap_or(available_trivia.len());

        *starting_cursor += leading_count;
        let cursor = *starting_cursor;

        let available_trivia = &self.trivia_list[*starting_cursor..];
        let mut text_position = token.span_end();
        let trailing_count = available_trivia
            .iter()
            .position(|trivia| {
                if !trivia.is_trailing() || trivia.span_start() != text_position {
                    return true;
                }
                text_position += trivia.len();
                false
            })
            .unwrap_or(available_trivia.len());

        *starting_cursor += trailing_count;
        (cursor as u32, leading_count as u16, trailing_count as u16)
    }

    /// Returns all trivia, both leading and trailing, that appear contiguously before the given
    /// position. This method assumes that `cursor` is on a token boundary where trivia could exist.
    /// It will not search within trivia positions.
    fn preceding_contiguous_trivia(&self, mut text_position: usize) -> &[Trivia] {
        let end_cursor = 1 + match self
            .trivia_list
            .binary_search_by_key(&text_position, |trivia| trivia.span_end())
        {
            Ok(index) => index,
            Err(_) => return &[],
        };

        let mut start_cursor = end_cursor;
        for trivia in self.trivia_list[0..end_cursor].iter().rev() {
            if trivia.span_end() == text_position {
                text_position = trivia.span_start();
                start_cursor -= 1;
            } else {
                break;
            }
        }

        &self.trivia_list[start_cursor..end_cursor]
    }

    /// Returns all trivia, both trailing and leading, that appear contiguously after the given
    /// position. This method assumes that `cursor` is on a token boundary where trivia could exist.
    /// It will not search within trivia positions.
    fn following_contiguous_trivia(&self, mut text_position: usize) -> &[Trivia] {
        let start_cursor = match self
            .trivia_list
            .binary_search_by_key(&text_position, |trivia| trivia.span_start())
        {
            Ok(index) => index,
            Err(_) => return &[],
        };

        let mut end_cursor = start_cursor;
        for trivia in &self.trivia_list[start_cursor..] {
            if trivia.span_start() == text_position {
                text_position = trivia.span_end();
                end_cursor += 1;
            } else {
                break;
            }
        }

        &self.trivia_list[start_cursor..=end_cursor]
    }
}
impl Deref for TriviaList {
    type Target = Vec<Trivia>;

    fn deref(&self) -> &Self::Target {
        &self.trivia_list
    }
}
impl DerefMut for TriviaList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.trivia_list
    }
}

#[derive(Clone)]
pub struct TriviaPointer {
    list: Rc<TriviaList>,
    cursor: u32,
    leading_count: u16,
    trailing_count: u16,
}

impl std::fmt::Debug for TriviaPointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "TriviaPointer({}..{}..{}, {:?}, {:?})",
            self.leading_count,
            self.cursor,
            self.trailing_count,
            self.leading_trivia(),
            self.trailing_trivia()
        ))
    }
}

impl TriviaPointer {
    pub fn from_token(
        syntax_token: &SyntaxToken,
        trivia_list: &Rc<TriviaList>,
        trivia_cursor: &mut usize,
    ) -> Self {
        let (cursor, leading_count, trailing_count) =
            trivia_list.get_pointer_data(syntax_token, trivia_cursor);
        Self {
            list: Rc::clone(trivia_list),
            cursor,
            leading_count,
            trailing_count,
        }
    }

    pub fn leading_trivia(&self) -> &[Trivia] {
        let end = self.cursor as usize;
        let start = end - self.leading_count as usize;
        &self.list[start..end]
    }

    pub fn trailing_trivia(&self) -> &[Trivia] {
        let start = self.cursor as usize;
        let end = start + self.trailing_count as usize;
        &self.list[start..end]
    }
}

/// A singular token entity, including both the kind of the token and it's
/// span in the underlying text. The actual text that the token represents is
/// stored as a reference-counted pointer to the original text, allowing tokens
/// to be cheaply cloned and passed around without worry of the lifetime of the
/// underlying string.
///
/// Syntax tokens are the "raw" tokens that come from the input source, and are
/// only used while parsing to create the event stream. After parsing, when
/// building the tree from the events, SyntaxTokens are converted to regular
/// Tokens, which handle mapping Trivia onto the tokens as well.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxToken {
    /// The kind of token present here.
    kind: SyntaxKind,
    /// The text contained by this token
    range: Range<usize>,
    flags: TokenFlags,
}

impl SyntaxToken {
    pub fn new(kind: SyntaxKind, range: Range<usize>) -> Self {
        Self {
            kind,
            range,
            flags: TokenFlags::default(),
        }
    }

    pub fn with_flags(mut self, flags: TokenFlags) -> Self {
        self.flags = flags;
        self
    }

    pub fn with_kind(mut self, kind: SyntaxKind) -> Self {
        self.kind = kind;
        self
    }

    /// Merges the range from the earliest start to the latest end of the two
    /// tokens by consuming `self` and `other` and creating a new token that
    /// spans the total range of the two.
    ///
    /// This method does not check the kinds of the tokens or any other
    /// information before performing the merge. The result will _always_ be a
    /// plain TEXT token.
    pub fn from_bounds(first: SyntaxToken, second: SyntaxToken) -> Self {
        let start = std::cmp::min(first.span_start(), second.span_start());
        let end = std::cmp::max(first.span_end(), second.span_end());

        SyntaxToken::new(SyntaxKind::TEXT, start..end)
    }

    /// Returns the kind of this token.
    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    /// Returns the flags for this token.
    pub fn flags(&self) -> TokenFlags {
        self.flags
    }

    /// Returns the positional range this token represents in the source.
    pub fn span(&self) -> Range<usize> {
        self.range.clone()
    }

    /// Returns the starting character position of this token in the source.
    pub fn span_start(&self) -> usize {
        self.range.start
    }

    /// Returns the ending character position of this token in the source.
    pub fn span_end(&self) -> usize {
        self.range.end
    }
}

/// Fully-resolve Tokens that appear in the Node tree. Tokens are formed by
/// consuming a SyntaxToken from the event stream, including any leading trivia
/// before it and trailing trivia after it.
#[derive(Clone, Debug)]
pub struct Token {
    kind: SyntaxKind,
    text: Substr,
    flags: TokenFlags,
    trivia: TriviaPointer,
}

impl Token {
    pub fn from_syntax(syntax: SyntaxToken, text: Substr, trivia: TriviaPointer) -> Self {
        Self {
            kind: syntax.kind(),
            text,
            flags: syntax.flags,
            trivia,
        }
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }

    pub fn text(&self) -> &str {
        &self.text.as_str()
    }

    pub fn range(&self) -> Range<usize> {
        self.text.range()
    }

    pub fn parent_text(&self) -> &ArcStr {
        &self.text.parent()
    }

    /// Return a single substring containing both the Token's text _and_ all of its trailing trivia.
    pub fn text_with_trailing_trivia(&self) -> Substr {
        let start = self.text.range().start;
        let end = self
            .trivia
            .trailing_trivia()
            .last()
            .map_or(self.text.range().end, |trivia| trivia.text.range().end);

        self.text.parent().substr(start..end)
    }

    /// Return a single substring containing only the trailing trivia of the token.
    pub fn trailing_trivia_text(&self) -> Substr {
        let start = self
            .trivia
            .trailing_trivia()
            .first()
            .map_or(self.text.range().end, |trivia| trivia.text.range().start);
        let end = self
            .trivia
            .trailing_trivia()
            .last()
            .map_or(self.text.range().end, |trivia| trivia.text.range().end);

        self.text.parent().substr(start..end)
    }

    pub fn flags(&self) -> TokenFlags {
        self.flags
    }

    pub fn preceding_contiguous_trivia(&self) -> &[Trivia] {
        self.trivia
            .list
            .preceding_contiguous_trivia(self.text.range().start)
    }

    pub fn following_contiguous_trivia(&self) -> &[Trivia] {
        self.trivia
            .list
            .following_contiguous_trivia(self.text.range().end)
    }

    pub fn leading_trivia(&self) -> &[Trivia] {
        &self.trivia.leading_trivia()
    }

    pub fn trailing_trivia(&self) -> &[Trivia] {
        &self.trivia.trailing_trivia()
    }

    pub fn has_trailing_newline(&self) -> bool {
        self.trailing_trivia()
            .iter()
            .any(|t| t.kind() == SyntaxKind::LINE_ENDING)
    }
}

// `first_token` and `last_token` here are implemented to be compatible with the methods of the
// same name on all Node types, allowing arbitrary access to the first and last tokens of a
// node, no matter what the internal structure is.
impl TokenSpan for Token {
    #[inline(always)]
    fn first_token(&self) -> Option<&Token> {
        Some(&self)
    }
    #[inline(always)]
    fn last_token(&self) -> Option<&Token> {
        Some(&self)
    }
}
