use std::borrow::Cow;
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

pub(crate) fn unescape_json_str(str: &str) -> Result<Cow<'_, str>, Error> {
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
