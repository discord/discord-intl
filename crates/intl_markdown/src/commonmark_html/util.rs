use memchr::Memchr;

pub fn unescaped_chunks(text: &str) -> UnescapedChunksIterator {
    UnescapedChunksIterator::new(text)
}

pub struct UnescapedChunksIterator<'a> {
    text: &'a str,
    cursor: usize,
    slash_iter: Memchr<'a>,
}

impl UnescapedChunksIterator<'_> {
    pub fn new(text: &str) -> UnescapedChunksIterator {
        UnescapedChunksIterator {
            text,
            cursor: 0,
            slash_iter: memchr::memchr_iter(b'\\', &text.as_bytes()),
        }
    }
}

impl<'a> Iterator for UnescapedChunksIterator<'a> {
    type Item = &'a str;

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
                let remaining_text = &self.text[chunk_start..];
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
                    let text = &self.text[chunk_start..self.cursor];
                    self.cursor += 1;
                    return Some(text);
                }
                // Carriage returns are removed entirely, so we still return the chunk up to this
                // point, but push the cursor past the `r` as well.
                b'\r' => {
                    let text = &self.text[chunk_start..self.cursor];
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

pub struct WriteNewlinesAsSpaces<'a> {
    output: &'a mut dyn std::fmt::Write,
}
impl std::fmt::Write for WriteNewlinesAsSpaces<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let mut lines = s.lines();
        if let Some(line) = lines.next() {
            self.output.write_str(line)?;
        };
        for line in lines {
            self.output.write_str(" ")?;
            self.output.write_str(line)?;
        }
        if s.ends_with('\n') {
            self.output.write_str(" ")?;
        }
        Ok(())
    }

    fn write_char(&mut self, c: char) -> std::fmt::Result {
        if c == '\n' {
            self.write_str(" ")
        } else {
            self.write_str(&c.to_string())
        }
    }
}

pub fn convert_newlines_to_spaces(target: &mut dyn std::fmt::Write) -> WriteNewlinesAsSpaces {
    WriteNewlinesAsSpaces { output: target }
}

/// Quickly replace instances of `needle` in `haystack` with `replacement` in place, using the fact
/// that  the needle and replacement have the same byte length to avoid a new allocation.
pub fn fast_replace(haystack: &mut str, needle: u8, replacement: u8) -> &str {
    debug_assert!(
        needle.is_ascii(),
        "Needle is not an ASCII character. Cannot guarantee UTF-8 validity"
    );
    debug_assert!(
        replacement.is_ascii(),
        "Replacement is not an ASCII character. Cannot guarantee UTF-8 validity"
    );
    // SAFETY: We're only working with a single byte replacement of ASCII characters, so there's no
    // worry about creating invalid UTF-8 sequences.
    let bytes = unsafe { haystack.as_bytes_mut() };
    let matches: Vec<usize> = memchr::memchr_iter(needle, bytes).collect();
    for index in matches {
        bytes[index] = replacement;
    }
    haystack
}
