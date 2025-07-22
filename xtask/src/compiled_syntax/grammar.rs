use crate::util::as_ident;
use std::str::FromStr;
use ungrammar::{Grammar, NodeData, Rule, TokenData};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Field {
    /// String representation of the root type of the value, like
    /// `Name: CompiledElement*` would have a value type of just
    /// `CompiledElement`, and the rest of the struct would set `is_list` to
    /// true and `is_boxed` to true.
    value_type: Option<String>,
    /// Name used for the variant name as an enum field.
    name: Option<String>,
    /// Whether the field needs to be wrapped in an `Option`.
    is_optional: bool,
    /// Whether the field needs to be wrapped in a `Box` (always true when
    /// the value_type includes a `CompiledElement`.
    is_boxed: bool,
    /// Whether the field is a list of the named value type.
    is_list: bool,
}

impl Field {
    pub fn new(value_type: String, name: Option<String>) -> Self {
        let requires_box = value_type == "CompiledElement";
        let value_type = if value_type == "Empty" {
            None
        } else {
            Some(value_type)
        };

        Self {
            value_type,
            name,
            is_optional: false,
            is_boxed: requires_box,
            is_list: false,
        }
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn ident(&self) -> Option<proc_macro2::Ident> {
        self.name.as_ref().map(|name| as_ident(&name))
    }

    pub fn value_type(&self) -> Option<&String> {
        self.value_type.as_ref()
    }

    pub fn is_optional(&self) -> bool {
        self.is_optional
    }

    pub fn is_boxed(&self) -> bool {
        self.is_boxed
    }

    pub fn is_list(&self) -> bool {
        self.is_list
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_value_type(mut self, value_type: Option<String>) -> Self {
        self.value_type = value_type.and_then(|value_type| {
            if value_type == "Empty" {
                None
            } else {
                Some(value_type)
            }
        });
        self
    }

    pub fn with_optional(mut self, is_optional: bool) -> Self {
        self.is_optional = is_optional;
        self
    }

    pub fn with_boxed(mut self, is_boxed: bool) -> Self {
        self.is_boxed = is_boxed;
        self
    }

    pub fn with_list(mut self, is_list: bool) -> Self {
        self.is_list = is_list;
        self.is_boxed = is_list || self.is_boxed;
        self
    }

    pub fn variant_ident(&self) -> proc_macro2::Ident {
        as_ident(
            self.name
                .as_ref()
                .or(self.value_type.as_ref())
                .expect("Requesting a variant type should have either a name or a root value type"),
        )
    }

    pub fn complete_type_string(&self) -> String {
        let mut constructed_type = self.value_type.clone().unwrap_or("()".into());
        if self.is_list {
            constructed_type.insert(0, '[');
            constructed_type.push(']');
        }
        if self.is_boxed {
            constructed_type.insert_str(0, "Box<");
            constructed_type.push('>');
        }
        if self.is_optional {
            constructed_type.insert_str(0, "Option<");
            constructed_type.push('>');
        }
        constructed_type
    }

    pub fn complete_type(&self) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::from_str(&self.complete_type_string()).unwrap()
    }
}

pub enum CompiledGrammarNode {
    Struct(CompiledStructNode),
    List(CompiledListNode),
    Enum(CompiledEnumNode),
}

impl CompiledGrammarNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        match self {
            CompiledGrammarNode::Struct(node) => node.ident(),
            CompiledGrammarNode::List(node) => node.ident(),
            CompiledGrammarNode::Enum(node) => node.ident(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            CompiledGrammarNode::Struct(node) => &node.name,
            CompiledGrammarNode::List(node) => &node.name,
            CompiledGrammarNode::Enum(node) => &node.name,
        }
    }
}

pub struct CompiledStructNode {
    pub name: String,
    pub fields: Vec<Field>,
}

impl CompiledStructNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }
}

pub struct CompiledListNode {
    pub name: String,
    pub inner: Field,
}

impl CompiledListNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }
}

pub struct CompiledEnumNode {
    pub name: String,
    pub variants: Vec<Field>,
}

impl CompiledEnumNode {
    pub fn ident(&self) -> proc_macro2::Ident {
        as_ident(&self.name)
    }
}

pub(super) fn parse_struct_node(node: &NodeData, grammar: &Grammar) -> CompiledGrammarNode {
    let fields = match &node.rule {
        Rule::Seq(rules) => rules
            .iter()
            .map(|rule| get_single_node_field(rule, grammar))
            .collect(),
        rule => vec![get_single_node_field(rule, grammar)],
    };

    CompiledGrammarNode::Struct(CompiledStructNode {
        name: node.name.clone(),
        fields,
    })
}

pub(super) fn parse_list_node(node: &NodeData, grammar: &Grammar) -> CompiledGrammarNode {
    let Rule::Rep(rule) = &node.rule else {
        panic!("Repetition node {:?} should only be an Alternation", node)
    };

    CompiledGrammarNode::List(CompiledListNode {
        name: node.name.clone(),
        inner: get_single_node_field(&rule, grammar),
    })
}

pub(super) fn parse_enum_node(node: &NodeData, grammar: &Grammar) -> CompiledGrammarNode {
    let Rule::Alt(rules) = &node.rule else {
        panic!("Enum node {:?} should only be an Alternation", node);
    };

    CompiledGrammarNode::Enum(CompiledEnumNode {
        name: node.name.clone(),
        variants: get_enum_variants(rules, grammar),
    })
}

fn get_primitive_field(token: &TokenData) -> Field {
    match token.name.as_str() {
        "u8" | "u16" | "u32" | "usize" => Field::new(token.name.clone(), None),
        _ => panic!("Non-primitive Token values are not supported in the Compiled grammar"),
    }
}

fn get_single_node_field(rule: &Rule, grammar: &Grammar) -> Field {
    match rule {
        Rule::Labeled { label, rule } => {
            get_single_node_field(rule, grammar).with_name(Some(label.clone()))
        }
        Rule::Node(node) => Field::new(grammar[*node].name.clone(), None),
        Rule::Opt(rule) => get_single_node_field(rule, grammar).with_optional(true),
        Rule::Rep(rule) => get_single_node_field(rule, grammar).with_list(true),
        Rule::Token(token) => get_primitive_field(&grammar[*token]),
        Rule::Seq(_) => panic!("Single nodes cannot contain sequences"),
        Rule::Alt(_) => panic!("Single nodes cannot contain alternations"),
    }
}

fn get_enum_variants(rules: &Vec<Rule>, grammar: &Grammar) -> Vec<Field> {
    rules
        .into_iter()
        .map(|rule| {
            let Rule::Labeled { label, rule } = rule else {
                panic!("Enum node alternates must all be labeled nodes");
            };

            get_single_node_field(rule, grammar).with_name(Some(label.clone()))
        })
        .collect()
}
