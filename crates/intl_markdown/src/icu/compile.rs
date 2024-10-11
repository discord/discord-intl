//! Compiling for ICU messages is like serialization, but with a mono-morphed structure that
//! succinctly represents the AST in a way that is explicitly compatible with FormatJS. The output
//! of this compilation should match _exactly_ with FormatJS's `@formatjs/cli compile --ast` when
//! serialized to JSON. However, this format also allows more compact representations like keyless
//! JSON or even binary formats.
use serde::ser::SerializeMap;
use serde::{self, Serialize, Serializer};

use crate::ast::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate, IcuNumber,
    IcuPlural, IcuPluralArm, IcuPluralKind, IcuSelect, IcuTime, IcuVariable, InlineContent, Link,
    Paragraph, Strikethrough, Strong, TextOrPlaceholder,
};
use crate::icu::serialize::FormatJsElementType;
use crate::icu::tags::DEFAULT_TAG_NAMES;

/// Compile a parsed ICU-Markdown document into a FormatJS Node tree, that can then be directly
/// serialized to any format and back with any other FormatJS-compatible tools.
pub fn compile_to_format_js(document: &Document) -> FormatJsNode {
    FormatJsNode::from(document)
}

/// A mono-morphed type capable of representing any node in an ICU tree following the FormatJS JSON
/// structure. The ordering of these fields is explicitly done to match FormatJS's serialization and
/// allow for minified, structured serialization without field names.
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum FormatJsNode<'a> {
    Literal(&'a str),
    SingleNode(FormatJsSingleNode<'a>),
    ListNode(Vec<FormatJsNode<'a>>),
}

impl<'a> FormatJsNode<'a> {
    fn list(values: Vec<FormatJsNode<'a>>) -> Self {
        Self::from(values)
    }

    fn literal(value: &'a str) -> Self {
        Self::Literal(value)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Serialize)]
pub struct FormatJsSingleNode<'a> {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<FormatJsElementType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Box<FormatJsNode<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<FormatJsNodeOptions<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<usize>,
    #[serde(rename = "pluralType", skip_serializing_if = "Option::is_none")]
    pub plural_type: Option<IcuPluralKind>,
}

impl<'a> FormatJsSingleNode<'a> {
    fn tag(tag_name: &'a str) -> Self {
        Self::default()
            .with_type(FormatJsElementType::Tag)
            .with_value(tag_name.into())
    }

    fn variable(name: &'a str) -> Self {
        Self::default()
            .with_type(FormatJsElementType::Argument)
            .with_value(name)
    }

    /// Create an "empty" node, an ICU variable what will should always resolve to an empty string
    /// at runtime and can be used as a marker to separate adjacent text nodes, such as in links
    /// with static destinations.
    fn empty() -> Self {
        Self::variable("$_")
    }

    fn with_type(mut self, ty: FormatJsElementType) -> Self {
        self.ty = Some(ty);
        self
    }

    fn with_value(mut self, value: &'a str) -> Self {
        self.value = Some(value);
        self
    }

    fn with_children(mut self, children: FormatJsNode<'a>) -> Self {
        self.children = Some(Box::new(children));
        self
    }

    fn with_options(mut self, options: FormatJsNodeOptions<'a>) -> Self {
        self.options = Some(options);
        self
    }

    fn with_style(mut self, style: &'a str) -> Self {
        self.style = Some(style);
        self
    }

    fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    fn with_plural_type(mut self, plural_type: IcuPluralKind) -> Self {
        self.plural_type = Some(plural_type);
        self
    }
}

impl<'a> From<FormatJsSingleNode<'a>> for FormatJsNode<'a> {
    fn from(value: FormatJsSingleNode<'a>) -> Self {
        FormatJsNode::SingleNode(value)
    }
}

impl<'a> From<Vec<FormatJsNode<'a>>> for FormatJsNode<'a> {
    fn from(value: Vec<FormatJsNode<'a>>) -> Self {
        FormatJsNode::ListNode(value)
    }
}

//#region Serialization

#[derive(Debug, Eq, PartialEq)]
pub struct FormatJsNodeOptions<'a>(&'a Vec<IcuPluralArm>);
impl Serialize for FormatJsNodeOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut arms = serializer.serialize_map(Some(self.0.len()))?;
        for arm in self.0 {
            arms.serialize_entry(arm.selector(), &FormatJsNode::from(arm.content()))?;
        }
        arms.end()
    }
}

