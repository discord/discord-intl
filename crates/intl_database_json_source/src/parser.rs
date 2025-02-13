use intl_database_core::{key_symbol, RawMessageTranslation, RawPosition};
use std::borrow::Cow;
use std::ops::Range;
use unescape_zero_copy::Error;

// COPIED FROM byte_lookup.rs in crates/intl_markdown.
// Learned from: https://nullprogram.com/blog/2017/10/06/
#[rustfmt::skip]
static UTF8_LENGTH_LOOKUP: [usize; 32] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 3, 3, 4, 0,
];

/// Return the byte length of the complete UTF-8 code point that starts with `byte`. This can be
/// done branchlessly and without computing the entire `char`.
#[inline(always)]
pub(crate) fn char_length_from_byte(byte: u8) -> usize {
    UTF8_LENGTH_LOOKUP[byte as usize >> 3]
}

struct TranslationsJsonParser<'a> {
    text: &'a str,
    position: usize,
    line: usize,
    last_line_start: usize,
    has_failed: bool,
}

impl<'a> TranslationsJsonParser<'a> {
    fn new(text: &'a str) -> TranslationsJsonParser<'a> {
        Self {
            text,
            position: 0,
            line: 1,
            last_line_start: 0,
            has_failed: false,
        }
    }

    #[inline]
    fn current(&self) -> u8 {
        self.text.as_bytes()[self.position]
    }

    #[inline]
    fn peek(&self, n: usize) -> u8 {
        self.text.as_bytes()[self.position + n]
    }

    #[inline]
    fn current_slice(&self) -> &[u8] {
        &self.text.as_bytes()[self.position..]
    }

    #[inline]
    fn str_slice(&self, range: Range<usize>) -> &str {
        &self.text[range]
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.position >= self.text.len()
    }

    #[allow(unused)]
    fn expect(&mut self, expected: u8) -> Option<()> {
        if self.current() == expected {
            self.position += char_length_from_byte(expected);
            return Some(());
        }
        self.has_failed = true;
        None
    }

    fn consume_lines(&mut self, line_count: usize, new_position: usize) {
        self.line += line_count;
        self.last_line_start = new_position;
    }

    /// Advance the parser until the next quote character, typically for moving
    /// from the end of a message to the start of the next, or between the key
    /// and the value of the same message.
    /// This is _not_ safe for parsing message content that may contain escapes.
    fn advance_past_quote(&mut self) -> Option<usize> {
        let iter = memchr::memchr3_iter(b'"', b'\n', b'}', self.current_slice());

        let search_start = self.position;
        let mut line_count = 0;
        let mut last_line_position = 0;
        let mut has_match = false;
        for next in iter {
            let next_position = search_start + next + 1;
            match self.peek(next) {
                b'\n' => {
                    last_line_position = next_position;
                    line_count += 1
                }
                b'"' => {
                    has_match = true;
                    self.position = next_position;
                    break;
                }
                // `}` will always be the end of the object when advancing at the top level.
                // Otherwise, we would expect to hit a `"` first in this search.
                b'}' => {
                    self.position = self.text.len();
                    break;
                }
                unexpected => unreachable!(
                    "memchr match found something other than expected: {}",
                    unexpected
                ),
            }
        }

        if line_count > 0 {
            self.consume_lines(line_count, last_line_position);
        }
        if !has_match {
            self.has_failed = true;
            return None;
        }

        Some(self.position)
    }

    fn advance_past(&mut self, needle: u8) -> Option<usize> {
        let iter = memchr::memchr2_iter(needle, b'\n', self.current_slice());

        let search_start = self.position;
        let mut line_count = 0;
        let mut last_line_position = 0;
        let mut has_match = false;
        for next in iter {
            let next_position = search_start + next + 1;
            match self.peek(next) {
                b'\n' => {
                    last_line_position = next_position;
                    line_count += 1
                }
                _ => {
                    has_match = true;
                    self.position = next_position;
                    break;
                }
            }
        }

        if line_count > 0 {
            self.consume_lines(line_count, last_line_position);
        }
        if !has_match {
            self.has_failed = true;
            return None;
        }

        Some(self.position)
    }

    fn advance_to_message_end(&mut self) -> Option<(usize, bool)> {
        let Some(first) = memchr::memchr2(b'"', b'\\', self.current_slice()) else {
            return None;
        };
        self.position += first;

        // If the first found index is just an end quote, then the message has no escapes and can
        // skip extra processing.
        if self.current() == b'"' {
            self.position += 1;
            return Some((self.position, false));
        }

        // Otherwise, finding an escape means it'll need to increment more carefully. To keep
        // it simple, we'll just revert to linear lexing from here.
        let mut last_was_escape_char = false;
        loop {
            match self.current() {
                b'\n' => {
                    self.has_failed = true;
                    return None;
                }
                b'"' if !last_was_escape_char => {
                    self.position += 1;
                    return Some((self.position, true));
                }
                b'\\' => last_was_escape_char = !last_was_escape_char,
                _ => last_was_escape_char = false,
            }
            self.position += char_length_from_byte(self.current());
            if self.is_eof() {
                return None;
            }
        }
    }

    fn advance_after_message(&mut self) -> Option<usize> {
        let iter = memchr::memchr3_iter(b',', b'\n', b'}', self.current_slice());

        let search_start = self.position;
        let mut line_count = 0;
        let mut last_line_position = 0;
        let mut has_match = false;
        for next in iter {
            let next_position = search_start + next + 1;
            match self.peek(next) {
                b'\n' => {
                    line_count += 1;
                    last_line_position = next_position;
                }
                b',' => {
                    has_match = true;
                    self.position = next_position;
                    break;
                }
                b'}' => {
                    has_match = true;
                    self.position = self.text.len();
                    break;
                }
                unexpected => unreachable!(
                    "memchr match found something other than expected: {}",
                    unexpected
                ),
            }
        }

        if line_count > 0 {
            self.consume_lines(line_count, last_line_position);
        }
        if !has_match {
            self.has_failed = true;
            return None;
        }

        Some(self.position)
    }

    fn parse_start(&mut self) {
        // Assert a valid object start and advance into the content of the object. We can
        // _technically_ skip this, but we want some assurance that the input is well-formed beyond
        // just assuming it will be.
        self.advance_past(b'{');
    }

    fn parse_message(&mut self) -> Option<RawMessageTranslation> {
        // Presume the last parse advanced past the `,` between messages or the
        // end of the object, so this should always succeed.
        let key_start = self.advance_past_quote()?;
        // Key parsing is anything until the next quote.
        // We make the assumption that it won't contain escapes,`}`, or newlines.
        let key_end = self.advance_past_quote()? - 1;

        // Advance through to the value start
        self.advance_past(b':')?;
        // Then get the value bounds
        let value_start = self.advance_past_quote()?;
        let value_line = self.line;
        let value_column = value_start - self.last_line_start;
        let (value_end, has_escapes) = self.advance_to_message_end()?;

        // Finally advance through to the end of the message set.
        self.advance_after_message()?;

        let message_key = self.str_slice(key_start..key_end);
        let raw = self.str_slice(value_start..value_end - 1);
        let value = if has_escapes {
            key_symbol(&unescape_json_str(raw).ok()?)
        } else {
            key_symbol(raw)
        };

        Some(RawMessageTranslation::new(
            key_symbol(message_key),
            RawPosition {
                line: value_line as u32,
                // We make an assumption here that there is no unicode in the message key, meaning
                // the column is just the number of bytes since the last newline.
                col: value_column as u32,
            },
            value,
        ))
    }

    #[inline]
    fn parse_next(&mut self) -> Option<RawMessageTranslation> {
        if self.has_failed || self.is_eof() {
            return None;
        }

        self.parse_message()
    }
}

impl Iterator for TranslationsJsonParser<'_> {
    type Item = RawMessageTranslation;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next()
    }
}

