use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};

use crate::ast::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate, IcuNumber,
    IcuPlural, IcuPluralArm, IcuPluralKind, IcuSelect, IcuTime, IcuVariable, InlineContent, Link,
    LinkDestination, Paragraph, Strikethrough, Strong,
};
use crate::icu::tags::DEFAULT_TAG_NAMES;

/// The order of these types matches the order that FormatJS serializes in. This ordering is
/// important when using keyless-json serialization, and represents the expected order that struct
/// fields will be encountered when deserializing.
pub(super) mod fjs_types {
    pub(crate) static TYPE: &str = "type";
    pub(crate) static VALUE: &str = "value";
    pub(crate) static CHILDREN: &str = "children";
    /// Custom extension to FormatJS' AST allowing for meta information on nodes, like destinations
    /// for links.
    pub(crate) static CONTROL: &str = "control";
    pub(crate) static OPTIONS: &str = "options";
    pub(crate) static STYLE: &str = "style";
    pub(crate) static OFFSET: &str = "offset";
    pub(crate) static PLURAL_TYPE: &str = "pluralType";
}

impl Serialize for IcuPluralKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            IcuPluralKind::Plural => "cardinal",
            IcuPluralKind::SelectOrdinal => "ordinal",
        })
    }
}

/// Enum matching a type of element to it's FormatJS type number. The order defines the numbering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum FormatJsElementType {
    Literal = 0,
    Argument,
    Number,
    Date,
    Time,
    Select,
    Plural,
    Pound,
    Tag,
}

impl Serialize for FormatJsElementType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[inline(always)]
fn serialize_literal<S: Serializer>(serializer: S, value: &str) -> Result<S::Ok, S::Error> {
    let mut literal = serializer.serialize_struct("Literal", 2)?;
    literal.serialize_field(fjs_types::TYPE, &FormatJsElementType::Literal)?;
    literal.serialize_field(fjs_types::VALUE, value)?;
    literal.end()
}

#[inline(always)]
fn serialize_tag<S: Serializer, T: Serialize>(
    serializer: S,
    name: &str,
    content: &T,
) -> Result<S::Ok, S::Error> {
    let mut tag = serializer.serialize_struct("Tag", 3)?;
    tag.serialize_field(fjs_types::TYPE, &FormatJsElementType::Tag)?;
    tag.serialize_field(fjs_types::VALUE, name)?;
    tag.serialize_field(fjs_types::CHILDREN, content)?;
    tag.end()
}

struct SerializeHandler<'a>(&'a String);
impl Serialize for SerializeHandler<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut variable = serializer.serialize_struct("IcuVariable", 2)?;
        variable.serialize_field(fjs_types::TYPE, &FormatJsElementType::Argument)?;
        variable.serialize_field(fjs_types::VALUE, self.0)?;
        variable.end()
    }
}

impl Serialize for Document {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut root = serializer.serialize_seq(Some(self.blocks().len()))?;
        for block in self.blocks() {
            match block {
                BlockNode::Paragraph(paragraph) => root.serialize_element(&paragraph)?,
                BlockNode::Heading(heading) => root.serialize_element(&heading)?,
                BlockNode::CodeBlock(code_block) => root.serialize_element(&code_block)?,
                BlockNode::ThematicBreak => root.serialize_element(&"<hr />")?,
                BlockNode::InlineContent(content) => {
                    for element in content {
                        root.serialize_element(&element)?
                    }
                }
            }
        }

        root.end()
    }
}

macro_rules! tag_serializer {
    ($struct:ident, $tag:expr, $method:ident) => {
        impl Serialize for $struct {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serialize_tag(serializer, $tag, self.$method())
            }
        }
    };
}

tag_serializer!(CodeBlock, DEFAULT_TAG_NAMES.code_block(), content);
tag_serializer!(Paragraph, DEFAULT_TAG_NAMES.paragraph(), content);
tag_serializer!(Emphasis, DEFAULT_TAG_NAMES.emphasis(), content);
tag_serializer!(Strong, DEFAULT_TAG_NAMES.strong(), content);
tag_serializer!(Strikethrough, DEFAULT_TAG_NAMES.strike_through(), content);

impl Serialize for CodeSpan {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_tag(
            serializer,
            DEFAULT_TAG_NAMES.code(),
            &[InlineContent::Text(self.content().clone())],
        )
    }
}

impl Serialize for Heading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_tag(
            serializer,
            DEFAULT_TAG_NAMES.heading(self.level()),
            self.content(),
        )
    }
}

impl Serialize for LinkDestination {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Insert the link destination as the first child of the link element. Static (plain text)
        // destinations get serialized as a custom tag `_`, which is used as a simple separator to
        // prevent FormatJS from joining adjacent text pieces together.
        match self {
            LinkDestination::Text(text) => InlineContent::Text(text.clone()).serialize(serializer),
            LinkDestination::Placeholder(icu) => icu.serialize(serializer),
            LinkDestination::Handler(handler_name) => {
                SerializeHandler(&handler_name).serialize(serializer)
            }
        }
    }
}

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut link = serializer.serialize_struct("Link", 3)?;
        link.serialize_field(fjs_types::TYPE, &FormatJsElementType::Tag)?;
        link.serialize_field(fjs_types::VALUE, DEFAULT_TAG_NAMES.link())?;
        link.serialize_field(fjs_types::CHILDREN, &self.label())?;
        link.serialize_field(fjs_types::CONTROL, self.destination())?;
        link.end()
    }
}

