use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{format_ident, quote_spanned};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, LitByteStr, Token, Type};

#[proc_macro_derive(FromSyntax)]
pub fn derive_from_syntax(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote_spanned! { proc_macro2::Span::call_site() =>
        impl crate::syntax::FromSyntax for #name {
            fn from_syntax(syntax: crate::syntax::SyntaxNode) -> Self {
                Self { syntax }
            }
        }
    }
    .into()
}

struct AstNodeField {
    name: Ident,
    optional: bool,
    ty: Type,
}
impl Parse for AstNodeField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let optional = input.parse::<Token![?]>().is_ok();
        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;
        Ok(AstNodeField { name, optional, ty })
    }
}

struct AstNodeDefinitionInput {
    name: Ident,
    fields: Vec<AstNodeField>,
}
impl Parse for AstNodeDefinitionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let fields = input
            .parse_terminated(AstNodeField::parse, Token![,])?
            .into_iter()
            .collect();
        Ok(AstNodeDefinitionInput { name, fields })
    }
}

#[proc_macro]
pub fn ast_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AstNodeDefinitionInput);

    let name = &input.name;

    let accessors = input
        .fields
        .iter()
        .enumerate()
        .map(|(slot, field)| {
            let AstNodeField { name, optional, ty } = &field;
            let type_name = ty.span().source_text().unwrap();
            let mapper = if type_name == "Token" {
                if *optional {
                    format_ident!("optional_token")
                } else {
                    format_ident!("required_token")
                }
            } else {
                if *optional {
                    format_ident!("optional_node")
                } else {
                    format_ident!("required_node")
                }
            };
            quote_spanned! { field.name.span() =>
                fn #name(&self) -> #ty {
                    #mapper(&self.syntax, #slot)
                }
            }
        })
        .collect::<Vec<_>>();

    let expanded = quote_spanned! { proc_macro2::Span::call_site() =>
        #[derive(Clone, Debug)]
        struct #name {
            syntax: crate::syntax::SyntaxNode,
        }

        impl #name {
            pub fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }

            #(#accessors)*
        }

        impl FromSyntax for #name {
            fn from_syntax(syntax: crate::syntax::SyntaxNode) -> Self {
                Self { syntax }
            }
        }
    };
    TokenStream::from(expanded)
}

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
#[proc_macro]
pub fn generate_byte_lookup_table(input: TokenStream) -> TokenStream {
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