fn unescape_json_str(str: &str) -> Result<Cow<'_, str>, Error> {
    unescape_zero_copy::unescape(json_escape_sequence, str)
}
pub fn json_escape_sequence(s: &str) -> Result<(char, &str), Error> {
    let mut chars = s.chars();
    let next = chars.next().ok_or(Error::IncompleteSequence)?;
    match next {
        'b' => Ok(('\x08', chars.as_str())),
        'f' => Ok(('\x0C', chars.as_str())),
        'n' => Ok(('\n', chars.as_str())),
        'r' => Ok(('\r', chars.as_str())),
        't' => Ok(('\t', chars.as_str())),
        '\r' | '\n' => Ok((next, chars.as_str())),
        'u' => {
            let first = u32::from_str_radix(&s[1..5], 16)?;
            // This is the BMP surrogate range for the second character in a surrogate pair. If the
            // value is in this range and this step, then we know it's incorrect since it can't
            // stand on its own.
            if (0xDC00..=0xDFFF).contains(&first) {
                return Err(Error::InvalidUnicode(first));
            }
            if !(0xD800..=0xDBFF).contains(&first) {
                // Characters outside the surrogate range should always be valid.
                let next = char::from_u32(first).unwrap();
                return Ok((next, &s[5..]));
            }
            // Now the value must definitely be a surrogate pair, so the second one should follow
            // immediately, `\uXXXX\uXXXX`. We need to skip past the next `\u` as well to get the
            // following bytes.
            let second = u32::from_str_radix(&s[7..11], 16)?;
            // Taken from serde_json: https://github.com/serde-rs/json/blob/1d7378e8ee87e9225da28094329e06345b76cd99/src/read.rs#L969
            let next =
                char::from_u32((((first - 0xD800) << 10) | (second - 0xDC00)) + 0x1_0000).unwrap();
            Ok((next, &s[11..]))
        }
        ch => Ok((ch, chars.as_str())),
    }
}

