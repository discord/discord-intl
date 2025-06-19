use crate::util::as_ident;
use heck::{ToShoutySnakeCase, ToSnakeCase};
use quote::{format_ident, quote, ToTokens};
use std::fmt::Formatter;
use ungrammar::{Grammar, NodeData, Rule, TokenData};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ElementKind {
    Token,
    Node(String),
}

impl ElementKind {
    pub fn is_node(&self) -> bool {
        matches!(self, ElementKind::Node(_))
    }
}

impl std::fmt::Display for ElementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementKind::Token => write!(f, "SyntaxToken"),
            ElementKind::Node(name) => f.write_str(&name),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GrammarField {
    pub name: String,
    pub return_kind: ElementKind,
    pub optional: bool,
    pub slot: usize,
}

impl GrammarField {
    fn new(name: String, return_kind: ElementKind, optional: bool, slot: usize) -> Self {
        Self {
            name,
            return_kind,
            optional,
            slot,
        }
    }

    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }

    pub fn return_kind_ident(&self) -> proc_macro2::Ident {
        as_ident(&self.return_kind.to_string())
    }

    pub fn accessor_impl(&self) -> impl ToTokens {
        let method = match (self.optional, &self.return_kind) {
            (true, ElementKind::Token) => quote! { support::optional_token },
            (false, ElementKind::Token) => quote! { support::required_token },
            (true, ElementKind::Node(_)) => quote! { support::optional_node },
            (false, ElementKind::Node(_)) => quote! { support::required_node },
        };
        let slot = self.slot;
        quote! { #method(&self.syntax, #slot) }
    }

    pub fn accessor_ident(&self) -> proc_macro2::Ident {
        let name = match self.return_kind {
            ElementKind::Token => {
                if self.name == "token" {
                    "token".into()
                } else {
                    format!("{}_token", &self.name.to_snake_case())
                }
            }
            ElementKind::Node(_) => self.name.to_snake_case().into(),
        };
        format_ident!("{}", name)
    }

    pub fn slot_name(&self) -> String {
        format!("[{}] {}", self.slot, self.accessor_ident())
    }

    pub fn return_ty(&self) -> proc_macro2::TokenStream {
        let ty = self.return_kind_ident();
        if self.optional {
            quote!(Option<#ty>)
        } else {
            quote!(#ty)
        }
    }
}

pub enum AnyGrammarNode {
    Struct(GrammarStructNode),
    List(GrammarListNode),
    Enum(GrammarEnumNode),
}

impl AnyGrammarNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        match self {
            AnyGrammarNode::Struct(node) => node.ident(),
            AnyGrammarNode::List(node) => node.ident(),
            AnyGrammarNode::Enum(node) => node.ident(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AnyGrammarNode::Struct(node) => &node.name,
            AnyGrammarNode::List(node) => &node.name,
            AnyGrammarNode::Enum(node) => &node.name,
        }
    }

    pub fn method_name(&self) -> String {
        self.name().to_snake_case()
    }
}

pub struct GrammarStructNode {
    pub name: String,
    pub fields: Vec<GrammarField>,
}

impl GrammarStructNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }
}

pub struct GrammarListNode {
    pub name: String,
    pub kind: ElementKind,
}

impl GrammarListNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }
}

pub struct GrammarEnumNode {
    pub name: String,
    pub variants: Vec<GrammarVariant>,
}

impl GrammarEnumNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }

    pub fn variant_idents(&self) -> Vec<proc_macro2::Ident> {
        self.variants.iter().map(|v| v.variant_ident()).collect()
    }

    pub fn variant_definitions(&self) -> Vec<proc_macro2::TokenStream> {
        self.variants.iter().map(|v| v.definition()).collect()
    }

    pub fn syntax_mappings(&self) -> Vec<proc_macro2::TokenStream> {
        self.variants
            .iter()
            .flat_map(|v| v.syntax_mappings())
            .collect()
    }
}

pub struct GrammarVariant {
    // For nested variants, this strips the `Any` prefix to make it more workable.
    pub variant_name: String,
    // The name of the node type contained by this variant.
    pub type_name: String,
    // Syntax kinds that map to this variant of the parent enum node.
    // This allows for nesting enum variants without needing another intermediate node, like:
    // `AnyBlockNode::Heading(AnyHeading::AtxHeading(AtxHeading))`
    // vs
    // `AnyBlockNode::Heading(Heading(AnyHeading::AtxHeading(AtxHeading))`.
    pub syntax_kinds: Vec<String>,
}

