use unescape_zero_copy::Error;

/// Copied from unescape_zero_copy::unicode_char, which isn't public.
#[inline]
fn unicode_char(s: &str, chars: usize) -> Result<(char, &str), Error> {
    if s.len() < chars {
        Err(Error::IncompleteSequence)
    } else {
        let num = u32::from_str_radix(&s[0..chars], 16)?;
        let ch = char::from_u32(num).ok_or(Error::InvalidUnicode(num))?;
        Ok((ch, &s[chars..]))
    }
}

/// An ECMA-262 escape string handler. This is much more lenient than the
/// default one from unescape_zero_copy, and is guaranteed to never error out,
/// matching the ECMA definition of escape handling.
///
/// NOTE: This does not currently _fully_ implement the ECMA escape rules, just
/// enough of them to be useful for general strings.
///
/// Single character escapes: https://tc39.es/ecma262/2025/#table-string-single-character-escape-sequences
/// - `\b` - backspace
/// - `\f` - form feed.
/// - `\n` - line feed
/// - `\r` - carriage return
/// - `\t` - tab
/// - `\v` - vertical tab
///
/// Unicode codepoints:
/// - `\uNNNN` as above, but with four hex digits.
/// - `\u{NN...}` as above, but with variable hex digits.
///
/// Different bases:
/// - `\xNN` to the Unicode character in the two hex digits.
///
/// All other escaped values just return the character after the slash directly.
pub(crate) fn js_escape_handler(s: &str) -> Result<(char, &str), Error> {
    type Error = unescape_zero_copy::Error;

    let mut chars = s.chars();
    let next = chars.next().ok_or(Error::IncompleteSequence)?;
    match next {
        '0' => Ok(('\0', chars.as_str())),
        'b' => Ok(('\x08', chars.as_str())),
        'f' => Ok(('\x0C', chars.as_str())),
        'n' => Ok(('\n', chars.as_str())),
        'r' => Ok(('\r', chars.as_str())),
        't' => Ok(('\t', chars.as_str())),
        'v' => Ok(('\x0B', chars.as_str())),
        '\r' | '\n' => Ok((next, chars.as_str())),
        'x' => unicode_char(chars.as_str(), 2),
        'u' => {
            let s = chars.as_str();
            if chars.next() == Some('{') {
                let s = chars.as_str();
                let size = chars.by_ref().take_while(|n| *n != '}').count();
                let num = u32::from_str_radix(&s[0..size], 16)?;
                let ch = char::from_u32(num).ok_or(Error::InvalidUnicode(num))?;
                Ok((ch, chars.as_str()))
            } else {
                unicode_char(s, 4)
            }
        }
        'U' => unicode_char(chars.as_str(), 8),
        _ => {
            let count = s.chars().take_while(|n| n.is_digit(8)).count().min(3);
            if count > 0 {
                let num = u32::from_str_radix(&s[0..count], 8)?;
                let ch = char::from_u32(num).ok_or(Error::InvalidUnicode(num))?;
                Ok((ch, &s[count..]))
            } else {
                Ok((next, chars.as_str()))
            }
        }
    }
}
