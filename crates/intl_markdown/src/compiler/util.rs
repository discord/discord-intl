use crate::{AnyInlineNode, InlineContent, SyntaxKind, SyntaxToken, Visit, VisitWith};
use intl_markdown_syntax::html_entities::get_html_entity;
use intl_markdown_syntax::{
    MinimalTextIter, PositionalIterator, Syntax, SyntaxNodeTokenIter, TextPointer, TextSize,
    TokenTextIter, TokenTextIterOptions,
};
use memchr::Memchr;
use std::sync::Arc;

pub fn iter_tokens<N: Syntax>(node: &N) -> SyntaxNodeTokenIter<'_> {
    node.syntax().iter_tokens()
}

pub fn iter_node_text<N: Syntax>(
    node: &N,
    options: TokenTextIterOptions,
) -> MinimalTextIter<TokenTextIter<'_, SyntaxNodeTokenIter<'_>>> {
    iter_tokens(node)
        .into_text_iter()
        .with_options(options)
        .minimal()
}

pub struct PlainTextFormatter {
    pointers: Vec<TextPointer>,
    text_options: TokenTextIterOptions,
}

impl PlainTextFormatter {
    fn new(text_options: TokenTextIterOptions) -> Self {
        Self {
            pointers: Vec::with_capacity(4),
            text_options,
        }
    }

    pub fn format<N: VisitWith<Self>>(
        node: &N,
        text_options: TokenTextIterOptions,
    ) -> MinimalTextIter<std::vec::IntoIter<TextPointer>> {
        let mut f = Self::new(text_options);
        node.visit_with(&mut f);
        f.finish()
    }

    pub fn finish(self) -> MinimalTextIter<std::vec::IntoIter<TextPointer>> {
        MinimalTextIter::new(self.pointers.into_iter())
    }

    fn collect_token_pointers(&mut self, tokens: impl Iterator<Item = SyntaxToken>) {
        for token in tokens {
            self.pointers.push(token.text_pointer().clone())
        }
    }
}

impl Visit for PlainTextFormatter {
    /// Processes the list of inline elements by taking only the visual text that appears within each
    /// item. For example, a `Strong` element like `**hello**` would just be written as `hello` rather
    /// than `<strong>hello</strong>` as it might in an HTML format.
    fn visit_inline_content(&mut self, node: &InlineContent) {
        for (position, child) in node.children().with_positions() {
            match child {
                AnyInlineNode::TextSpan(node) => {
                    // Text spans are the only non-delimited kinds of text possible in inline
                    // content, so they're the only ones that need to potentially apply trims. All
                    // other trivia is preserved since it is important within delimiters.
                    for (span_position, text_child) in node.children().with_positions() {
                        self.pointers.push(text_child.trimmed_text_pointer(
                            self.text_options.trim_kind(
                                position.is_first() && span_position.is_first(),
                                position.is_last() && span_position.is_last(),
                            ),
                        ))
                    }
                }
                AnyInlineNode::Autolink(node) => {
                    self.pointers.push(node.uri_token().text_pointer().clone());
                }
                AnyInlineNode::CodeSpan(node) => {
                    self.collect_token_pointers(node.content().children());
                }
                AnyInlineNode::Link(node) => node.content().visit_with(self),
                AnyInlineNode::Image(node) => node.content().visit_with(self),
                AnyInlineNode::Hook(node) => node.content().visit_with(self),
                AnyInlineNode::Emphasis(node) => node.content().visit_with(self),
                AnyInlineNode::Strong(node) => node.content().visit_with(self),
                AnyInlineNode::Strikethrough(node) => node.content().visit_with(self),
                AnyInlineNode::Icu(_) => todo!(),
                AnyInlineNode::IcuPound(_) => todo!(),
            }
        }
    }
}

pub fn unescaped_pointer_chunks(text: &TextPointer) -> UnescapedChunksIterator<'_> {
    UnescapedChunksIterator::new(text)
}

pub struct UnescapedChunksIterator<'a> {
    text: &'a TextPointer,
    cursor: usize,
    slash_iter: Memchr<'a>,
}

impl UnescapedChunksIterator<'_> {
    pub fn new(text: &TextPointer) -> UnescapedChunksIterator<'_> {
        UnescapedChunksIterator {
            text,
            cursor: 0,
            slash_iter: memchr::memchr_iter(b'\\', &text.as_bytes()),
        }
    }
}

impl<'a> Iterator for UnescapedChunksIterator<'a> {
    type Item = TextPointer;

