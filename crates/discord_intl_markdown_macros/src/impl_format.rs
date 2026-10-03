use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote_spanned;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse_macro_input, Expr, ExprClosure, Token};

struct ImplFormatInput {
    trait_name: Ident,
    node_name: Ident,
    implementation: ExprClosure,
}

impl Parse for ImplFormatInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name = input.parse::<Ident>()?;
        input.parse::<Token![,]>()?;
        let node_name = input.parse::<Ident>()?;
        let implementation = input.parse::<ExprClosure>()?;
        Ok(Self {
            trait_name,
            node_name,
            implementation,
        })
    }
}

pub fn impl_format_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ImplFormatInput);

    let trait_name = input.trait_name;
    let node_name = input.node_name;
    let format_name = input.implementation.inputs.first().unwrap();
    let body = input.implementation.body;
    let body_span = body.span();
    let body = if matches!(*body, Expr::Block(_)) {
        quote_spanned! { body_span => #body }
    } else {
        quote_spanned! { body_span => { #body }}
    };

    quote_spanned! { proc_macro2::Span::call_site() =>
        impl #trait_name for #node_name {
            fn fmt(&self, #format_name: &mut impl std::fmt::Write) -> std::fmt::Result #body
        }
    }
    .into()
}
