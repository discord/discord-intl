use proc_macro::TokenStream;

use convert_case::{Case, Casing};
use proc_macro2::Ident;
use quote::{format_ident, quote_spanned};
use syn::{Data, DataEnum, DataStruct, DeriveInput, LitByteStr, parse_macro_input, Token, Type};
use syn::parse::{Parse, ParseStream};

#[proc_macro_derive(ReadFromEvents)]
pub fn derive_read_from_events(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // get the name of the type we want to implement the trait for
    let name = &input.ident;

    let syntax_kind = format_ident!("{}", name.to_string().to_case(Case::UpperSnake));

    match input.data {
        Data::Struct(data) => derive_read_for_struct(name, syntax_kind, data),
        Data::Enum(data) => derive_read_for_enum(name, syntax_kind, data),
        _ => panic!("ReadFromEvents is only applicable on structs"),
    }
}

fn derive_read_for_enum(name: &Ident, _: Ident, data: DataEnum) -> TokenStream {
    let variants = data.variants;
    let variant_idents = Vec::from_iter(variants.iter().map(|variant| &variant.ident));

    let syntax_names = Vec::from_iter(
        variant_idents
            .iter()
            .map(|ident| ident.to_string())
            .map(|name| format_ident!("{}", name.to_string().to_case(Case::UpperSnake))),
    );

    let boilerplate_impls = quote_spanned! { proc_macro2::Span::call_site() =>
        impl #name {
            pub fn kind(&self) -> SyntaxKind {
                match self {
                    #(#name::#variant_idents(_) => SyntaxKind::#syntax_names),*
                }
            }
        }

        impl crate::tree_builder::TokenSpan for #name {
            fn first_token(&self) -> Option<&Token> {
                match self {
                    #(#name::#variant_idents(v) => v.first_token(),)*
                }
            }

            fn last_token(&self) -> Option<&Token> {
                match self {
                    #(#name::#variant_idents(v) => v.last_token(),)*
                }
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(Self::#variant_idents(v) => v.fmt(f)),*
                }
            }
        }
    };

    let expanded = quote_spanned! { proc_macro2::Span::call_site() =>
        impl ReadFromEventBuf for #name {
            fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self {
                let start = buf.peek();
                let Some(Event::Start(start_kind)) = start else {
                    unreachable!("Encountered an event other than Start when reading a node");
                };

                let node = match start_kind {
                    SyntaxKind::TOMBSTONE => unreachable!(
                        "Tried to parse a real event, but encountered a tombstone (an abandoned event)"
                    ),
                    #(SyntaxKind::#syntax_names => #name::#variant_idents(#variant_idents::read_from(buf)),)*
                    kind => unreachable!(
                        "Expected parsed buffer to have a valid node kind, but got {:?}",
                        kind
                    ),
                };

                node
            }

            fn matches_kind(kind: SyntaxKind) -> bool {
                matches!(kind, #(SyntaxKind::#syntax_names)|*)
            }
        }

        #boilerplate_impls
    };

    TokenStream::from(expanded)
}

fn derive_read_for_struct(name: &Ident, syntax_kind: Ident, data: DataStruct) -> TokenStream {
    let fields = data.fields;

    let mut readers = vec![];
    let mut assigners = vec![];
    let mut accessors = vec![];

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();

        let kind = match &field.ty {
            Type::Path(path) => path
                .path
                .get_ident()
                .unwrap_or(&path.path.segments[0].ident),
            _ => panic!("ReadFromEvents only supports Path types"),
        };
        let reader = quote_spanned! { kind.span() =>
            let #field_name = #kind::read_from(buf);
        };

        let assigner = quote_spanned! { field_name.span() => #field_name };
        let accessor = quote_spanned! { field_name.span() => self.#field_name };

        readers.push(reader);
        assigners.push(assigner);
        accessors.push(accessor);
    }

    let reverse_accessors = accessors.iter().rev();

    let boilerplate_impls = quote_spanned! { proc_macro2::Span::call_site() =>
        impl crate::tree_builder::TokenSpan for #name {
            fn first_token(&self) -> Option<&Token> {
                #(if let Some(token) = #accessors.first_token() {
                    return Some(token);
                };)*
                None
            }

            fn last_token(&self) -> Option<&Token> {
                #(if let Some(token) = #reverse_accessors.last_token() {
                    return Some(token);
                };)*
                None
            }
        }
    };

    let expanded = quote_spanned! { proc_macro2::Span::call_site() =>
        impl crate::tree_builder::ReadFromEventBuf for #name {
            const KIND: SyntaxKind = SyntaxKind::#syntax_kind;

            fn read_from<I: Iterator<Item = Event>>(buf: &mut EventBuffer<I>) -> Self {
                buf.next_as_start();
                #(#readers)*
                buf.next_as_finish(Self::KIND);

                Self {
                    #(#assigners),*
                }
            }
        }

        #boilerplate_impls
    };

    TokenStream::from(expanded)
}

struct GenerateAsciiLookupTableInput {
    ident: Ident,
    array: LitByteStr,
}

impl Parse for GenerateAsciiLookupTableInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            panic!("At least a name must be specified for an ascii lookup table");
        }

        let ident = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let array = input.parse::<LitByteStr>()?;

        Ok(GenerateAsciiLookupTableInput { ident, array })
    }
}

/// Generate an ASCII Lookup Table where each byte of the given string in the
/// table are marked as true and everything else is false. The table will be
/// assigned to a new static constant with the given name.
///
/// ```ignore
/// generate_ascii_lookup_table!(WHITESPACE, b"\n\r \t");
/// ```
#[proc_macro]
pub fn generate_ascii_lookup_table(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as GenerateAsciiLookupTableInput);

    let name = input.ident;

    let values = &mut [0u8; 256];
    for valid in input.array.value() {
        values[valid as usize] = 1u8;
    }

    let expanded = quote_spanned! { proc_macro2::Span::call_site() =>
        static #name: [u8; 256] = [#(#values),*];
    };

    TokenStream::from(expanded)
}