/// Parse the given `text` as a single, flat JSON object of message keys to
/// message values. The JSON is assumed to be well-formed, and minimal error
/// handling is implemented.
///
/// Since sources are able to accept an iterator of translation values, this
/// parser never stores the completed object in memory and instead yields each
/// translation as it is parsed.
///
/// This parser also handles tracking line and column positions within the
/// source, which is the primary reason for this implementation over existing
/// libraries like serde that only track that state internally.
///
/// Note that some extra assumptions are made here for the sake of simplicity
/// and efficient parsing that other implementations aren't able to make:
/// - The given text is a well-formed, flat JSON object
/// - Keys may not contain escaped quotes, `}`, or newlines.
/// - The only important positional information is the first character of the value.
/// - There will be no errors during parsing. The iterator will return None instead.
pub fn parse_flat_translation_json(
    text: &str,
) -> impl Iterator<Item = RawMessageTranslation> + use<'_> {
    let mut parser = TranslationsJsonParser::new(text);
    parser.parse_start();
    parser.into_iter()
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_line_column(value: &RawMessageTranslation, line: u32, col: u32) {
        assert_eq!(value.position.line, line);
        assert_eq!(value.position.col, col);
    }

    #[test]
    pub fn test_empty_object() {
        let result = parse_flat_translation_json("{}").collect::<Vec<_>>();
        assert!(result.is_empty());
    }

    #[test]
    pub fn test_empty_object_with_spaces() {
        let result = parse_flat_translation_json(" {  \n }  ").collect::<Vec<_>>();
        assert!(result.is_empty());
    }

    #[test]
    pub fn test_one_message() {
        let result = parse_flat_translation_json(r#"{"KEY": "value"}  "#).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "KEY");
        assert_eq!(result[0].value.raw, "value");
    }

    #[test]
    pub fn test_multiple_dense_messages() {
        let result = parse_flat_translation_json(r#"{"SINGLE_KEY": "value","KEY2":"value2"}  "#)
            .collect::<Vec<_>>();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "SINGLE_KEY");
        assert_eq!(result[0].value.raw, "value");
    }

    #[test]
    pub fn test_single_trailing_comma() {
        let result = parse_flat_translation_json(r#"{"KEY": "trailing",}  "#).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "KEY");
        assert_eq!(result[0].value.raw, "trailing");
    }

    #[test]
    pub fn test_multiple_trailing_comma() {
        let result = parse_flat_translation_json(r#"{"KEY": "value","KEY2":"value2",}  "#)
            .collect::<Vec<_>>();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "KEY");
        assert_eq!(result[0].value.raw, "value");
        assert_eq!(result[1].name, "KEY2");
        assert_eq!(result[1].value.raw, "value2");
    }

    #[test]
    pub fn test_spaced_out_json() {
        let result = parse_flat_translation_json(
            r#"{
        "KEY"  : "value"
        ,
        "KEY2":
        "value2",

        }  "#,
        )
        .collect::<Vec<_>>();
        println!("{:?}", result);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "KEY");
        assert_eq!(result[0].value.raw, "value");
        assert_line_column(&result[0], 2, 18);
        assert_eq!(result[1].name, "KEY2");
        assert_eq!(result[1].value.raw, "value2");
        assert_line_column(&result[1], 5, 9);
    }

    #[test]
    pub fn test_unicode_value() {
        let result =
            parse_flat_translation_json(r#"{"EMAIL": "Въведи код",}  "#).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "EMAIL");
        assert_eq!(result[0].value.raw, "Въведи код");
    }

    #[test]
    pub fn test_emoji_value() {
        let result = parse_flat_translation_json(r#"{"SPEAKER": "its a speaker 🔈",}  "#)
            .collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "SPEAKER");
        assert_eq!(result[0].value.raw, "its a speaker 🔈");
    }

    #[test]
    pub fn test_escaped_unicode_value() {
        let result =
            parse_flat_translation_json(r#"{"ESCAPED": "escaped speaker \uD83D\uDD08",}  "#)
                .collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "ESCAPED");
        assert_eq!(result[0].value.raw, "escaped speaker 🔈");
    }

    #[test]
    pub fn test_ascii_escaped_values() {
        let result =
            parse_flat_translation_json(r#"{"ESCAPED": "\n\t\f55\/\r\\",}  "#).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "ESCAPED");
        assert_eq!(result[0].value.raw, "\n\t\x0C55/\r\\");
    }

    #[test]
    pub fn test_multibyte_unicode_in_value() {
        let result = parse_flat_translation_json(r#"{"ELIPSIS": "hello…",}  "#).collect::<Vec<_>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].value.raw, "hello…");
    }
}