//#endregion

//#region AST to Node conversions
impl<'a> From<&'a str> for FormatJsNode<'a> {
    fn from(value: &'a str) -> Self {
        FormatJsNode::Literal(value)
    }
}
impl<'a> From<&'a String> for FormatJsNode<'a> {
    fn from(value: &'a String) -> Self {
        FormatJsNode::Literal(value)
    }
}

impl<'a> From<&'a Document> for FormatJsNode<'a> {
    fn from(value: &'a Document) -> Self {
        match value.blocks().get(0) {
            // For Documents with a single InlineContent segment, the value shouldn't get wrapped
            // with another list node. Otherwise, the output is like `[["content"]]`.
            Some(BlockNode::InlineContent(content)) if value.blocks().len() == 1 => {
                FormatJsNode::from(content)
            }
            _ => FormatJsNode::list(value.blocks().iter().map(FormatJsNode::from).collect()),
        }
    }
}

impl<'a> From<&'a InlineContent> for FormatJsNode<'a> {
    fn from(value: &'a InlineContent) -> Self {
        match value {
            InlineContent::Text(text) => FormatJsNode::literal(text),
            InlineContent::Emphasis(emphasis) => FormatJsNode::from(emphasis),
            InlineContent::Strong(strong) => FormatJsNode::from(strong),
            InlineContent::Link(link) => FormatJsNode::from(link),
            InlineContent::CodeSpan(code_span) => FormatJsNode::from(code_span),
            InlineContent::HardLineBreak => FormatJsSingleNode::tag(DEFAULT_TAG_NAMES.br())
                .with_children(FormatJsNode::list(vec![]))
                .into(),
            InlineContent::Hook(hook) => FormatJsNode::from(hook),
            InlineContent::Strikethrough(strikethrough) => FormatJsNode::from(strikethrough),
            InlineContent::Icu(icu) => FormatJsNode::from(icu),
            InlineContent::IcuPound => FormatJsSingleNode::default()
                .with_type(FormatJsElementType::Pound)
                .into(),
        }
    }
}

impl<'a> From<&'a Vec<InlineContent>> for FormatJsNode<'a> {
    fn from(value: &'a Vec<InlineContent>) -> Self {
        FormatJsNode::list(value.iter().map(FormatJsNode::from).collect())
    }
}

macro_rules! impl_from_for_tag_node {
    ($struct:ident, $tag:expr, $method:ident) => {
        impl<'a> From<&'a $struct> for FormatJsNode<'a> {
            fn from(value: &'a $struct) -> Self {
                FormatJsSingleNode::tag($tag)
                    .with_children(value.$method().into())
                    .into()
            }
        }
    };
}

impl_from_for_tag_node!(CodeBlock, DEFAULT_TAG_NAMES.code_block(), content);
impl_from_for_tag_node!(Paragraph, DEFAULT_TAG_NAMES.paragraph(), content);
impl_from_for_tag_node!(Emphasis, DEFAULT_TAG_NAMES.emphasis(), content);
impl_from_for_tag_node!(Strong, DEFAULT_TAG_NAMES.strong(), content);
impl_from_for_tag_node!(Strikethrough, DEFAULT_TAG_NAMES.strike_through(), content);

impl<'a> From<&'a CodeSpan> for FormatJsNode<'a> {
    fn from(value: &'a CodeSpan) -> Self {
        FormatJsSingleNode::tag(DEFAULT_TAG_NAMES.code())
            .with_children(FormatJsNode::ListNode(vec![FormatJsNode::literal(
                value.content(),
            )]))
            .into()
    }
}

impl<'a> From<&'a Heading> for FormatJsNode<'a> {
    fn from(value: &'a Heading) -> Self {
        FormatJsSingleNode::tag(DEFAULT_TAG_NAMES.heading(value.level()))
            .with_children(value.content().into())
            .into()
    }
}

