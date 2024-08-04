//! Compiling for ICU messages is like serialization, but with a mono-morphed structure that
//! succinctly represents the AST in a way that is explicitly compatible with FormatJS. The output
//! of this compilation should match _exactly_ with FormatJS's `@formatjs/cli compile --ast` when
//! serialized to JSON. However, this format also allows more compact representations like keyless
//! JSON or even binary formats.
use serde::{self, Serialize, Serializer};
use serde::ser::SerializeMap;

use crate::ast::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate, IcuNumber,
    IcuPlural, IcuPluralArm, IcuPluralKind, IcuTime, IcuVariable, InlineContent, Link, Paragraph,
    Strikethrough, Strong, TextOrPlaceholder,
};
use crate::icu::serialize::{fjs_types, FormatJsElementType};

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
    SingleNode(FormatJsSingleNode<'a>),
    ListNode(Vec<FormatJsNode<'a>>),
}

impl<'a> FormatJsNode<'a> {
    fn list(values: Vec<FormatJsNode<'a>>) -> Self {
        Self::from(values)
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
    pub offset: Option<u64>,
    #[serde(rename = "pluralType", skip_serializing_if = "Option::is_none")]
    pub plural_type: Option<IcuPluralKind>,
}

impl<'a> FormatJsSingleNode<'a> {
    fn tag(tag_name: &'a str) -> Self {
        Self::default()
            .with_type(FormatJsElementType::Tag)
            .with_value(tag_name.into())
    }

    fn literal(value: &'a str) -> Self {
        Self::default()
            .with_type(FormatJsElementType::Literal)
            .with_value(value)
    }

    fn variable(name: &'a str) -> Self {
        Self::default()
            .with_type(FormatJsElementType::Argument)
            .with_value(name)
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

    fn with_offset(mut self, offset: u64) -> Self {
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
        // TODO(faulty): It'd be really nice to serialize options as a direct object, like:
        // `{"one":[<content>],"other":[<content>]}`, but that's not possible to do while keeping
        // perfect compatibility with FormatJS. We would want to skip the `FormatJsPluralArm`
        // serialization for the entry value here and instead just use `arm.content()` directly.
        // For now, this yields `{"one":{"value":[<content>]}}` in keyless JSON, which is close
        // enough, though the repetition of `"value"` all over the place is disappointing.
        let mut arms = serializer.serialize_map(Some(self.0.len()))?;
        for arm in self.0 {
            arms.serialize_entry(arm.selector(), &FormatJsPluralArm(arm))?;
        }
        arms.end()
    }
}

pub struct FormatJsPluralArm<'a>(&'a IcuPluralArm);
impl Serialize for FormatJsPluralArm<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut arm = serializer.serialize_map(Some(1))?;
        arm.serialize_entry(fjs_types::VALUE, &FormatJsNode::from(self.0.content()))?;
        arm.end()
    }
}
//#endregion

//#region AST to Node conversions
impl<'a> From<&'a str> for FormatJsNode<'a> {
    fn from(value: &'a str) -> Self {
        FormatJsSingleNode::literal(value).into()
    }
}
impl<'a> From<&'a String> for FormatJsNode<'a> {
    fn from(value: &'a String) -> Self {
        FormatJsSingleNode::literal(&value).into()
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
            InlineContent::Text(text) => FormatJsSingleNode::literal(&text).into(),
            InlineContent::Emphasis(emphasis) => FormatJsNode::from(emphasis),
            InlineContent::Strong(strong) => FormatJsNode::from(strong),
            InlineContent::Link(link) => FormatJsNode::from(link),
            InlineContent::CodeSpan(code_span) => FormatJsNode::from(code_span),
            InlineContent::HardLineBreak => FormatJsSingleNode::tag("br")
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
    ($struct:ident, $tag:literal, $method:ident) => {
        impl<'a> From<&'a $struct> for FormatJsNode<'a> {
            fn from(value: &'a $struct) -> Self {
                FormatJsSingleNode::tag($tag)
                    .with_children(value.$method().into())
                    .into()
            }
        }
    };
}

impl_from_for_tag_node!(CodeBlock, "codeBlock", content);
impl_from_for_tag_node!(Paragraph, "p", content);
impl_from_for_tag_node!(Emphasis, "i", content);
impl_from_for_tag_node!(Strong, "b", content);
impl_from_for_tag_node!(Strikethrough, "del", content);

impl<'a> From<&'a CodeSpan> for FormatJsNode<'a> {
    fn from(value: &'a CodeSpan) -> Self {
        FormatJsSingleNode::tag("code")
            .with_children(FormatJsNode::ListNode(vec![FormatJsSingleNode::literal(
                &value.content(),
            )
            .into()]))
            .into()
    }
}

impl<'a> From<&'a Heading> for FormatJsNode<'a> {
    fn from(value: &'a Heading) -> Self {
        let heading = match value.level() {
            1 => "h1",
            2 => "h2",
            3 => "h3",
            4 => "h4",
            5 => "h5",
            6 => "h6",
            _ => unreachable!(),
        };
        FormatJsSingleNode::tag(heading)
            .with_children(value.content().into())
            .into()
    }
}

fn compile_link_children<'a>(
    destination: &'a TextOrPlaceholder,
    label: &'a Vec<InlineContent>,
) -> FormatJsNode<'a> {
    let destination = match destination {
        TextOrPlaceholder::Text(text) => FormatJsNode::from(text),
        TextOrPlaceholder::Placeholder(icu) => FormatJsNode::from(icu),
    };

    let mut children = Vec::with_capacity(label.len() + 1);
    children.insert(0, destination.into());
    children.extend(label.iter().map(FormatJsNode::from));
    FormatJsNode::list(children)
}

impl<'a> From<&'a Link> for FormatJsNode<'a> {
    fn from(value: &'a Link) -> Self {
        FormatJsSingleNode::tag("link")
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
            BlockNode::ThematicBreak => FormatJsSingleNode::tag("hr")
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

impl<'a> From<&'a IcuVariable> for FormatJsNode<'a> {
    fn from(value: &'a IcuVariable) -> Self {
        FormatJsSingleNode::variable(value.name()).into()
    }
}

//#endregion

#[cfg(test)]
mod tests {
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
        ($name:literal, [$($content:expr),* $(,)*]) => {
            FormatJsSingleNode::tag($name).with_children(FormatJsNode::from(vec![$($content.into()),*]))
        }

    }

    macro_rules! lit {
        ($name:literal) => {
            FormatJsSingleNode::literal($name)
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
                "i",
                [tag!("b", [lit!("emphasized")]), lit!(" italicized")]
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
                "link",
                [lit!("./somewhere.png"), lit!("a "), tag!("i", ["link"])]
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
            &list!(tag!("p", [lit!("this paragraph has words")])),
            true,
        );

        assert_formatjs_with_blocks(
            r#"this paragraph has words

and another paragraph here   includes multiple spaces"#,
            &list!(
                tag!("p", [lit!("this paragraph has words")]),
                tag!(
                    "p",
                    [lit!(
                        "and another paragraph here   includes multiple spaces"
                    )]
                )
            ),
            true,
        )
    }
}
