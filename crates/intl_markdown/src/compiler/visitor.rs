use super::element::*;
use crate::syntax::TextPointer;
pub trait VisitCompiled {
    fn visit_text_pointer(&mut self, text: &TextPointer) {
        text.visit_children_with(self);
    }
    fn visit_compiled_element(&mut self, node: &CompiledElement) {
        node.visit_children_with(self);
    }
    fn visit_compiled_node(&mut self, node: &CompiledNode) {
        node.visit_children_with(self);
    }
    fn visit_markdown_node(&mut self, node: &MarkdownNode) {
        node.visit_children_with(self);
    }
    fn visit_icu_node(&mut self, node: &IcuNode) {
        node.visit_children_with(self);
    }
    fn visit_paragraph_node(&mut self, node: &ParagraphNode) {
        node.visit_children_with(self);
    }
    fn visit_code_block_node(&mut self, node: &CodeBlockNode) {
        node.visit_children_with(self);
    }
    fn visit_heading_node(&mut self, node: &HeadingNode) {
        node.visit_children_with(self);
    }
    fn visit_thematic_break_node(&mut self, node: &ThematicBreakNode) {
        node.visit_children_with(self);
    }
    fn visit_line_break_node(&mut self, node: &LineBreakNode) {
        node.visit_children_with(self);
    }
    fn visit_strong_node(&mut self, node: &StrongNode) {
        node.visit_children_with(self);
    }
    fn visit_emphasis_node(&mut self, node: &EmphasisNode) {
        node.visit_children_with(self);
    }
    fn visit_strikethrough_node(&mut self, node: &StrikethroughNode) {
        node.visit_children_with(self);
    }
    fn visit_code_node(&mut self, node: &CodeNode) {
        node.visit_children_with(self);
    }
    fn visit_link_node(&mut self, node: &LinkNode) {
        node.visit_children_with(self);
    }
    fn visit_hook_node(&mut self, node: &HookNode) {
        node.visit_children_with(self);
    }
    fn visit_link_destination(&mut self, node: &LinkDestination) {
        node.visit_children_with(self);
    }
    fn visit_argument_node(&mut self, node: &ArgumentNode) {
        node.visit_children_with(self);
    }
    fn visit_number_node(&mut self, node: &NumberNode) {
        node.visit_children_with(self);
    }
    fn visit_date_node(&mut self, node: &DateNode) {
        node.visit_children_with(self);
    }
    fn visit_time_node(&mut self, node: &TimeNode) {
        node.visit_children_with(self);
    }
    fn visit_selectable_node(&mut self, node: &SelectableNode) {
        node.visit_children_with(self);
    }
    fn visit_pound_node(&mut self, node: &PoundNode) {
        node.visit_children_with(self);
    }
    fn visit_icu_option(&mut self, node: &IcuOption) {
        node.visit_children_with(self);
    }
}
pub trait FoldCompiled {
    fn fold_text_pointer(&mut self, text: TextPointer) -> TextPointer;
    fn fold_compiled_element(&mut self, node: CompiledElement) -> CompiledElement;
    fn fold_compiled_node(&mut self, node: CompiledNode) -> CompiledNode;
    fn fold_markdown_node(&mut self, node: MarkdownNode) -> MarkdownNode;
    fn fold_icu_node(&mut self, node: IcuNode) -> IcuNode;
    fn fold_paragraph_node(&mut self, node: ParagraphNode) -> ParagraphNode;
    fn fold_code_block_node(&mut self, node: CodeBlockNode) -> CodeBlockNode;
    fn fold_heading_node(&mut self, node: HeadingNode) -> HeadingNode;
    fn fold_thematic_break_node(&mut self, node: ThematicBreakNode) -> ThematicBreakNode;
    fn fold_line_break_node(&mut self, node: LineBreakNode) -> LineBreakNode;
    fn fold_strong_node(&mut self, node: StrongNode) -> StrongNode;
    fn fold_emphasis_node(&mut self, node: EmphasisNode) -> EmphasisNode;
    fn fold_strikethrough_node(&mut self, node: StrikethroughNode) -> StrikethroughNode;
    fn fold_code_node(&mut self, node: CodeNode) -> CodeNode;
    fn fold_link_node(&mut self, node: LinkNode) -> LinkNode;
    fn fold_hook_node(&mut self, node: HookNode) -> HookNode;
    fn fold_link_destination(&mut self, node: LinkDestination) -> LinkDestination;
    fn fold_argument_node(&mut self, node: ArgumentNode) -> ArgumentNode;
    fn fold_number_node(&mut self, node: NumberNode) -> NumberNode;
    fn fold_date_node(&mut self, node: DateNode) -> DateNode;
    fn fold_time_node(&mut self, node: TimeNode) -> TimeNode;
    fn fold_selectable_node(&mut self, node: SelectableNode) -> SelectableNode;
    fn fold_pound_node(&mut self, node: PoundNode) -> PoundNode;
    fn fold_icu_option(&mut self, node: IcuOption) -> IcuOption;
}
pub trait VisitCompiledWith<V: ?Sized + VisitCompiled> {
    fn visit_with(&self, visitor: &mut V);
    fn visit_children_with(&self, visitor: &mut V);
}
impl<V: ?Sized + VisitCompiled, T: VisitCompiledWith<V>> VisitCompiledWith<V> for Option<T> {
    fn visit_with(&self, visitor: &mut V) {
        self.as_ref().map(|v| v.visit_with(visitor));
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.as_ref().map(|v| v.visit_children_with(visitor));
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for TextPointer {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_text_pointer(self)
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for CompiledElement {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_compiled_element(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::BlockList(children) => {
                children.iter().for_each(|child| child.visit_with(visitor))
            }
            Self::List(children) => children.iter().for_each(|child| child.visit_with(visitor)),
            Self::Node(node) => node.visit_with(visitor),
            Self::Literal(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for CompiledNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_compiled_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Markdown(node) => node.visit_with(visitor),
            Self::Icu(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for MarkdownNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_markdown_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Paragraph(node) => node.visit_with(visitor),
            Self::CodeBlock(node) => node.visit_with(visitor),
            Self::Heading(node) => node.visit_with(visitor),
            Self::ThematicBreak(node) => node.visit_with(visitor),
            Self::LineBreak(node) => node.visit_with(visitor),
            Self::Strong(node) => node.visit_with(visitor),
            Self::Emphasis(node) => node.visit_with(visitor),
            Self::Strikethrough(node) => node.visit_with(visitor),
            Self::Code(node) => node.visit_with(visitor),
            Self::Link(node) => node.visit_with(visitor),
            Self::Hook(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for IcuNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Argument(node) => node.visit_with(visitor),
            Self::Number(node) => node.visit_with(visitor),
            Self::Date(node) => node.visit_with(visitor),
            Self::Time(node) => node.visit_with(visitor),
            Self::Selectable(node) => node.visit_with(visitor),
            Self::Pound(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for ParagraphNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_paragraph_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for CodeBlockNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_block_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for HeadingNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_heading_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for ThematicBreakNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_thematic_break_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for LineBreakNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_line_break_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for StrongNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_strong_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for EmphasisNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_emphasis_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for StrikethroughNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_strikethrough_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for CodeNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for LinkNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.destination.visit_with(visitor);
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for HookNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_hook_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content.visit_with(visitor)
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for LinkDestination {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link_destination(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Static(node) => node.visit_with(visitor),
            Self::Dynamic(node) => node.visit_with(visitor),
            Self::Handler(node) => node.visit_with(visitor),
            Self::Empty => {}
        }
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for ArgumentNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_argument_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for NumberNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_number_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for DateNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_date_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for TimeNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_time_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for SelectableNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_selectable_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.options
            .iter()
            .for_each(|child| child.visit_with(visitor))
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for PoundNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_pound_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + VisitCompiled> VisitCompiledWith<V> for IcuOption {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_option(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.value.visit_with(visitor)
    }
}
