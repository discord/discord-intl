use anyhow;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use quote::{format_ident, quote, ToTokens};
use std::fmt::Formatter;
use std::str::FromStr;
use ungrammar::{Grammar, NodeData, Rule, TokenData};
use xshell::{cmd, Shell};

fn main() {
    try_main().unwrap_or_else(|e| eprintln!("{}", e));
}

fn try_main() -> anyhow::Result<()> {
    let repo_root = std::path::Path::new(
        &std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf();

    let grammar = Grammar::from_str(include_str!("../../data/markdown.ungram"))?;
    let generated = generate_tree_from_grammar(&grammar);

    let ast_nodes_path = repo_root.join("crates/intl_markdown/src/cst/nodes.rs");
    println!("{:?}", ast_nodes_path);
    std::fs::write(&ast_nodes_path, generated)?;
    let shell = Shell::new()?;
    cmd!(shell, "cargo fmt -- {ast_nodes_path}").run()?;
    Ok(())
}

fn generate_tree_from_grammar(grammar: &Grammar) -> String {
    let mut result = quote! {
        use crate::syntax::*;
        use crate::cst::util::*;
    };
    for node in grammar.iter() {
        let node = &grammar[node];
        if node.name.starts_with("Any") {
            impl_enum_node(node, grammar).to_tokens(&mut result)
        } else if matches!(node.rule, Rule::Rep(_)) {
            impl_list_node(node, grammar).to_tokens(&mut result)
        } else {
            impl_node(node, grammar).to_tokens(&mut result)
        };
    }
    result.to_string()
}

fn impl_node(node: &NodeData, grammar: &Grammar) -> impl ToTokens {
    let name_str = &node.name;
    let name = format_ident!("{}", &node.name);
    let fields = get_node_fields(&node.rule, &grammar);

    let mut accessors = quote! {};
    let mut debug_fields = quote! {};
    for field in fields {
        let field_name = field.field_name();
        let return_ty = {
            let ty = format_ident!("{}", field.return_kind.to_string());
            if field.optional {
                quote!(Option<#ty>)
            } else {
                quote!(#ty)
            }
        };
        let accessor_impl = field.accessor_impl();
        let field_ident = format_ident!("{}", field_name);
        let slot_str = format!("[{}] {}", field.slot, field_name);
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
        pub struct #name {
            syntax: SyntaxNode
        }

        impl Syntax for #name {
            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }

        impl FromSyntax for #name {
            fn from_syntax(syntax: SyntaxNode) -> Self { Self { syntax } }
        }

        impl #name {
            #accessors
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#name_str)
                    #debug_fields
                    .finish()
            }
        }
    }
}

fn impl_list_node(node: &NodeData, grammar: &Grammar) -> impl ToTokens {
    let name_str = &node.name;
    let name = format_ident!("{}", name_str);

    let Rule::Rep(rule) = &node.rule else {
        panic!("Repetition node {:?} should only be an Alternation", node)
    };

    let kind = get_single_node_kind(&rule, grammar, false);
    let children_impl = match &kind {
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

    quote! {
        #[derive(Clone, Eq, PartialEq)]
        #[repr(transparent)]
        pub struct #name {
            syntax: SyntaxNode
        }

        impl Syntax for #name {
            fn syntax(&self) -> &SyntaxNode {
                &self.syntax
            }
        }

        impl FromSyntax for #name {
            fn from_syntax(syntax: SyntaxNode) -> Self { Self { syntax } }
        }

        impl #name {
            pub fn len(&self) -> usize {
                self.syntax.len()
            }
            #children_impl
        }

        impl std::ops::Index<usize> for #name {
            type Output = SyntaxToken;

            fn index(&self, index: usize) -> &Self::Output {
                &self.syntax[index].token()
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_list().entries(self.children()).finish()
            }
        }
    }
}

