use crate::compiled_syntax::grammar::{
    parse_enum_node, parse_list_node, parse_struct_node, CompiledEnumNode, CompiledGrammarNode,
    CompiledListNode, CompiledStructNode, Field,
};
use crate::util::{as_ident, Codegen};
use quote::{quote, ToTokens};
use std::str::FromStr;
use ungrammar::{Grammar, Rule};

pub mod grammar;

pub fn codegen_compiled_syntax(codegen: &mut Codegen, grammar: &Grammar) -> anyhow::Result<()> {
    let compiled_syntax = syntax_from_compiled_grammar(grammar);
    codegen.write_file(
        "compiler/element.rs",
        generate_tree_from_grammar(&compiled_syntax),
    )?;
    codegen.write_file(
        "compiler/visitor.rs",
        generate_visitor_from_grammar(&compiled_syntax),
    )?;
    Ok(())
}

pub fn syntax_from_compiled_grammar(grammar: &Grammar) -> Vec<CompiledGrammarNode> {
    let mut result = vec![];
    for node_id in grammar.iter() {
        let node = &grammar[node_id];
        // Intrinsics don't need to be generated.
        if node.name == "Empty" || node.name == "TextPointer" {
            continue;
        }
        let grammar_node = match &node.rule {
            Rule::Alt(_) => parse_enum_node(node, grammar),
            Rule::Rep(_) => parse_list_node(node, grammar),
            _ => parse_struct_node(node, grammar),
        };
        result.push(grammar_node);
    }

    result
}

fn generate_tree_from_grammar(grammar: &Vec<CompiledGrammarNode>) -> String {
    let mut result = quote! {
        use crate::syntax::TextPointer;
    };
    for node in grammar {
        let node_impl = match &node {
            CompiledGrammarNode::Struct(node) => impl_node(node),
            CompiledGrammarNode::List(node) => impl_list_node(node),
            CompiledGrammarNode::Enum(node) => impl_enum_node(node, grammar),
        };

        node_impl.to_tokens(&mut result);
    }
    result.to_string()
}

fn impl_node(node: &CompiledStructNode) -> proc_macro2::TokenStream {
    let ident = node.ident();

    let mut definitions = vec![];
    let mut new_parameters = vec![];
    let mut constructors = vec![];
    for field in &node.fields {
        let ty = field.complete_type();
        let name = field.variant_ident();

        definitions.push(quote! { pub #name: #ty });
        if field.value_type().is_some_and(|ty| ty == "CompiledElement")
            && !field.is_list()
            && !field.is_optional()
        {
            new_parameters.push(quote! { #name: impl Into<CompiledElement> });
            new_parameters.push(quote! { #name: impl Into<CompiledElement> });
            constructors.push(quote! { #name: Box::from(#name.into())});
        } else {
            new_parameters.push(quote! { #name: #ty });
            constructors.push(quote! { #name });
        }
    }

    quote! {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub struct #ident {
            #(#definitions),*
        }

        impl #ident {
            pub fn new(#(#new_parameters),*) -> #ident {
                Self {
                    #(#constructors),*
                }
            }
        }
    }
}

fn impl_list_node(_node: &CompiledListNode) -> proc_macro2::TokenStream {
    unimplemented!("List nodes are not yet necessary for the Compiled grammar")
}

fn impl_enum_node(
    node: &CompiledEnumNode,
    grammar: &Vec<CompiledGrammarNode>,
) -> proc_macro2::TokenStream {
    let ident = node.ident();
    let variant_defs = node.variants.iter().map(|variant| {
        let name = variant.variant_ident();
        if variant.value_type().is_some() {
            let ty = variant.complete_type();
            quote! { #name(#ty) }
        } else {
            quote! { #name }
        }
    });

    let mut from_paths = vec![];
    collect_impl_from_paths(node, grammar, vec![], &mut from_paths);

    let mut from_impls = quote! {};
    for path in from_paths {
        let (_, end_field) = path
            .last()
            .expect("From path must have at least one element");
        let from_ty = end_field.complete_type();
        if end_field.value_type().is_none() {
            continue;
        }

        let mut constructor = String::new();
        let mut closing_paren_count = 0;
        for (index, (parent, field)) in path.iter().enumerate() {
            if field.is_boxed() && index > 0 {
                constructor.push_str("Box::from(");
                closing_paren_count += 1;
            }
            constructor.push_str(&parent.to_string());
            constructor.push_str("::");
            constructor.push_str(&field.variant_ident().to_string());
            constructor.push_str("(");
            closing_paren_count += 1;
        }
        constructor.push_str("value");
        constructor.push_str(&")".repeat(closing_paren_count));
        let constructor = proc_macro2::TokenStream::from_str(&constructor).unwrap();
        quote! {
            impl From<#from_ty> for #ident {
                fn from(value: #from_ty) -> Self {
                    #constructor
                }
            }
        }
        .to_tokens(&mut from_impls)
    }

    quote! {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        pub enum #ident {
            #(#variant_defs),*
        }

        #from_impls
    }
}

fn collect_impl_from_paths(
    current_enum: &CompiledEnumNode,
    grammar: &Vec<CompiledGrammarNode>,
    current_path: Vec<(proc_macro2::Ident, Field)>,
    all_paths: &mut Vec<Vec<(proc_macro2::Ident, Field)>>,
) {
    let current_ident = as_ident(&current_enum.name);
    for variant in &current_enum.variants {
        // Special case for CompiledElement's BlockList and List since they are ambiguous and can't
        // be uniquely constructed.
        if current_enum.name == "CompiledElement"
            && variant
                .name()
                .is_some_and(|name| name == "BlockList" || name == "List")
        {
            continue;
        }
        // Also special case LinkDestination::Handler for the same reason.
        if current_enum.name == "LinkDestination"
            && variant.name().is_some_and(|name| name == "Handler")
        {
            continue;
        }
        let mut current_path = current_path.clone();
        current_path.push((current_ident.clone(), variant.clone()));
        all_paths.push(current_path.clone());

        let Some(node_name) = variant.value_type() else {
            continue;
        };
        let Some(CompiledGrammarNode::Enum(inner_enum)) =
            grammar.iter().find(|node| node.name() == node_name)
        else {
            continue;
        };
        if inner_enum.name == "CompiledElement" {
            continue;
        }

        collect_impl_from_paths(inner_enum, grammar, current_path, all_paths);
    }
}

fn generate_visitor_from_grammar(grammar: &Vec<CompiledGrammarNode>) -> String {
    "".into()
}