impl Serialize for InlineContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            InlineContent::Text(text) => serialize_literal(serializer, &text),
            InlineContent::Emphasis(emphasis) => emphasis.serialize(serializer),
            InlineContent::Strong(strong) => strong.serialize(serializer),
            InlineContent::Link(link) => link.serialize(serializer),
            InlineContent::CodeSpan(code_span) => code_span.serialize(serializer),
            InlineContent::HardLineBreak => serialize_tag(serializer, DEFAULT_TAG_NAMES.br(), &()),
            InlineContent::Hook(hook) => hook.serialize(serializer),
            InlineContent::Strikethrough(strikethrough) => strikethrough.serialize(serializer),
            InlineContent::Icu(icu) => icu.serialize(serializer),
            InlineContent::IcuPound => {
                let mut pound = serializer.serialize_struct("IcuPound", 1)?;
                pound.serialize_field(fjs_types::TYPE, &FormatJsElementType::Pound)?;
                pound.end()
            }
        }
    }
}

impl Serialize for Hook {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_tag(serializer, self.name(), self.content())
    }
}

impl Serialize for Icu {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Icu::IcuVariable(variable) => variable.serialize(serializer),
            Icu::IcuPlural(plural) => plural.serialize(serializer),
            Icu::IcuSelect(select) => select.serialize(serializer),
            Icu::IcuDate(date) => date.serialize(serializer),
            Icu::IcuTime(time) => time.serialize(serializer),
            Icu::IcuNumber(number) => number.serialize(serializer),
        }
    }
}

impl Serialize for IcuVariable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut variable = serializer.serialize_struct("IcuVariable", 2)?;
        variable.serialize_field(fjs_types::TYPE, &FormatJsElementType::Argument)?;
        variable.serialize_field(fjs_types::VALUE, self.name())?;
        variable.end()
    }
}

struct SerializePluralArm<'a>(&'a IcuPluralArm);
impl Serialize for SerializePluralArm<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut arm = serializer.serialize_struct("PluralArm", 1)?;
        arm.serialize_field(fjs_types::VALUE, self.0.content())?;
        arm.end()
    }
}

struct SerializePluralArms<'a>(&'a Vec<IcuPluralArm>);
impl Serialize for SerializePluralArms<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut arms = serializer.serialize_map(Some(self.0.len()))?;
        for arm in self.0 {
            arms.serialize_entry(arm.selector(), &SerializePluralArm(arm))?;
        }
        arms.end()
    }
}

impl Serialize for IcuPlural {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut plural = serializer.serialize_struct("IcuPlural", 5)?;
        plural.serialize_field(fjs_types::TYPE, &FormatJsElementType::Plural)?;
        plural.serialize_field(fjs_types::VALUE, self.name())?;
        plural.serialize_field(fjs_types::OPTIONS, &SerializePluralArms(self.arms()))?;
        // TODO: Implement offset in parsing
        plural.serialize_field(fjs_types::OFFSET, &0)?;
        plural.serialize_field(fjs_types::PLURAL_TYPE, self.kind())?;
        plural.end()
    }
}

impl Serialize for IcuSelect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut select = serializer.serialize_struct("IcuSelect", 3)?;
        select.serialize_field(fjs_types::TYPE, &FormatJsElementType::Select)?;
        select.serialize_field(fjs_types::VALUE, self.name())?;
        select.serialize_field(fjs_types::OPTIONS, &SerializePluralArms(self.arms()))?;
        select.end()
    }
}

impl Serialize for IcuDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_style = self.style().is_some();
        let len = if has_style { 3 } else { 2 };
        let mut date = serializer.serialize_struct("IcuDate", len)?;
        date.serialize_field(fjs_types::TYPE, &FormatJsElementType::Date)?;
        date.serialize_field(fjs_types::VALUE, self.name())?;
        if let Some(style) = self.style() {
            date.serialize_field(fjs_types::STYLE, style.text())?;
        }
        date.end()
    }
}

impl Serialize for IcuTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_style = self.style().is_some();
        let len = if has_style { 3 } else { 2 };
        let mut time = serializer.serialize_struct("IcuTime", len)?;
        time.serialize_field(fjs_types::TYPE, &FormatJsElementType::Time)?;
        time.serialize_field(fjs_types::VALUE, self.name())?;
        if let Some(style) = self.style() {
            time.serialize_field(fjs_types::STYLE, style.text())?;
        }
        time.end()
    }
}

impl Serialize for IcuNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_style = self.style().is_some();
        let len = if has_style { 3 } else { 2 };
        let mut number = serializer.serialize_struct("IcuNumber", len)?;
        number.serialize_field(fjs_types::TYPE, &FormatJsElementType::Number)?;
        number.serialize_field(fjs_types::VALUE, self.name())?;
        if let Some(style) = self.style() {
            number.serialize_field(fjs_types::STYLE, style.text())?;
        }
        number.end()
    }
}
