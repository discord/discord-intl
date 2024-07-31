use serde::{Serialize, Serializer};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};

use crate::ast::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate, IcuNumber,
    IcuPlural, IcuPluralArm, IcuPluralKind, IcuTime, IcuVariable, InlineContent, Link, Paragraph,
    Strikethrough, Strong, TextOrPlaceholder,
};

/// The order of these types matches the order that FormatJS serializes in. This ordering is
/// important when using keyless-json serialization, and represents the expected order that struct
/// fields will be encountered when deserializing.
pub(super) mod fjs_types {
    pub(crate) static TYPE: &str = "type";
    pub(crate) static VALUE: &str = "value";
    pub(crate) static CHILDREN: &str = "children";
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
            _ => unreachable!(),
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
    #[allow(unused)]
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
    ($struct:ident, $tag:literal, $method:ident) => {
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

tag_serializer!(CodeBlock, "codeBlock", content);
tag_serializer!(Paragraph, "p", content);
tag_serializer!(Emphasis, "i", content);
tag_serializer!(Strong, "b", content);
tag_serializer!(Strikethrough, "del", content);

impl Serialize for CodeSpan {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_tag(
            serializer,
            "code",
            &[InlineContent::Text(self.content().clone())],
        )
    }
}

impl Serialize for Heading {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_tag(serializer, &format!("h{}", self.level()), self.content())
    }
}

struct SerializeLinkChildren<'a>(&'a TextOrPlaceholder, &'a Vec<InlineContent>);
impl Serialize for SerializeLinkChildren<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut children = serializer.serialize_seq(Some(self.1.len() + 1))?;
        // Insert the link destination as the first child of the link element.
        match self.0 {
            TextOrPlaceholder::Text(text) => {
                children.serialize_element(&InlineContent::Text(text.clone()))?
            }
            TextOrPlaceholder::Placeholder(icu) => children.serialize_element(&icu)?,
        }
        // Then add the rest of the children directly.
        for element in self.1 {
            children.serialize_element(element)?;
        }
        children.end()
    }
}

impl Serialize for Link {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut link = serializer.serialize_struct("Link", 3)?;
        link.serialize_field(fjs_types::TYPE, &FormatJsElementType::Tag)?;
        link.serialize_field(fjs_types::VALUE, "link")?;
        link.serialize_field(
            fjs_types::CHILDREN,
            &SerializeLinkChildren(self.destination(), self.label()),
        )?;
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
            InlineContent::HardLineBreak => serialize_tag(serializer, "br", &()),
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

impl Serialize for IcuDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_format = self.format().is_some();
        let len = if has_format { 3 } else { 2 };
        let mut date = serializer.serialize_struct("IcuDate", len)?;
        date.serialize_field(fjs_types::TYPE, &FormatJsElementType::Date)?;
        date.serialize_field(fjs_types::VALUE, self.name())?;
        if has_format {
            date.serialize_field(fjs_types::STYLE, self.format().as_ref().unwrap())?;
        }
        date.end()
    }
}

impl Serialize for IcuTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_format = self.format().is_some();
        let len = if has_format { 3 } else { 2 };
        let mut time = serializer.serialize_struct("IcuTime", len)?;
        time.serialize_field(fjs_types::TYPE, &FormatJsElementType::Time)?;
        time.serialize_field(fjs_types::VALUE, self.name())?;
        if has_format {
            time.serialize_field(fjs_types::STYLE, self.format().as_ref().unwrap())?;
        }
        time.end()
    }
}

impl Serialize for IcuNumber {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let has_format = self.format().is_some();
        let len = if has_format { 3 } else { 2 };
        let mut number = serializer.serialize_struct("IcuNumber", len)?;
        number.serialize_field(fjs_types::TYPE, &FormatJsElementType::Number)?;
        number.serialize_field(fjs_types::VALUE, self.name())?;
        if has_format {
            number.serialize_field(fjs_types::STYLE, self.format().as_ref().unwrap())?;
        }
        number.end()
    }
}
