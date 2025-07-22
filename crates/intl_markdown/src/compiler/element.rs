use crate::syntax::TextPointer;
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CompiledElement {
    BlockList(Box<[CompiledElement]>),
    List(Box<[CompiledElement]>),
    Node(CompiledNode),
    Literal(TextPointer),
}
impl From<CompiledNode> for CompiledElement {
    fn from(value: CompiledNode) -> Self {
        CompiledElement::Node(value)
    }
}
impl From<MarkdownNode> for CompiledElement {
    fn from(value: MarkdownNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(value))
    }
}
impl From<ParagraphNode> for CompiledElement {
    fn from(value: ParagraphNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Paragraph(value)))
    }
}
impl From<CodeBlockNode> for CompiledElement {
    fn from(value: CodeBlockNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::CodeBlock(value)))
    }
}
impl From<HeadingNode> for CompiledElement {
    fn from(value: HeadingNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Heading(value)))
    }
}
impl From<StrongNode> for CompiledElement {
    fn from(value: StrongNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Strong(value)))
    }
}
impl From<EmphasisNode> for CompiledElement {
    fn from(value: EmphasisNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Emphasis(value)))
    }
}
impl From<StrikethroughNode> for CompiledElement {
    fn from(value: StrikethroughNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Strikethrough(value)))
    }
}
impl From<CodeNode> for CompiledElement {
    fn from(value: CodeNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Code(value)))
    }
}
impl From<LinkNode> for CompiledElement {
    fn from(value: LinkNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Link(value)))
    }
}
impl From<HookNode> for CompiledElement {
    fn from(value: HookNode) -> Self {
        CompiledElement::Node(CompiledNode::Markdown(MarkdownNode::Hook(value)))
    }
}
impl From<IcuNode> for CompiledElement {
    fn from(value: IcuNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(value))
    }
}
impl From<ArgumentNode> for CompiledElement {
    fn from(value: ArgumentNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(IcuNode::Argument(value)))
    }
}
impl From<NumberNode> for CompiledElement {
    fn from(value: NumberNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(IcuNode::Number(value)))
    }
}
impl From<DateNode> for CompiledElement {
    fn from(value: DateNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(IcuNode::Date(value)))
    }
}
impl From<TimeNode> for CompiledElement {
    fn from(value: TimeNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(IcuNode::Time(value)))
    }
}
impl From<SelectableNode> for CompiledElement {
    fn from(value: SelectableNode) -> Self {
        CompiledElement::Node(CompiledNode::Icu(IcuNode::Selectable(value)))
    }
}
impl From<TextPointer> for CompiledElement {
    fn from(value: TextPointer) -> Self {
        CompiledElement::Literal(value)
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum CompiledNode {
    Markdown(MarkdownNode),
    Icu(IcuNode),
}
impl From<MarkdownNode> for CompiledNode {
    fn from(value: MarkdownNode) -> Self {
        CompiledNode::Markdown(value)
    }
}
impl From<ParagraphNode> for CompiledNode {
    fn from(value: ParagraphNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Paragraph(value))
    }
}
impl From<CodeBlockNode> for CompiledNode {
    fn from(value: CodeBlockNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::CodeBlock(value))
    }
}
impl From<HeadingNode> for CompiledNode {
    fn from(value: HeadingNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Heading(value))
    }
}
impl From<StrongNode> for CompiledNode {
    fn from(value: StrongNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Strong(value))
    }
}
impl From<EmphasisNode> for CompiledNode {
    fn from(value: EmphasisNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Emphasis(value))
    }
}
impl From<StrikethroughNode> for CompiledNode {
    fn from(value: StrikethroughNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Strikethrough(value))
    }
}
impl From<CodeNode> for CompiledNode {
    fn from(value: CodeNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Code(value))
    }
}
impl From<LinkNode> for CompiledNode {
    fn from(value: LinkNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Link(value))
    }
}
impl From<HookNode> for CompiledNode {
    fn from(value: HookNode) -> Self {
        CompiledNode::Markdown(MarkdownNode::Hook(value))
    }
}
impl From<IcuNode> for CompiledNode {
    fn from(value: IcuNode) -> Self {
        CompiledNode::Icu(value)
    }
}
impl From<ArgumentNode> for CompiledNode {
    fn from(value: ArgumentNode) -> Self {
        CompiledNode::Icu(IcuNode::Argument(value))
    }
}
impl From<NumberNode> for CompiledNode {
    fn from(value: NumberNode) -> Self {
        CompiledNode::Icu(IcuNode::Number(value))
    }
}
impl From<DateNode> for CompiledNode {
    fn from(value: DateNode) -> Self {
        CompiledNode::Icu(IcuNode::Date(value))
    }
}
impl From<TimeNode> for CompiledNode {
    fn from(value: TimeNode) -> Self {
        CompiledNode::Icu(IcuNode::Time(value))
    }
}
impl From<SelectableNode> for CompiledNode {
    fn from(value: SelectableNode) -> Self {
        CompiledNode::Icu(IcuNode::Selectable(value))
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MarkdownNode {
    Paragraph(ParagraphNode),
    CodeBlock(CodeBlockNode),
    Heading(HeadingNode),
    ThematicBreak,
    LineBreak,
    Strong(StrongNode),
    Emphasis(EmphasisNode),
    Strikethrough(StrikethroughNode),
    Code(CodeNode),
    Link(LinkNode),
    Hook(HookNode),
}
impl From<ParagraphNode> for MarkdownNode {
    fn from(value: ParagraphNode) -> Self {
        MarkdownNode::Paragraph(value)
    }
}
impl From<CodeBlockNode> for MarkdownNode {
    fn from(value: CodeBlockNode) -> Self {
        MarkdownNode::CodeBlock(value)
    }
}
impl From<HeadingNode> for MarkdownNode {
    fn from(value: HeadingNode) -> Self {
        MarkdownNode::Heading(value)
    }
}
impl From<StrongNode> for MarkdownNode {
    fn from(value: StrongNode) -> Self {
        MarkdownNode::Strong(value)
    }
}
impl From<EmphasisNode> for MarkdownNode {
    fn from(value: EmphasisNode) -> Self {
        MarkdownNode::Emphasis(value)
    }
}
impl From<StrikethroughNode> for MarkdownNode {
    fn from(value: StrikethroughNode) -> Self {
        MarkdownNode::Strikethrough(value)
    }
}
impl From<CodeNode> for MarkdownNode {
    fn from(value: CodeNode) -> Self {
        MarkdownNode::Code(value)
    }
}
impl From<LinkNode> for MarkdownNode {
    fn from(value: LinkNode) -> Self {
        MarkdownNode::Link(value)
    }
}
impl From<HookNode> for MarkdownNode {
    fn from(value: HookNode) -> Self {
        MarkdownNode::Hook(value)
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum IcuNode {
    Argument(ArgumentNode),
    Number(NumberNode),
    Date(DateNode),
    Time(TimeNode),
    Selectable(SelectableNode),
    Pound,
}
impl From<ArgumentNode> for IcuNode {
    fn from(value: ArgumentNode) -> Self {
        IcuNode::Argument(value)
    }
}
impl From<NumberNode> for IcuNode {
    fn from(value: NumberNode) -> Self {
        IcuNode::Number(value)
    }
}
impl From<DateNode> for IcuNode {
    fn from(value: DateNode) -> Self {
        IcuNode::Date(value)
    }
}
impl From<TimeNode> for IcuNode {
    fn from(value: TimeNode) -> Self {
        IcuNode::Time(value)
    }
}
impl From<SelectableNode> for IcuNode {
    fn from(value: SelectableNode) -> Self {
        IcuNode::Selectable(value)
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ParagraphNode {
    pub content: Box<CompiledElement>,
}
impl ParagraphNode {
    pub fn new(content: impl Into<CompiledElement>) -> ParagraphNode {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CodeBlockNode {
    pub info_string: Option<TextPointer>,
    pub content: Box<CompiledElement>,
}
impl CodeBlockNode {
    pub fn new(
        info_string: Option<TextPointer>,
        content: impl Into<CompiledElement>,
    ) -> CodeBlockNode {
        Self {
            info_string,
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HeadingNode {
    pub level: u8,
    pub content: Box<CompiledElement>,
}
impl HeadingNode {
    pub fn new(level: u8, content: impl Into<CompiledElement>) -> HeadingNode {
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
impl StrongNode {
    pub fn new(content: impl Into<CompiledElement>) -> StrongNode {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EmphasisNode {
    pub content: Box<CompiledElement>,
}
impl EmphasisNode {
    pub fn new(content: impl Into<CompiledElement>) -> EmphasisNode {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StrikethroughNode {
    pub content: Box<CompiledElement>,
}
impl StrikethroughNode {
    pub fn new(content: impl Into<CompiledElement>) -> StrikethroughNode {
        Self {
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CodeNode {
    pub content: Box<CompiledElement>,
}
impl CodeNode {
    pub fn new(content: impl Into<CompiledElement>) -> CodeNode {
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
    pub content: Box<CompiledElement>,
}
impl LinkNode {
    pub fn new(
        kind: LinkKind,
        destination: LinkDestination,
        title: Option<TextPointer>,
        alt: Option<TextPointer>,
        content: impl Into<CompiledElement>,
    ) -> LinkNode {
        Self {
            kind,
            destination,
            title,
            alt,
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct HookNode {
    pub name: TextPointer,
    pub content: Box<CompiledElement>,
}
impl HookNode {
    pub fn new(name: TextPointer, content: impl Into<CompiledElement>) -> HookNode {
        Self {
            name,
            content: Box::from(content.into()),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
impl From<TextPointer> for LinkDestination {
    fn from(value: TextPointer) -> Self {
        LinkDestination::Static(value)
    }
}
impl From<IcuNode> for LinkDestination {
    fn from(value: IcuNode) -> Self {
        LinkDestination::Dynamic(value)
    }
}
impl From<ArgumentNode> for LinkDestination {
    fn from(value: ArgumentNode) -> Self {
        LinkDestination::Dynamic(IcuNode::Argument(value))
    }
}
impl From<NumberNode> for LinkDestination {
    fn from(value: NumberNode) -> Self {
        LinkDestination::Dynamic(IcuNode::Number(value))
    }
}
impl From<DateNode> for LinkDestination {
    fn from(value: DateNode) -> Self {
        LinkDestination::Dynamic(IcuNode::Date(value))
    }
}
impl From<TimeNode> for LinkDestination {
    fn from(value: TimeNode) -> Self {
        LinkDestination::Dynamic(IcuNode::Time(value))
    }
}
impl From<SelectableNode> for LinkDestination {
    fn from(value: SelectableNode) -> Self {
        LinkDestination::Dynamic(IcuNode::Selectable(value))
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ArgumentNode {
    pub name: TextPointer,
}
impl ArgumentNode {
    pub fn new(name: TextPointer) -> ArgumentNode {
        Self { name }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct NumberNode {
    pub name: TextPointer,
    pub style: Option<TextPointer>,
}
impl NumberNode {
    pub fn new(name: TextPointer, style: Option<TextPointer>) -> NumberNode {
        Self { name, style }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct DateNode {
    pub name: TextPointer,
    pub style: Option<TextPointer>,
}
impl DateNode {
    pub fn new(name: TextPointer, style: Option<TextPointer>) -> DateNode {
        Self { name, style }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TimeNode {
    pub name: TextPointer,
    pub style: Option<TextPointer>,
}
impl TimeNode {
    pub fn new(name: TextPointer, style: Option<TextPointer>) -> TimeNode {
        Self { name, style }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SelectableNode {
    pub name: TextPointer,
    pub kind: SelectableKind,
    pub offset: Option<TextPointer>,
    pub options: Box<[IcuOption]>,
}
impl SelectableNode {
    pub fn new(
        name: TextPointer,
        kind: SelectableKind,
        offset: Option<TextPointer>,
        options: Box<[IcuOption]>,
    ) -> SelectableNode {
        Self {
            name,
            kind,
            offset,
            options,
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum SelectableKind {
    Select,
    SelectOrdinal,
    Plural,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct IcuOption {
    pub name: TextPointer,
    pub value: Box<CompiledElement>,
}
impl IcuOption {
    pub fn new(name: TextPointer, value: impl Into<CompiledElement>) -> IcuOption {
        Self {
            name,
            value: Box::from(value.into()),
        }
    }
}
