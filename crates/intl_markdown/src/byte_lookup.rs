use intl_markdown_macros::generate_ascii_lookup_table;

generate_ascii_lookup_table!(
    SIGNIFICANT_PUNCTUATION_BYTES,
    b"\n\x0C\r!\"$&'()*:<>[\\]_`{}~#"
);

/// Returns true if the given byte represents a significant character that
/// could become a new type of token. This effectively just includes
/// punctuation and newline characters.
///
/// Note that these are only the characters that are significant when they
/// interrupt textual content. For example, a `-` could become a MINUS token,
/// but within a word it can never be significant, e.g. the dash in `two-part`
/// is not significant.
///
/// Inline whitespace in this context _is not_ considered significant, but
/// vertical whitespace _is_ significant.
#[inline(always)]
pub(crate) fn byte_is_significant_punctuation(byte: u8) -> bool {
    SIGNIFICANT_PUNCTUATION_BYTES[byte as usize] != 0
}

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

/// Returns true if the char is valid as the starting character of a unicode identifier.
#[inline(always)]
pub(crate) fn is_unicode_identifier_start(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_start(c)
}

/// Returns true if the char is valid as any character after the start of a unicode identifier.
#[inline(always)]
pub(crate) fn is_unicode_identifier_continue(c: char) -> bool {
    unicode_xid::UnicodeXID::is_xid_continue(c)
}
