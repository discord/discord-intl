use crate::html_entities::get_html_entity;
use crate::syntax::{TextPointer, TextSize};
use crate::{SyntaxKind, SyntaxToken};
use intl_markdown_macros::generate_ascii_encoding_table;
use memchr::Memchr;
use std::rc::Rc;

pub fn unescaped_pointer_chunks(text: &TextPointer) -> UnescapedChunksIterator {
    UnescapedChunksIterator::new(text)
}

pub struct UnescapedChunksIterator<'a> {
    text: &'a TextPointer,
    cursor: usize,
    slash_iter: Memchr<'a>,
}

impl UnescapedChunksIterator<'_> {
    pub fn new(text: &TextPointer) -> UnescapedChunksIterator {
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
                // ASCII punctuation is allowed to be escaped, so if we reach that, return the
                // chunk up to that point (not including the slash).
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
    TextPointer::new(Rc::from(clone), 0, pointer.len() as TextSize)
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

type AsciiReplacementTable = [Option<&'static str>; 256];

/// An iterator that returns chunks of text where each instance of a matching entity is encoded
/// according to a lookup table containing the replacement text to use.
pub struct AsciiEncodingIter<'a> {
    text: &'a str,
    table: AsciiReplacementTable,
    cursor: usize,
}

impl<'a> AsciiEncodingIter<'a> {
    pub fn new(text: &'a str, table: AsciiReplacementTable) -> Self {
        Self {
            text,
            table,
            cursor: 0,
        }
    }
}

impl<'a> Iterator for AsciiEncodingIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        // No text left, so the iterator is finished.
        if self.cursor >= self.text.len() {
            return None;
        }

        // If we're currently at a character that needs encoding, return that encoding as a
        // standalone chunk.
        let current_bytes = &self.text.as_bytes()[self.cursor..];
        let byte = current_bytes[0] as usize;
        if self.table[byte].is_some() {
            self.cursor += 1;
            return self.table[byte];
        }

        let chunk_end = current_bytes
            .iter()
            .position(|&byte| self.table[byte as usize].is_some())
            .unwrap_or(current_bytes.len());
        let text = &self.text[self.cursor..self.cursor + chunk_end];
        self.cursor += chunk_end;
        Some(text)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // There can only be a maximum of one escape per two characters, so the maximum count is
        // every other character being an escape.
        (1, Some(self.text.len() / 2 + 1))
    }
}

generate_ascii_encoding_table! {
    PERCENT_ENCODING,
    // Characters that can safely appear in URLs without needing a percent encoding. Adapted from:
    // https://github.com/pulldown-cmark/pulldown-cmark/blob/8713a415b04cdb0b7980a9a17c0ed0df0b36395e/pulldown-cmark-escape/src/lib.rs#L28C1-L38C3
    ! b"!#$%()*+,-./0123456789:;=?@ABCDEFGHIJKLMNOPQRSTUVWXYZ^_abcdefghijklmnopqrstuvwxyz~" => "%{:X}",
    // CommonMark spec tests encode `&` to `&amp;` rather than the percent encoding `%26` that it
    // would normally have.
    b'&' => "&amp;"
}

/// Replaces non-ascii and unsafe characters in a url string with their percent encoding. This is
/// specifically to match the CommonMark spec's _tests_, but is not actually defined by the spec
/// itself, and as such there is some slightly special handling, like encoding `&` to `&amp;` rather
/// than the percent encoding `%26` that it would normally have.
pub(crate) fn encode_href(text: &str) -> AsciiEncodingIter {
    AsciiEncodingIter::new(text, PERCENT_ENCODING_REPLACEMENT_TABLE)
}

generate_ascii_encoding_table! {
    HTML_CHAR_ENCODING,
    b'<' => "&lt;",
    b'>' => "&gt;",
    b'"' => "&quot;",
    // This doesn't seem to happen in the spec? But does in some other
    // implementations.
    // '\'' => "&#27;",
    b'&' => "&amp;"
}
pub(crate) fn encode_body_text(text: &str) -> AsciiEncodingIter {
    AsciiEncodingIter::new(text, HTML_CHAR_ENCODING_REPLACEMENT_TABLE)
}

generate_ascii_encoding_table! {
    STRING_LITERAL_ENCODING,
    // ASCII Control characters
    @control => "\\u00{:x}",
    b'"' => "\\\"",
    b'\\' => "\\\\",
    b'/' => "\\/",
    // Control characters that are invalid in Rust:
    b'\x08' => "\\b",
    b'\x0c' => "\\f",
    b'\n' => "\\n",
    b'\r' => "\\r",
    b'\t' => "\\t"
}
/// Replaces ASCII characters that are not representable in JSON strings. This includes the double
/// quote character that would close the string, newlines and other whitespace, and
/// This includes
/// Replaces non-ascii and unsafe characters in a url string with their percent encoding. This is
/// specifically to match the CommonMark spec's _tests_, but is not actually defined by the spec
/// itself, and as such there is some slightly special handling, like encoding `&` to `&amp;` rather
/// than the percent encoding `%26` that it would normally have.
pub(crate) fn encode_json_string_literal(text: &str) -> AsciiEncodingIter {
    AsciiEncodingIter::new(text, HTML_CHAR_ENCODING_REPLACEMENT_TABLE)
}
