use anyhow;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::str::FromStr;
use ungrammar::Grammar;
use xtask::grammar::{
    syntax_from_grammar, AnyGrammarNode, ElementKind, GrammarEnumNode, GrammarListNode,
    GrammarStructNode,
};
use xtask::util;
use xtask::util::Codegen;

fn main() {
    try_main().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
}

fn try_main() -> anyhow::Result<()> {
    let mut codegen = Codegen::new(util::repo_root().join("crates/intl_markdown/src/cst"));

    let grammar = Grammar::from_str(include_str!("../../data/markdown.ungram"))?;
    let syntax = syntax_from_grammar(&grammar);

    codegen.write_file("nodes.rs", generate_tree_from_grammar(&syntax))?;
    codegen.write_file("visitor.rs", generate_visitor_from_grammar(&syntax))?;
    codegen.finish()
}

fn generate_visitor_from_grammar(nodes: &Vec<AnyGrammarNode>) -> String {
    let mut visit_methods = vec![];
    let mut fold_methods = vec![];
    let mut visit_with_impls = vec![];
    for node in nodes {
        let node_ident = node.ident();
        let visit_ident = format_ident!("visit_{}", node.method_name());
        let fold_ident = format_ident!("fold_{}", node.method_name());
        visit_methods.push(quote! {
            fn #visit_ident(&mut self, node: &#node_ident) { node.visit_children_with(self); }
        });
        fold_methods.push(quote! {
            fn #fold_ident(&mut self, node: #node_ident) -> #node_ident;
        });

        let visit_children_impl = match node {
            AnyGrammarNode::Struct(node) => {
                let child_nodes = node
                    .fields
                    .iter()
                    .filter(|node| node.return_kind.is_node())
                    .collect::<Vec<_>>();
                let accessor_idents = child_nodes.iter().map(|f| f.accessor_ident());
                if child_nodes.len() == 0 {
                    quote! { let _ = visitor; }
                } else {
                    quote! {
                        #(self.#accessor_idents().visit_with(visitor);)*
                    }
                }
            }
            AnyGrammarNode::List(node) => {
                // Only node lists visit their children. Token lists are leaf nodes.
                match &node.kind {
                    ElementKind::Token => quote! {
                        let _ = visitor;
                    },
                    ElementKind::Node(_) => {
                        quote! {
                            for field in self.children() {
                                field.visit_with(visitor);
                            }
                        }
                    }
                }
            }
            AnyGrammarNode::Enum(node) => {
                let variants = node.variant_idents();
                quote! {
                    match self {
                        #(Self::#variants(node) => node.visit_with(visitor),)*
                    }
                }
            }
        };

        visit_with_impls.push(quote! {
            impl<V: ?Sized + Visit> VisitWith<V> for #node_ident {
                fn visit_with(&self, visitor: &mut V) {
                    visitor.#visit_ident(self);
                }

                fn visit_children_with(&self, visitor: &mut V) {
                    #visit_children_impl
                }
            }
        });
    }

    quote! {
        use super::nodes::*;


        pub trait Visit {
            #(#visit_methods)*
        }

        pub trait Fold {
            #(#fold_methods)*
        }

        pub trait VisitWith<V: ?Sized + Visit> {
            fn visit_with(&self, visitor: &mut V);
            fn visit_children_with(&self, visitor: &mut V);
        }

        impl<V: ?Sized + Visit, T: VisitWith<V>> VisitWith<V> for Option<T> {
            fn visit_with(&self, visitor: &mut V) {
                self.as_ref().map(|v| v.visit_with(visitor));
            }

            fn visit_children_with(&self, visitor: &mut V) {
                self.as_ref().map(|v| v.visit_children_with(visitor));
            }
        }

        #(#visit_with_impls)*
    }
    .to_string()
}

fn generate_tree_from_grammar(nodes: &Vec<AnyGrammarNode>) -> String {
    let mut result = quote! {
        use crate::syntax::*;
        use crate::cst::util::*;
    };
    for node in nodes {
        let node_impl = match &node {
            AnyGrammarNode::Struct(node) => impl_node(node),
            AnyGrammarNode::List(node) => impl_list_node(node),
            AnyGrammarNode::Enum(node) => impl_enum_node(node),
        };

        node_impl.to_tokens(&mut result);
    }
    result.to_string()
}

