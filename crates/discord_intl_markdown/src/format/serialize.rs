use crate::compiler::{
    ArgumentNode, CompiledElement, CompiledNode, IcuNode, IcuOption, LinkDestination, MarkdownNode,
    SelectableKind,
};
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};

#[derive(Clone, Copy)]
#[repr(u8)]
enum NodeTypeId {
    Argument = 1,
    Number = 2,
    Date = 3,
    Time = 4,
    Select = 5,
    Plural = 6,
    Pound = 7,
    Tag = 8,
}

impl Serialize for NodeTypeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl Serialize for CompiledElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // There's no actual difference between these anywhere other than in HTML formatting.
            CompiledElement::BlockList(elements) | CompiledElement::List(elements) => {
                let mut list = serializer.serialize_seq(Some(elements.len()))?;
                for element in elements {
                    list.serialize_element(&element)?;
                }
                list.end()
            }
            CompiledElement::Node(node) => match node {
                CompiledNode::Markdown(node) => node.serialize(serializer),
                CompiledNode::Icu(node) => node.serialize(serializer),
            },
            CompiledElement::Literal(text) => serializer.serialize_str(&text),
        }
    }
}

fn serialize_empty_tag<S: Serializer>(serializer: S, name: &str) -> Result<S::Ok, S::Error> {
    let mut tag = serializer.serialize_struct("Tag", 2)?;
    tag.serialize_field("type", &NodeTypeId::Tag)?;
    tag.serialize_field("name", &name)?;
    tag.end()
}

fn serialize_tag<S: Serializer>(
    serializer: S,
    name: &str,
    content: impl Serialize,
) -> Result<S::Ok, S::Error> {
    let mut tag = serializer.serialize_struct("Tag", 3)?;
    tag.serialize_field("type", &NodeTypeId::Tag)?;
    tag.serialize_field("name", &name)?;
    tag.serialize_field("content", &content)?;
    tag.end()
}
fn serialize_controlled_tag<S: Serializer>(
    serializer: S,
    name: &str,
    content: impl Serialize,
    control: impl Serialize,
) -> Result<S::Ok, S::Error> {
    let mut tag = serializer.serialize_struct("Tag", 4)?;
    tag.serialize_field("type", &NodeTypeId::Tag)?;
    tag.serialize_field("name", &name)?;
    tag.serialize_field("content", &content)?;
    tag.serialize_field("control", &control)?;
    tag.end()
}

const HEADER_TAGS: [&str; 6] = ["$h1", "$h2", "$h3", "$h4", "$h5", "$h6"];

impl Serialize for MarkdownNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MarkdownNode::Paragraph(paragraph) => {
                serialize_tag(serializer, "$p", &paragraph.content)
            }
            MarkdownNode::CodeBlock(block) => {
                serialize_tag(serializer, "$codeBlock", &block.content)
            }
            MarkdownNode::Heading(heading) => serialize_tag(
                serializer,
                HEADER_TAGS[heading.level as usize - 1],
                &heading.content,
            ),
            MarkdownNode::ThematicBreak(_) => serialize_empty_tag(serializer, "$hr"),
            MarkdownNode::LineBreak(_) => serialize_empty_tag(serializer, "$br"),
            MarkdownNode::Strong(strong) => serialize_tag(serializer, "$b", &strong.content),
            MarkdownNode::Emphasis(emphasis) => serialize_tag(serializer, "$i", &emphasis.content),
            MarkdownNode::Strikethrough(strikethrough) => {
                serialize_tag(serializer, "$del", &strikethrough.content)
            }
            MarkdownNode::Code(code) => serialize_tag(serializer, "$code", &code.content),
            MarkdownNode::Link(link) => {
                serialize_controlled_tag(serializer, "$link", &link.content, &link.destination)
            }
            MarkdownNode::Hook(hook) => serialize_tag(serializer, &hook.name, &hook.content),
        }
    }
}

impl Serialize for LinkDestination {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut control = serializer.serialize_seq(Some(1))?;
        match self {
            LinkDestination::Static(text) => control.serialize_element(text.as_str())?,
            LinkDestination::Dynamic(dynamic) => control.serialize_element(dynamic)?,
            LinkDestination::Handler(argument) => control.serialize_element(argument)?,
            // Empty destinations are treated as empty strings rather than actually being _empty_,
            // so that the serialized type is _always_ a string and never nullish.
            LinkDestination::Empty => control.serialize_element("")?,
        }
        control.end()
    }
}

impl Serialize for ArgumentNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut arg = serializer.serialize_struct("Argument", 2)?;
        arg.serialize_field("type", &NodeTypeId::Argument)?;
        arg.serialize_field("value", self.name.as_str())?;
        arg.end()
    }
}

impl Serialize for IcuNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            IcuNode::Argument(node) => node.serialize(serializer),
            IcuNode::Number(node) => {
                let mut number = serializer.serialize_struct("Number", 2)?;
                number.serialize_field("type", &NodeTypeId::Number)?;
                number.serialize_field("value", node.name.as_str())?;
                if let Some(style) = &node.style {
                    number.serialize_field("style", style.as_str())?;
                }
                number.end()
            }
            IcuNode::Date(node) => {
                let mut date = serializer.serialize_struct("Date", 2)?;
                date.serialize_field("type", &NodeTypeId::Date)?;
                date.serialize_field("value", node.name.as_str())?;
                if let Some(style) = &node.style {
                    date.serialize_field("style", style.as_str())?;
                }
                date.end()
            }
            IcuNode::Time(node) => {
                let mut time = serializer.serialize_struct("Time", 2)?;
                time.serialize_field("type", &NodeTypeId::Time)?;
                time.serialize_field("value", node.name.as_str())?;
                if let Some(style) = &node.style {
                    time.serialize_field("style", style.as_str())?;
                }
                time.end()
            }
            IcuNode::Selectable(selectable) => {
                let (ty, type_id, length) = match selectable.kind {
                    SelectableKind::Select => (None, NodeTypeId::Select, 3),
                    SelectableKind::Plural => (Some("cardinal"), NodeTypeId::Plural, 4),
                    SelectableKind::SelectOrdinal => (Some("ordinal"), NodeTypeId::Plural, 4),
                };
                let mut select = serializer.serialize_struct("Selectable", length)?;
                select.serialize_field("type", &type_id)?;
                select.serialize_field("value", selectable.name.as_str())?;
                select.serialize_field("options", &SerializeIcuOptions(&selectable.options))?;
                // TODO: implement `offset` properly
                if !matches!(selectable.kind, SelectableKind::Select) {
                    select.serialize_field("offset", &0)?;
                }
                if let Some(ty) = ty {
                    select.serialize_field("pluralType", ty)?;
                }
                select.end()
            }
            IcuNode::Pound(_) => {
                let mut pound = serializer.serialize_struct("Pound", 1)?;
                pound.serialize_field("type", &NodeTypeId::Pound)?;
                pound.end()
            }
        }
    }
}

struct SerializeIcuOptions<'a>(&'a [IcuOption]);

impl Serialize for SerializeIcuOptions<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut options = serializer.serialize_map(Some(self.0.len()))?;
        for option in self.0 {
            options.serialize_entry(option.name.as_str(), &option.value)?;
        }
        options.end()
    }
}
