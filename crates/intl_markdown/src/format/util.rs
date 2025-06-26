use intl_markdown_macros::generate_ascii_encoding_table;

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
