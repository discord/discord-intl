use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, LitByte, LitByteStr, LitStr, Token};

struct ByteTableArm {
    name: Ident,
    array: LitByteStr,
}

impl Parse for ByteTableArm {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=>]>()?;
        let array = input.parse::<LitByteStr>()?;
        Ok(Self { name, array })
    }
}

struct GenerateByteLookupTableInput {
    table_name: Ident,
    enum_name: Ident,
    arms: Vec<ByteTableArm>,
}

impl Parse for GenerateByteLookupTableInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            panic!("At least a name must be specified for an ascii lookup table");
        }

        let table_name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let enum_name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let arms = Punctuated::<ByteTableArm, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(GenerateByteLookupTableInput {
            table_name,
            enum_name,
            arms,
        })
    }
}

/// Generate an ASCII Lookup Table where each byte of the given string in the
/// table are marked as true and everything else is false. The table will be
/// assigned to a new static constant with the given name.
///
/// ```ignore
/// generate_byte_lookup_table!(WHITESPACE, b"\n\r \t");
/// ```
pub fn generate_byte_lookup_table_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GenerateByteLookupTableInput);

    // Validate that no categories overlap
    let values = &mut [0u8; 256];
    let mut enum_fields = vec![Ident::new("PLAIN", proc_macro2::Span::mixed_site())];
    let mut enum_values = vec![0];
    let mut type_value = 1;
    for ByteTableArm { name, array } in input.arms {
        let name_span = name.span();
        enum_fields.push(name);
        enum_values.push(type_value);
        for byte in array.value() {
            if values[byte as usize] != 0 {
                let char_value = char::from(byte);
                return TokenStream::from(
                    syn::Error::new(
                        name_span,
                        format!(
                            "byte '{char_value}' ({byte}) was given for two different categories"
                        ),
                    )
                    .to_compile_error(),
                );
            }
            values[byte as usize] = type_value;
        }
        type_value *= 2;
    }
    enum_fields.push(Ident::new("UNICODE", proc_macro2::Span::mixed_site()));
    enum_values.push(type_value);
    for byte in 128u8..=255 {
        values[byte as usize] = type_value;
    }

    let table_name = input.table_name;
    let enum_name = input.enum_name;
    let method_names = enum_fields
        .iter()
        .map(|field| format_ident!("is_{}", field.to_string().to_lowercase()))
        .collect::<Vec<_>>();

    let expanded = quote_spanned! { proc_macro2::Span::call_site() =>
        pub(crate) static #table_name: [u8; 256] = [#(#values),*];

        #[repr(u8)]
        #[allow(non_camel_case_types)]
        pub(crate) enum #enum_name {
            #(#enum_fields = #enum_values),*
        }

        impl From<u8> for #enum_name {
            fn from(byte: u8) -> Self {
                match byte {
                    #(#enum_values => #enum_name::#enum_fields),*,
                    _ => panic!("Invalid ByteType value given: {byte}")
                }
            }
        }

        impl #enum_name {
            #(pub(crate) fn #method_names(byte: u8) -> bool {
                #table_name[byte as usize] & #enum_values > 0u8
            })*
        }
    };

    TokenStream::from(expanded)
}

enum CharMapping {
    SingleByte { byte: u8, replacement: LitStr },
    ByteString { bytes: Vec<u8>, pattern: LitStr },
}

impl Parse for CharMapping {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitByte) {
            let byte = input.parse::<LitByte>()?;
            input.parse::<Token![=>]>()?;
            let replacement = input.parse::<LitStr>()?;
            Ok(CharMapping::SingleByte {
                byte: byte.value(),
                replacement,
            })
        } else if lookahead.peek(LitByteStr) || lookahead.peek(Token![!]) {
            let is_inverted = lookahead.peek(Token![!]);
            if is_inverted {
                input.parse::<Token![!]>()?;
            };
            let bytes = input.parse::<LitByteStr>()?;
            input.parse::<Token![=>]>()?;
            let replacement = input.parse::<LitStr>()?;

            let resolved_bytes = if is_inverted {
                let negative_bytes = bytes.value();
                (0..=255u8)
                    .filter(|byte| !negative_bytes.contains(byte))
                    .collect::<Vec<_>>()
            } else {
                bytes.value()
            };
            Ok(CharMapping::ByteString {
                bytes: resolved_bytes,
                pattern: replacement,
            })
        } else {
            Err(input.error("Expected a byte literal or a byte string mapping for replacement"))
        }
    }
}

impl CharMapping {
    fn to_replacements(&self) -> Vec<(usize, String)> {
        match self {
            CharMapping::SingleByte { byte, replacement } => {
                vec![(*byte as usize, replacement.value())]
            }
            CharMapping::ByteString { bytes, pattern } => {
                let mut mappings = vec![];
                let pattern = pattern.value();
                for &byte in bytes {
                    match pattern.as_str() {
                        "%{:X}" => {
                            mappings.push((byte as usize, format!("%{:02X}", byte)));
                        }
                        p => panic!("Unknown value pattern syntax {p}. Support is very limited. Edit the macro to support new patterns"),
                    }
                }
                mappings
            }
        }
    }
}

struct AsciiEncodingTableInput {
    name: Ident,
    mappings: Vec<CharMapping>,
}

impl Parse for AsciiEncodingTableInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let table_name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let mappings = input.parse_terminated(CharMapping::parse, Token![,])?;

        Ok(AsciiEncodingTableInput {
            name: table_name,
            mappings: mappings.into_iter().collect(),
        })
    }
}

pub fn generate_ascii_encoding_table_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AsciiEncodingTableInput);

    let replacement_table_name = format_ident!("{}_REPLACEMENT_TABLE", input.name);
    let mappings = input
        .mappings
        .iter()
        .flat_map(|mapping| mapping.to_replacements());

    let mut replacements: [Option<String>; 256] = [const { None }; 256];

    for (index, replacement) in mappings {
        replacements[index] = Some(replacement);
    }

    let replacement_values = replacements.map(|value| match value {
        Some(value) => quote! { Some(#value) },
        None => quote! {  None },
    });
    quote! {
        static #replacement_table_name: [Option<&'static str>; 256] = [
            #(#replacement_values),*
        ];
    }
    .into()
}