fn compile_link_children<'a>(
    destination: &'a TextOrPlaceholder,
    label: &'a Vec<InlineContent>,
) -> FormatJsNode<'a> {
    let destination = match destination {
        TextOrPlaceholder::Text(text) => {
            vec![
                FormatJsNode::literal(text),
                FormatJsSingleNode::empty().into(),
            ]
        }
        TextOrPlaceholder::Placeholder(icu) => vec![FormatJsNode::from(icu)],
        TextOrPlaceholder::Handler(handler_name) => {
            vec![FormatJsSingleNode::variable(handler_name).into()]
        }
    };

    let mut children = Vec::with_capacity(destination.len() + label.len());
    children.extend(destination);
    children.extend(label.iter().map(FormatJsNode::from));
    FormatJsNode::list(children)
}

impl<'a> From<&'a Link> for FormatJsNode<'a> {
    fn from(value: &'a Link) -> Self {
        FormatJsSingleNode::tag(DEFAULT_TAG_NAMES.link())
            .with_children(compile_link_children(value.destination(), value.label()))
            .into()
    }
}

impl<'a> From<&'a BlockNode> for FormatJsNode<'a> {
    fn from(value: &'a BlockNode) -> Self {
        match value {
            BlockNode::Paragraph(paragraph) => FormatJsNode::from(paragraph),
            BlockNode::Heading(heading) => FormatJsNode::from(heading),
            BlockNode::CodeBlock(code_block) => FormatJsNode::from(code_block),
            BlockNode::InlineContent(inline_content) => FormatJsNode::from(inline_content),
            BlockNode::ThematicBreak => FormatJsSingleNode::tag(DEFAULT_TAG_NAMES.hr())
                .with_children(FormatJsNode::list(vec![]))
                .into(),
        }
    }
}

impl<'a> From<&'a Hook> for FormatJsNode<'a> {
    fn from(value: &'a Hook) -> Self {
        FormatJsSingleNode::tag(value.name())
            .with_children(value.content().into())
            .into()
    }
}

impl<'a> From<&'a Icu> for FormatJsNode<'a> {
    fn from(value: &'a Icu) -> Self {
        match value {
            Icu::IcuVariable(variable) => FormatJsNode::from(variable),
            Icu::IcuPlural(plural) => FormatJsNode::from(plural),
            Icu::IcuSelect(select) => FormatJsNode::from(select),
            Icu::IcuDate(date) => FormatJsNode::from(date),
            Icu::IcuTime(time) => FormatJsNode::from(time),
            Icu::IcuNumber(number) => FormatJsNode::from(number),
        }
    }
}

macro_rules! impl_from_for_icu_type {
    ($struct:ident, $ty:expr) => {
        impl<'a> From<&'a $struct> for FormatJsNode<'a> {
            fn from(value: &'a $struct) -> Self {
                let mut node = FormatJsSingleNode::default()
                    .with_type($ty)
                    .with_value(value.name());
                if value.style().is_some() {
                    node = node.with_style(value.style().as_ref().unwrap().text());
                }
                node.into()
            }
        }
    };
}

impl_from_for_icu_type!(IcuDate, FormatJsElementType::Date);
impl_from_for_icu_type!(IcuTime, FormatJsElementType::Time);
impl_from_for_icu_type!(IcuNumber, FormatJsElementType::Number);

impl<'a> From<&'a IcuPlural> for FormatJsNode<'a> {
    fn from(value: &'a IcuPlural) -> Self {
        FormatJsSingleNode::default()
            .with_type(FormatJsElementType::Plural)
            .with_value(value.name())
            .with_options(FormatJsNodeOptions(value.arms()))
            // TODO: Implement offset in parsing
            .with_offset(0)
            .with_plural_type(*value.kind())
            .into()
    }
}

impl<'a> From<&'a IcuSelect> for FormatJsNode<'a> {
    fn from(value: &'a IcuSelect) -> Self {
        FormatJsSingleNode::default()
            .with_type(FormatJsElementType::Select)
            .with_value(value.name())
            .with_options(FormatJsNodeOptions(value.arms()))
            .into()
    }
}

impl<'a> From<&'a IcuVariable> for FormatJsNode<'a> {
    fn from(value: &'a IcuVariable) -> Self {
        FormatJsSingleNode::variable(value.name()).into()
    }
}

//#endregion