    fn next(&mut self) -> Option<Self::Item> {
        // No text left, so the iterator is finished.
        if self.cursor >= self.text.len() {
            return None;
        }

        let chunk_start = self.cursor;
        // Since it's possible that an escape might not be removable using Markdown rules (like a
        // `\f` being preserved as-is), we can keep looping until we find an actual escape to
        // reduce the total number of chunks being processed.
        loop {
            let next_slash = self.slash_iter.next();
            // If there's no next slash, or if it's the last character in the text, then just
            // consume the rest of the text together since it can't be a valid escape.
            if next_slash.is_none_or(|next| next == self.text.len() - 1) {
                let remaining_text = self.text.substr(chunk_start..);
                self.cursor = self.text.len();
                return Some(remaining_text);
            };

            self.cursor = next_slash.unwrap();
            // Now that we're at the slash, check the next character to know how to proceed
            // according to the Markdown rules.
            let next = self.text.as_bytes()[self.cursor + 1];
            match next {
                // If the next character is also a slash, it's like a normal escaped character, but
                // the `slash_iter` will also yield that escaped slash in the next iteration. To
                // avoid incidentally removing both slashes, the next entry from the iterator is
                // discarded before continuing, allowing every even-numbered slash to pass through.
                //
                // For example, `\\\\foo\\` _should_ yield "\\foo\", but without discarding the
                // iterator entries would remove all slashes and become just "foo".
                //
                // See `tests::spec_regression::regression_3` for an example.
                b'\\' => {
                    let text = self.text.substr(chunk_start..self.cursor);
                    // Intentionally unused to discard the second slash in the escape.
                    self.slash_iter.next();
                    self.cursor += 1;
                    return Some(text);
                }
                // Other ASCII punctuation is allowed to be escaped, so if we reach that, return
                // the chunk up to that point (not including the slash).
                c if c.is_ascii_punctuation() => {
                    let text = self.text.substr(chunk_start..self.cursor);
                    self.cursor += 1;
                    return Some(text);
                }
                // Carriage returns are removed entirely, so we still return the chunk up to this
                // point, but push the cursor past the `r` as well.
                b'\r' => {
                    let text = self.text.substr(chunk_start..self.cursor);
                    self.cursor += 2;
                    return Some(text);
                }
                // Any other character is not treated as an escape, so we can continue this chunk.
                _ => {}
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // There can only be a maximum of one escape per two characters, so the maximum count is
        // every other character being an escape.
        (1, Some(self.text.len() / 2 + 1))
    }
}

/// Quickly replace instances of `needle` in `haystack` with `replacement` in place, using the fact
/// that the needle and replacement have the same byte length to avoid a new allocation.
pub fn fast_replace_pointer(pointer: TextPointer, needle: u8, replacement: u8) -> TextPointer {
    debug_assert!(
        needle.is_ascii(),
        "Needle is not an ASCII character. Cannot guarantee UTF-8 validity"
    );
    debug_assert!(
        replacement.is_ascii(),
        "Replacement is not an ASCII character. Cannot guarantee UTF-8 validity"
    );
    // If there are no matches, we don't need to change the pointer at all and can just return the
    // same one.
    let matches: Vec<usize> = memchr::memchr_iter(needle, pointer.as_bytes()).collect();
    if matches.len() == 0 {
        return pointer;
    }
    let mut clone: Box<str> = Box::from(pointer.as_str());
    // SAFETY: We're only working with a single byte replacement of ASCII characters, so there's no
    // worry about creating invalid UTF-8 sequences.
    let bytes = unsafe { clone.as_bytes_mut() };
    for index in matches {
        bytes[index] = replacement;
    }
    TextPointer::new(Arc::from(clone), 0, pointer.len() as TextSize)
}

pub fn get_referenced_char(text: &str, radix: u32) -> Box<str> {
    // SAFETY: We're already replacing invalid chars with `REPLACEMENT_CHARACTER`.
    let replacement = u32::from_str_radix(text, radix)
        .ok()
        .and_then(|c| (c > 0).then_some(c))
        .and_then(char::from_u32)
        .unwrap_or(char::REPLACEMENT_CHARACTER);
    String::from(replacement).into_boxed_str()
}

pub fn replace_entity_reference(token: &SyntaxToken) -> TextPointer {
    match token.kind() {
        SyntaxKind::HTML_ENTITY => get_html_entity(token.text().as_bytes())
            .map(TextPointer::from_str)
            .unwrap_or(token.text_pointer().clone()),
        SyntaxKind::HEX_CHAR_REF => {
            get_referenced_char(&token.text()[3..token.text().len() - 1], 16).into()
        }
        SyntaxKind::DEC_CHAR_REF => {
            get_referenced_char(&token.text()[2..token.text().len() - 1], 10).into()
        }
        kind => unreachable!(
            "Caller should not allow token of kind {:?} to reach `replace_entity_reference`",
            kind
        ),
    }
}
