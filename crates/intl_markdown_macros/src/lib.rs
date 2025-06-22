mod ascii_table;
mod impl_format;

use proc_macro::TokenStream;

use crate::ascii_table::{generate_ascii_encoding_table_impl, generate_byte_lookup_table_impl};
use crate::impl_format::impl_format_impl;

/// Generate an ASCII Lookup Table where each byte of the given string in the
/// table are marked as true and everything else is false. The table will be
/// assigned to a new static constant with the given name.
///
/// ```ignore
/// generate_byte_lookup_table!(WHITESPACE, b"\n\r \t");
/// ```
#[proc_macro]
pub fn generate_byte_lookup_table(input: TokenStream) -> TokenStream {
    generate_byte_lookup_table_impl(input)
}
#[proc_macro]
pub fn generate_ascii_encoding_table(input: TokenStream) -> TokenStream {
    generate_ascii_encoding_table_impl(input)
}

#[proc_macro]
pub fn impl_format(input: TokenStream) -> TokenStream {
    impl_format_impl(input)
}