#[cfg(test)]
mod tests {
    use crate::icu::tags::DEFAULT_TAG_NAMES;
    use crate::parse_intl_message;

    use super::{compile_to_format_js, FormatJsElementType, FormatJsNode, FormatJsSingleNode};

    fn assert_formatjs_with_blocks(
        input_str: &str,
        expected_node: &FormatJsNode,
        include_blocks: bool,
    ) {
        let parsed = parse_intl_message(input_str, include_blocks);
        let compiled = compile_to_format_js(&parsed);
        assert_eq!(
            serde_json::to_string(&compiled).unwrap(),
            serde_json::to_string(expected_node).unwrap()
        );
    }

    fn assert_formatjs(input_str: &str, expected_node: &FormatJsNode) {
        assert_formatjs_with_blocks(input_str, expected_node, false)
    }

    macro_rules! tag {
        ($name:expr, [$($content:expr),* $(,)*]) => {
            FormatJsSingleNode::tag($name).with_children(FormatJsNode::from(vec![$($content.into()),*]))
        }

    }

    macro_rules! empty {
        () => {
            FormatJsSingleNode::empty()
        };
    }

    macro_rules! lit {
        ($name:literal) => {
            FormatJsNode::Literal($name)
        };
    }

    macro_rules! var {
        ($name:literal) => {
            FormatJsSingleNode::variable($name)
        };
        ($name:literal, $ty:ident) => {
            FormatJsSingleNode::variable($name).with_type(FormatJsElementType::$ty)
        };
    }

    macro_rules! list {
        ($($item:expr),+ $(,)*) => {
            FormatJsNode::ListNode(vec![$($item.into()),*])
        }
    }

    #[test]
    fn document_no_nesting() {
        assert_formatjs("some content", &list!(lit!("some content")));
    }

    #[test]
    fn style_tags() {
        assert_formatjs(
            "***emphasized** italicized*",
            &list!(tag!(
                DEFAULT_TAG_NAMES.emphasis(),
                [
                    tag!(DEFAULT_TAG_NAMES.strong(), [lit!("emphasized")]),
                    lit!(" italicized")
                ]
            )),
        );
    }

    #[test]
    fn links() {
        let doc = parse_intl_message("[a *link*](./somewhere.png)", false);
        let compiled = compile_to_format_js(&doc);
        // Asserting that the destination is placed as the first child, then all the label content.
        assert_eq!(
            compiled,
            list!(tag!(
                DEFAULT_TAG_NAMES.link(),
                [
                    lit!("./somewhere.png"),
                    empty!(),
                    lit!("a "),
                    tag!(DEFAULT_TAG_NAMES.emphasis(), ["link"])
                ]
            ))
        )
    }

    #[test]
    fn icu_variables() {
        assert_formatjs("{username}", &list!(var!("username")));
        assert_formatjs("{startDate, date}", &list!(var!("startDate", Date)));
        assert_formatjs(
            "{startDate, date, medium}",
            &list!(var!("startDate", Date).with_style("medium")),
        );
        assert_formatjs("{postedAt, time}", &list!(var!("postedAt", Time)));
        assert_formatjs(
            "{postedAt, time, ::hmsGy  }",
            &list!(var!("postedAt", Time).with_style("::hmsGy")),
        );

        assert_formatjs("{price, number}", &list!(var!("price", Number)));
        assert_formatjs(
            "{price, number,   ::.## sign-always currency/USD }",
            &list!(var!("price", Number).with_style("::.## sign-always currency/USD")),
        );
    }

    #[test]
    fn paragraph_text() {
        assert_formatjs_with_blocks(
            "this paragraph has words",
            &list!(tag!(
                DEFAULT_TAG_NAMES.paragraph(),
                [lit!("this paragraph has words")]
            )),
            true,
        );

        assert_formatjs_with_blocks(
            r#"this paragraph has words

and another paragraph here   includes multiple spaces"#,
            &list!(
                tag!(
                    DEFAULT_TAG_NAMES.paragraph(),
                    [lit!("this paragraph has words")]
                ),
                tag!(
                    DEFAULT_TAG_NAMES.paragraph(),
                    [lit!(
                        "and another paragraph here   includes multiple spaces"
                    )]
                )
            ),
            true,
        )
    }
}
