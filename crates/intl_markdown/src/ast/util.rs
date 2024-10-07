use std::borrow::Cow;

use memchr::memchr;

use crate::ast::InlineContent;

fn icu_markdown_escape_handler(s: &str) -> Result<(Option<char>, &str), unescape_zero_copy::Error> {
    let mut chars = s.chars();
    let next = chars
        .next()
        .ok_or(unescape_zero_copy::Error::IncompleteSequence)?;
    match next {
        // ASCII punctuation is allowed to be escaped.
        c if c.is_ascii_punctuation() => Ok((Some(c), chars.as_str())),
        // Carriage returns are removed
        '\r' => Ok((None, chars.as_str())),
        // Everything else is preserved as-is.
        _ => Ok((Some('\\'), s)),
    }
}

// Handle unescaping backslash characters (e.g., turning `\!` into `!`) and removing carriage
// returns from the input.
pub(crate) fn unescape(text: &str) -> String {
    unescape_cow(text).to_string()
}
// Handle unescaping backslash characters (e.g., turning `\!` into `!`) and removing carriage
// returns from the input.
pub(crate) fn unescape_cow(text: &str) -> Cow<str> {
    // Most strings don't contain escapes, so doing a quick lookup over the
    // whole string can speed up the normal case.
    if let None = memchr(b'\\', text.as_bytes()) {
        return Cow::Borrowed(text);
    }
    match unescape_zero_copy::unescape(icu_markdown_escape_handler, text.as_ref()) {
        Ok(result) => result,
        _ => Cow::Borrowed(text),
    }
}

// Taken from:
// https://github.com/pulldown-cmark/pulldown-cmark/blob/8713a415b04cdb0b7980a9a17c0ed0df0b36395e/pulldown-cmark-escape/src/lib.rs#L28C1-L38C3
// This list indicates ascii characters that are safe to preserve in a url.
#[rustfmt::skip]
static HREF_SAFE: [u8; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 1,
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0,
];

/// Replaces non-ascii and unsafe characters in a url string with their percent encoding. This is
/// specifically to match the CommonMark spec's _tests_, but is not actually defined by the spec
/// itself, and as such there is some slightly special handling, like encoding `&` to `&amp;` rather
/// than the percent encoding `%26` that it would normally have.
pub(crate) fn escape_href(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for (index, c) in text.char_indices() {
        if !c.is_ascii() || HREF_SAFE[c as usize] == 0 {
            match c {
                '&' => result.push_str("&amp;"),
                _ => {
                    for byte_index in index..index + c.len_utf8() {
                        result.push('%');
                        result.push_str(&format!("{:X}", text.as_bytes()[byte_index]));
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[inline]
fn get_special_entity_replacement(c: char) -> Option<&'static str> {
    match c {
        // A few special chars are escaped to entities in html output
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '"' => Some("&quot;"),
        // This doesn't seem to happen in the spec? But does in some other
        // implementations.
        // '\'' => Some("&#39;"),
        '&' => Some("&amp;"),
        _ => None,
    }
}

fn replace_special_entities(text: &str) -> String {
    let mut result = String::with_capacity(text.len());

    for c in text.chars() {
        match get_special_entity_replacement(c) {
            Some(entity) => result.push_str(entity),
            _ => result.push(c),
        }
    }

    result
}

pub(crate) fn escape_body_text(text: &str) -> String {
    replace_special_entities(text)
}

/// Processes the list of inline elements by taking only the visual text that appears within each
/// item. For example, a `Strong` element like `**hello**` would just be written as `hello` rather
/// than `<strong>hello</strong>` as it might in an html format.
pub(crate) fn format_plain_text(elements: &Vec<InlineContent>) -> String {
    let mut buffer = String::new();
    format_plain_text_inner(&mut buffer, &elements);
    buffer
}

fn format_plain_text_inner(buffer: &mut String, elements: &Vec<InlineContent>) {
    for element in elements {
        match element {
            InlineContent::Text(text) => buffer.push_str(&text),
            InlineContent::Strong(strong) => format_plain_text_inner(buffer, strong.content()),
            InlineContent::Emphasis(emphasis) => {
                format_plain_text_inner(buffer, emphasis.content())
            }
            InlineContent::Link(link) => format_plain_text_inner(buffer, link.label()),
            InlineContent::CodeSpan(code_span) => buffer.push_str(code_span.content()),
            InlineContent::HardLineBreak => {}
            InlineContent::Hook(hook) => format_plain_text_inner(buffer, hook.content()),
            InlineContent::Strikethrough(strikethrough) => {
                format_plain_text_inner(buffer, strikethrough.content())
            }
            InlineContent::Icu(_) => todo!(),
            InlineContent::IcuPound => buffer.push('#'),
        }
    }
}
