use crate::syntax::TextPointer;

macro_rules! compiled_element_from_node {
    (Markdown, $name:ident, $node:ident) => {
        impl From<$node> for CompiledElement {
            fn from(node: $node) -> CompiledElement {
                CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::$name(node)))
            }
        }
    };
    (Icu, $name:ident, $node:ident) => {
        impl From<$node> for CompiledElement {
            fn from(node: $node) -> CompiledElement {
                CompiledElement::Node(CompiledNode::Icu(IcuNode::$name(node)))
            }
        }
    };
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CompiledElement {
    BlockList(Box<[CompiledElement]>),
    List(Box<[CompiledElement]>),
    Node(CompiledNode),
    Literal(TextPointer),
}

impl From<TextPointer> for CompiledElement {
    fn from(pointer: TextPointer) -> Self {
        CompiledElement::Literal(pointer)
    }
}

impl From<CompiledNode> for CompiledElement {
    fn from(node: CompiledNode) -> Self {
        CompiledElement::Node(node)
    }
}

impl From<Box<[CompiledElement]>> for CompiledElement {
    fn from(list: Box<[CompiledElement]>) -> Self {
        CompiledElement::List(list)
    }
}

impl From<MarkdownNode> for CompiledElement {
    fn from(node: MarkdownNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(node.into()))
    }
}

impl From<IcuNode> for CompiledElement {
    fn from(node: IcuNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(node.into()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CompiledNode {
    Markdown(MarkdownNode),
    Icu(IcuNode),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MarkdownNode {
    Paragraph(ParagraphNode),
    CodeBlock(CodeBlockNode),
    Heading(HeadingNode),
    ThematicBreak,
    // Inline Nodes
    LineBreak,
    Strong(StrongNode),
    Emphasis(EmphasisNode),
    Strikethrough(StrikethroughNode),
    Code(CodeNode),
    Link(LinkNode),
    Hook(HookNode),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ParagraphNode {
    pub content: Box<CompiledElement>,
}
impl ParagraphNode {
    pub fn new(content: impl Into<CompiledElement>) -> Self {
        Self {
            content: Box::from(content.into()),
        }
    }
}
compiled_element_from_node!(Markdown, Paragraph, ParagraphNode);
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CodeBlockNode {
    pub info_string: Option<TextPointer>,
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, CodeBlock, CodeBlockNode);
impl CodeBlockNode {
    pub fn new(
        info_string: impl Into<Option<TextPointer>>,
        content: impl Into<CompiledElement>,
    ) -> Self {
        Self {
            info_string: info_string.into(),
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HeadingNode {
    pub level: u8,
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, Heading, HeadingNode);
impl HeadingNode {
    pub fn new(level: u8, content: impl Into<CompiledElement>) -> Self {
        Self {
            level,
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StrongNode {
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, Strong, StrongNode);
impl StrongNode {
    pub fn new(content: impl Into<CompiledElement>) -> Self {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EmphasisNode {
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, Emphasis, EmphasisNode);
impl EmphasisNode {
    pub fn new(content: impl Into<CompiledElement>) -> Self {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StrikethroughNode {
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, Strikethrough, StrikethroughNode);
impl StrikethroughNode {
    pub fn new(content: impl Into<CompiledElement>) -> Self {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CodeNode {
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, Code, CodeNode);
impl CodeNode {
    pub fn new(content: impl Into<CompiledElement>) -> Self {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LinkNode {
    pub kind: LinkKind,
    pub destination: LinkDestination,
    pub title: Option<TextPointer>,
    pub alt: Option<TextPointer>,
    pub content: Option<Box<CompiledElement>>,
}
compiled_element_from_node!(Markdown, Link, LinkNode);
impl LinkNode {
    pub fn new(
        kind: LinkKind,
        destination: LinkDestination,
        title: Option<TextPointer>,
        alt: Option<TextPointer>,
        content: impl Into<Option<CompiledElement>>,
    ) -> Self {
        Self {
            kind,
            destination,
            title,
            alt,
            content: content.into().map(Box::from),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HookNode {
    pub name: TextPointer,
    pub content: Box<CompiledElement>,
}
compiled_element_from_node!(Markdown, Hook, HookNode);
impl HookNode {
    pub fn new(name: TextPointer, content: impl Into<CompiledElement>) -> Self {
        Self {
            name,
            content: Box::from(content.into()),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum LinkKind {
    Link,
    Image,
    Email,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum LinkDestination {
    Static(TextPointer),
    Dynamic(IcuNode),
    Handler(ArgumentNode),
    Empty,
}

// This ordering is roughly meant to be the same as FormatJS
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum IcuNode {
    Argument(ArgumentNode),
    Number(NumberNode),
    Date(DateNode),
    Time(TimeNode),
    Selectable(SelectableNode),
    Pound,
}

// TODO: Make this a `try_from`.
impl From<CompiledElement> for IcuNode {
    fn from(value: CompiledElement) -> Self {
        match value {
            CompiledElement::Node(CompiledNode::Icu(node)) => node,
            t => panic!("Converting {t:?} to IcuNode is not possible"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ArgumentNode {
    pub name: TextPointer,
}
compiled_element_from_node!(Icu, Argument, ArgumentNode);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NumberNode {
    pub name: TextPointer,
    pub style: Option<TextPointer>,
}
compiled_element_from_node!(Icu, Number, NumberNode);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DateNode {
    pub name: TextPointer,
    pub style: Option<TextPointer>,
}
compiled_element_from_node!(Icu, Date, DateNode);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TimeNode {
    pub name: TextPointer,
    pub style: Option<TextPointer>,
}
compiled_element_from_node!(Icu, Time, TimeNode);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SelectKind {
    Select,
    SelectOrdinal,
    Plural,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SelectableNode {
    pub name: TextPointer,
    pub kind: SelectKind,
    pub offset: Option<TextPointer>,
    pub options: Box<[IcuOption]>,
}
compiled_element_from_node!(Icu, Selectable, SelectableNode);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IcuOption {
    pub name: TextPointer,
    pub value: Box<CompiledElement>,
}
