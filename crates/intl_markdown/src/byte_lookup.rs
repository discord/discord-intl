use intl_markdown_macros::generate_byte_lookup_table;

generate_byte_lookup_table!(
    SIGNIFICANT_PUNCTUATION_BYTES,
    ByteType,
    PUNCT => b"!\"$&'()*:<>[\\]_`{}~#",
    INLINE_SPACE => b" \t",
    LINE => b"\n\r\x0c",
);

pub(crate) mod simd {
    use crate::byte_lookup::{ByteType, SIGNIFICANT_PUNCTUATION_BYTES};
    use lazy_static::lazy_static;
    use std::simd::num::SimdUint;
    use std::simd::prelude::*;
    use std::simd::{Mask, Simd};

    const LANES: usize = 16;
    lazy_static! {
        static ref PUNCT: Simd<u8, LANES> = Simd::splat(ByteType::PUNCT as u8);
        static ref INLINE_SPACE: Simd<u8, LANES> = Simd::splat(ByteType::INLINE_SPACE as u8);
        static ref LINE: Simd<u8, LANES> = Simd::splat(ByteType::LINE as u8);
        static ref EMPTY: Simd<u8, LANES> = Simd::splat(ByteType::LINE as u8);
        static ref ENABLE_MASK: Mask<isize, LANES> = Mask::from_array([true; LANES]);
        static ref EMPTY_MASK: Mask<i8, LANES> = Mask::from_array([false; LANES]);
    }

    #[inline(always)]
    pub(crate) fn scan_for_significant_bytes(text: &[u8], merge_inner_whitespace: bool) -> usize {
        if text.len() < LANES {
            return 0;
        }

        let mut offset = 0;
        let chunk_count = text.len() / LANES;
        for _ in 0..chunk_count {
            // The chunk of text to analyze in the vector
            let chunk_bytes: Simd<u8, LANES> = Simd::load_or(
                &text[offset..std::cmp::min(text.len(), offset + LANES)],
                *PUNCT,
            );
            // Get the [ByteType] value of each byte in the vector
            let source = Simd::splat(SIGNIFICANT_PUNCTUATION_BYTES.as_ptr())
                .wrapping_add(chunk_bytes.cast::<usize>());

            let gathered = unsafe { Simd::gather_select_ptr(source, *ENABLE_MASK, *EMPTY) };
            let punct_mask = gathered.simd_eq(*PUNCT);
            let line_mask = gathered.simd_eq(*LINE);
            let space_mask = gathered.simd_eq(*INLINE_SPACE);

            let punct_index = punct_mask.first_set().unwrap_or(usize::MAX);
            let line_index = line_mask.first_set().unwrap_or(usize::MAX);
            let space_index = if merge_inner_whitespace
                && punct_index == usize::MAX
                && line_index == usize::MAX
            {
                // If the last character isn't a space and there are no other significant bytes in
                // the text, then we can "skip" this space as insignificant.
                if !space_mask.test(LANES - 1) {
                    usize::MAX
                } else {
                    space_mask.first_set().unwrap_or(usize::MAX)
                }
            } else {
                space_mask.first_set().unwrap_or(usize::MAX)
            };

            let found = std::cmp::min(punct_index, std::cmp::min(line_index, space_index));
            if found < usize::MAX {
                return offset + found;
            }

            offset += LANES;
        }

        std::cmp::min(text.len(), offset)
    }
}

pub(crate) fn get_byte_type(byte: u8) -> ByteType {
    ByteType::from(SIGNIFICANT_PUNCTUATION_BYTES[byte as usize])
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