fn impl_enum_node(node: &NodeData, grammar: &Grammar) -> impl ToTokens {
    let node_str = &node.name;
    let name = format_ident!("{}", node_str);
    let Rule::Alt(rules) = &node.rule else {
        panic!(
            "Enum node {:?} (starting with `Any`) should only be an Alternation",
            node
        );
    };

    let variants = get_enum_variant_map(rules, grammar);
    let variant_idents = variants
        .iter()
        .map(|v| format_ident!("{}", v.variant_name))
        .collect::<Vec<_>>();
    let variant_defs = variants
        .iter()
        .map(|v| {
            let name = format_ident!("{}", &v.variant_name);
            let value = format_ident!("{}", &v.name);
            quote! { #name(#value) }
        })
        .collect::<Vec<_>>();
    let syntax_mappings = variants.iter().flat_map(|v| {
        let variant_name = format_ident!("{}", v.variant_name);
        let type_name = format_ident!("{}", v.name);
        v.syntax_kinds.iter().cloned().map(move |kind| {
            let kind = format_ident!("{}", kind);
            quote! { SyntaxKind::#kind => Self::#variant_name(#type_name::from_syntax(syntax)) }
        })
    });

    quote! {
        #[derive(Clone, Eq, PartialEq)]
        pub enum #name {
            #(#variant_defs),*
        }

        impl Syntax for #name {
            fn syntax(&self) -> &SyntaxNode {
                match self {
                    #(Self::#variant_idents(node) => node.syntax()),*
                }
            }
        }

        impl FromSyntax for #name {
            fn from_syntax(syntax: SyntaxNode) -> Self {
                match syntax.kind() {
                    #(#syntax_mappings,)*
                    kind => unreachable!("Invalid syntax kind {:?} encountered when constructing enum node {}", kind, #node_str)
                }
            }
        }

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut tuple = f.debug_tuple(#node_str);
                match self {
                    #(Self::#variant_idents(node) => tuple.field(node)),*
                };
                tuple.finish()
            }
        }
    }
}

struct NodeVariant {
    name: String,
    // For nested variants, this strips the `Any` prefix to make it more workable.
    variant_name: String,
    // Syntax kinds that map to this variant of the parent enum node.
    // This allows for nesting enum variants without needing another intermediate node, like:
    // `AnyBlockNode::Heading(AnyHeading::AtxHeading(AtxHeading))`
    // vs
    // `AnyBlockNode::Heading(Heading(AnyHeading::AtxHeading(AtxHeading))`.
    syntax_kinds: Vec<String>,
}

fn get_enum_variant_map(rules: &Vec<Rule>, grammar: &Grammar) -> Vec<NodeVariant> {
    let mut result = vec![];
    for rule in rules {
        let Rule::Node(node) = rule else {
            panic!("Enum node alternates must all be plain nodes");
        };
        let mut syntax_kinds = Vec::new();

        let name = grammar[*node].name.clone();

        get_syntax_kinds_from_rule(rule, grammar, &mut syntax_kinds);
        result.push(NodeVariant {
            name: name.clone(),
            variant_name: name.strip_prefix("Any").map_or(name.clone(), String::from),
            syntax_kinds,
        });
    }

    result
}

/// Recursively collect all SyntaxKind names that can be used to construct the given Rule. This
/// traverses through nested `Any*` nodes to collect all possible kinds for the given rule.
fn get_syntax_kinds_from_rule(rule: &Rule, grammar: &Grammar, result: &mut Vec<String>) {
    match rule {
        Rule::Node(node) => {
            let node = &grammar[*node];
            if node.name.starts_with("Any") {
                get_syntax_kinds_from_rule(&node.rule, grammar, result);
            } else {
                result.push(node.name.to_shouty_snake_case());
            }
        }
        Rule::Token(token) => result.push(get_syntax_kind_from_token(&grammar[*token])),
        Rule::Alt(alts) => {
            for alt in alts {
                get_syntax_kinds_from_rule(alt, grammar, result);
            }
        }
        Rule::Labeled { rule: inner, .. } => {
            get_syntax_kinds_from_rule(inner, grammar, result);
        }
        Rule::Seq(_) => panic!("Cannot get constructable syntax names from a rule sequence."),
        Rule::Opt(_) => panic!("Cannot get constructable syntax names from an optional rule."),
        Rule::Rep(node) => get_syntax_kinds_from_rule(node, grammar, result),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ElementKind {
    Token,
    Node(String),
}

impl std::fmt::Display for ElementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementKind::Token => write!(f, "SyntaxToken"),
            ElementKind::Node(name) => f.write_str(&name),
        }
    }
}

struct AstField {
    name: String,
    return_kind: ElementKind,
    optional: bool,
    slot: usize,
}

impl AstField {
    fn new(name: String, return_kind: ElementKind, optional: bool, slot: usize) -> Self {
        Self {
            name,
            return_kind,
            optional,
            slot,
        }
    }

