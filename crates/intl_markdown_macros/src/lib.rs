mod ascii_table;
mod impl_format;

use crate::ascii_table::{generate_ascii_encoding_table_impl, generate_byte_lookup_table_impl};
use crate::impl_format::impl_format_impl;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Ident, LitStr, Token};

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

#[proc_macro]
pub fn header_tag_lookup_map(input: TokenStream) -> TokenStream {
    struct HeaderTagLookupInput {
        constant_name: Ident,
        function_name: Ident,
        pattern: String,
    }
    impl Parse for HeaderTagLookupInput {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let constant_name = input.parse::<Ident>()?;
            input.parse::<Token![,]>()?;
            let function_name = input.parse::<Ident>()?;
            input.parse::<Token![,]>()?;
            let pattern = input.parse::<LitStr>()?;
            Ok(HeaderTagLookupInput {
                constant_name,
                function_name,
                pattern: pattern.value(),
            })
        }
    }
    let input = parse_macro_input!(input as HeaderTagLookupInput);
    let constant_name = &input.constant_name;
    let function_name = &input.function_name;

    let tags = (1..=6)
        .map(|i| input.pattern.replace("{}", &i.to_string()))
        .collect::<Vec<_>>();

    quote! {
        static #constant_name: [&'static str; 6] = [#(#tags,)*];

        fn #function_name(level: u8) -> &'static str {
            debug_assert!((1u8..6u8).contains(&level), "Heading level must be in the range [1,6]");
            &#constant_name[level as usize - 1]
        }
    }
    .into()
}