impl GrammarVariant {
    pub fn type_ident(&self) -> proc_macro2::Ident {
        as_ident(&self.type_name)
    }

    pub fn variant_ident(&self) -> proc_macro2::Ident {
        as_ident(&self.variant_name)
    }

    pub fn definition(&self) -> proc_macro2::TokenStream {
        let variant_name = self.variant_ident();
        let type_name = self.type_ident();
        quote! { #variant_name(#type_name) }
    }

    pub fn syntax_mappings(&self) -> Vec<proc_macro2::TokenStream> {
        let variant_name = self.variant_ident();
        let type_name = self.type_ident();

        self.syntax_kinds
            .iter()
            .cloned()
            .map(move |kind| {
                let kind = format_ident!("{}", kind);
                quote! { SyntaxKind::#kind => Self::#variant_name(#type_name::from_syntax(syntax)) }
            })
            .collect()
    }
}

pub fn syntax_from_grammar(grammar: &Grammar) -> Vec<AnyGrammarNode> {
    let mut result = vec![];
    for node_id in grammar.iter() {
        let node = &grammar[node_id];
        if node.name.starts_with("Any") {
            result.push(parse_enum_node(node, grammar));
        } else if matches!(node.rule, Rule::Rep(_)) {
            result.push(parse_list_node(node, grammar));
        } else {
            result.push(parse_struct_node(node, grammar));
        };
    }

    result
}

fn parse_struct_node(node: &NodeData, grammar: &Grammar) -> AnyGrammarNode {
    AnyGrammarNode::Struct(GrammarStructNode {
        name: node.name.clone(),
        fields: get_node_fields(&node.rule, &grammar),
    })
}

fn parse_list_node(node: &NodeData, grammar: &Grammar) -> AnyGrammarNode {
    let Rule::Rep(rule) = &node.rule else {
        panic!("Repetition node {:?} should only be an Alternation", node)
    };

    AnyGrammarNode::List(GrammarListNode {
        name: node.name.clone(),
        kind: get_single_node_kind(&rule, grammar, false),
    })
}

fn parse_enum_node(node: &NodeData, grammar: &Grammar) -> AnyGrammarNode {
    let Rule::Alt(rules) = &node.rule else {
        panic!(
            "Enum node {:?} (starting with `Any`) should only be an Alternation",
            node
        );
    };

    AnyGrammarNode::Enum(GrammarEnumNode {
        name: node.name.clone(),
        variants: get_enum_variants(rules, grammar),
    })
}

fn get_node_fields(node_rule: &Rule, grammar: &Grammar) -> Vec<GrammarField> {
    match node_rule {
        Rule::Node(node) => {
            let node_name = grammar[*node].name.clone();
            vec![GrammarField::new(
                node_name.clone(),
                ElementKind::Node(node_name),
                false,
                0,
            )]
        }
        Rule::Token(token) => vec![GrammarField::new(
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
                    Some(GrammarField::new(
                        node_name.clone(),
                        ElementKind::Node(node_name),
                        false,
                        slot,
                    ))
                }
                Rule::Token(token) => Some(GrammarField::new(
                    get_token_name(&grammar[*token]).into(),
                    ElementKind::Token,
                    false,
                    slot,
                )),
                Rule::Labeled { label, rule } => Some(GrammarField::new(
                    label.clone(),
                    get_single_node_kind(rule, grammar, true),
                    matches!(rule.as_ref(), Rule::Opt(_)),
                    slot,
                )),
                _ => None,
            })
            .collect(),
        Rule::Labeled { label, rule } => {
            vec![GrammarField::new(
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

fn get_enum_variants(rules: &Vec<Rule>, grammar: &Grammar) -> Vec<GrammarVariant> {
    let mut result = vec![];
    for rule in rules {
        let Rule::Node(node) = rule else {
            panic!("Enum node alternates must all be plain nodes");
        };
        let mut syntax_kinds = Vec::new();

        let name = grammar[*node].name.clone();

        get_syntax_kinds_from_rule(rule, grammar, &mut syntax_kinds);
        result.push(GrammarVariant {
            variant_name: name.strip_prefix("Any").map_or(name.clone(), String::from),
            type_name: name.clone(),
            syntax_kinds,
        });
    }

    result
}