    fn accessor_impl(&self) -> impl ToTokens {
        let method = match (self.optional, &self.return_kind) {
            (true, ElementKind::Token) => quote! { support::optional_token },
            (false, ElementKind::Token) => quote! { support::required_token },
            (true, ElementKind::Node(_)) => quote! { support::optional_node },
            (false, ElementKind::Node(_)) => quote! { support::required_node },
        };
        let slot = self.slot;
        quote! { #method(&self.syntax, #slot) }
    }

    fn field_name(&self) -> String {
        match self.return_kind {
            ElementKind::Token => {
                if self.name == "token" {
                    "token".into()
                } else {
                    format!("{}_token", &self.name.to_snake_case())
                }
            }
            ElementKind::Node(_) => self.name.to_snake_case().into(),
        }
    }
}

fn get_node_fields(node_rule: &Rule, grammar: &Grammar) -> Vec<AstField> {
    match node_rule {
        Rule::Node(node) => {
            let node_name = grammar[*node].name.clone();
            vec![AstField::new(
                node_name.clone(),
                ElementKind::Node(node_name),
                false,
                0,
            )]
        }
        Rule::Token(token) => vec![AstField::new(
            get_token_name(&grammar[*token]).to_string(),
            ElementKind::Token,
            false,
            0,
        )],
        Rule::Seq(rules) => rules
            .iter()
            .enumerate()
            .filter_map(|(slot, rule)| match rule {
                Rule::Node(node) => {
                    let node_name = grammar[*node].name.clone();
                    Some(AstField::new(
                        node_name.clone(),
                        ElementKind::Node(node_name),
                        false,
                        slot,
                    ))
                }
                Rule::Token(token) => Some(AstField::new(
                    get_token_name(&grammar[*token]).into(),
                    ElementKind::Token,
                    false,
                    slot,
                )),
                Rule::Labeled { label, rule } => Some(AstField::new(
                    label.clone(),
                    get_single_node_kind(rule, grammar, true),
                    matches!(rule.as_ref(), Rule::Opt(_)),
                    slot,
                )),
                _ => None,
            })
            .collect(),
        Rule::Labeled { label, rule } => {
            vec![AstField::new(
                label.clone(),
                get_single_node_kind(rule, grammar, true),
                matches!(rule.as_ref(), Rule::Opt(_)),
                0,
            )]
        }
        _ => vec![],
    }
}

fn get_token_name(token: &TokenData) -> &str {
    match token.name.as_str() {
        ">" => "r_angle",
        "<" => "l_angle",
        "!" => "exclaim",
        "#" => "hash",
        "~" => "tilde",
        "`" => "backtick",
        "'" => "quote",
        "\"" => "double_quote",
        "[" => "l_square",
        "]" => "r_square",
        "(" => "l_paren",
        ")" => "r_paren",
        "{" => "l_curly",
        "}" => "r_curly",
        value => value,
    }
}

fn get_syntax_kind_from_token(token: &TokenData) -> String {
    match token.name.as_str() {
        ">" => "RANGLE".into(),
        "<" => "LANGLE".into(),
        "!" => "EXCLAIM".into(),
        "#" => "HASH".into(),
        "~" => "TILDE".into(),
        "`" => "BACKTICK".into(),
        "'" => "QUOTE".into(),
        "\"" => "DOUBLE_QUOTE".into(),
        "[" => "LSQUARE".into(),
        "]" => "RSQUARE".into(),
        "(" => "LPAREN".into(),
        ")" => "RPAREN".into(),
        "{" => "LCURLY".into(),
        "}" => "RCURLY".into(),
        value => value.to_shouty_snake_case(),
    }
}

fn get_single_node_kind(rule: &Rule, grammar: &Grammar, allow_optional: bool) -> ElementKind {
    match rule {
        Rule::Node(node) => ElementKind::Node(grammar[*node].name.clone()),
        Rule::Token(_) => ElementKind::Token,
        Rule::Alt(alts) => {
            let kinds = alts
                .iter()
                .map(|a| get_single_node_kind(a, grammar, allow_optional))
                .collect::<Vec<_>>();
            if !kinds.windows(2).all(|pair| pair[0] == pair[1]) {
                panic!("All elements in a repetition must be equal");
            }
            kinds[0].clone()
        }
        Rule::Opt(rule) => {
            if allow_optional {
                get_single_node_kind(rule, grammar, allow_optional)
            } else {
                panic!("Single repetitions cannot be made of optional nodes")
            }
        }
        Rule::Seq(_) => panic!("Single repetitions cannot be made of node sequences"),
        Rule::Labeled { .. } => panic!("Single repetitions cannot be made of labeled nodes"),
        Rule::Rep(_) => panic!("Single repetitions cannot be nested"),
    }
}