fn impl_node(node: &GrammarStructNode) -> TokenStream {
    let name_str = &node.name;
    let ident = node.ident();

    let mut accessors = quote! {};
    let mut debug_fields = quote! {};
    for field in &node.fields {
        let return_ty = field.return_ty();
        let accessor_impl = field.accessor_impl();
        let field_ident = field.accessor_ident();
        let slot_str = field.slot_name();
        quote! { .field(#slot_str, &self.#field_ident()) }.to_tokens(&mut debug_fields);
        quote! {
            pub fn #field_ident(&self) -> #return_ty {
                #accessor_impl
            }
        }
        .to_tokens(&mut accessors);
    }

    quote! {
        #[derive(Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct #ident {
            syntax: SyntaxNode
        }

        impl Syntax for #ident {
            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }

        impl FromSyntax for #ident {
            fn from_syntax(syntax: SyntaxNode) -> Self { Self { syntax } }
        }

        impl #ident {
            #accessors
        }

        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#name_str)
                    #debug_fields
                    .finish()
            }
        }
    }
}

fn impl_list_node(node: &GrammarListNode) -> TokenStream {
    let children_impl = match &node.kind {
        ElementKind::Token => {
            quote! {
                pub fn children(&self) -> SyntaxTokenChildren {
                    SyntaxTokenChildren::new(self.syntax.children())
                }
                pub fn get(&self, index: usize) -> Option<&SyntaxToken> {
                    self.syntax.get(index).map(|element| element.token())
                }
            }
        }
        ElementKind::Node(name) => {
            let node_ty = format_ident!("{}", name);
            quote! {
                pub fn children(&self) -> TypedNodeChildren<#node_ty> {
                    TypedNodeChildren::new(SyntaxNodeChildren::new(self.syntax.children()))
                }
                pub fn get(&self, index: usize) -> Option<#node_ty> {
                    self.syntax.get(index).map(|node| #node_ty::from_syntax_element(node.clone()))
                }
            }
        }
    };

    let ident = node.ident();
    let name = node.name.clone();

    quote! {
        #[derive(Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct #ident {
            syntax: SyntaxNode
        }

        impl Syntax for #ident {
            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }

        impl FromSyntax for #ident {
            fn from_syntax(syntax: SyntaxNode) -> Self { Self { syntax } }
        }

        impl #ident {
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
            #children_impl
        }

        impl std::ops::Index<usize> for #ident {
            type Output = SyntaxToken;

            fn index(&self, index: usize) -> &Self::Output {
                &self.syntax[index].token()
            }
        }

        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(#name)?;
                f.debug_list().entries(self.children()).finish()
            }
        }
    }
}

fn impl_enum_node(node: &GrammarEnumNode) -> TokenStream {
    let name = node.name.clone();
    let ident = node.ident();
    let variant_idents = node.variant_idents();
    let type_idents = node.type_idents();
    let variant_defs = node.variant_definitions();
    let syntax_mappings = node.syntax_mappings();

    quote! {
        #[derive(Clone, Eq, PartialEq)]
        pub enum #ident {
            #(#variant_defs),*
        }

        impl Syntax for #ident {
            fn syntax(&self) -> &SyntaxNode {
                match self {
                    #(Self::#variant_idents(node) => node.syntax()),*
                }
            }
        }

        impl FromSyntax for #ident {
            fn from_syntax(syntax: SyntaxNode) -> Self {
                match syntax.kind() {
                    #(#syntax_mappings,)*
                    kind => unreachable!("Invalid syntax kind {:?} encountered when constructing enum node {}", kind, #name)
                }
            }
        }

        #(impl From<#type_idents> for #ident {
            fn from(value: #type_idents) -> Self {
                Self::#variant_idents(value)
            }
        })*

        impl std::fmt::Debug for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut tuple = f.debug_tuple(#name);
                match self {
                    #(Self::#variant_idents(node) => tuple.field(node)),*
                };
                tuple.finish()
            }
        }
    }
}
