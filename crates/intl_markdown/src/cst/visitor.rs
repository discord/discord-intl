use super::nodes::*;
pub trait Visit {
    fn visit_any_document(&mut self, node: &AnyDocument) {
        node.visit_children_with(self);
    }
    fn visit_block_document(&mut self, node: &BlockDocument) {
        node.visit_children_with(self);
    }
    fn visit_inline_content(&mut self, node: &InlineContent) {
        node.visit_children_with(self);
    }
    fn visit_any_block_node(&mut self, node: &AnyBlockNode) {
        node.visit_children_with(self);
    }
    fn visit_paragraph(&mut self, node: &Paragraph) {
        node.visit_children_with(self);
    }
    fn visit_thematic_break(&mut self, node: &ThematicBreak) {
        node.visit_children_with(self);
    }
    fn visit_any_heading(&mut self, node: &AnyHeading) {
        node.visit_children_with(self);
    }
    fn visit_any_code_block(&mut self, node: &AnyCodeBlock) {
        node.visit_children_with(self);
    }
    fn visit_block_space(&mut self, node: &BlockSpace) {
        node.visit_children_with(self);
    }
    fn visit_atx_heading(&mut self, node: &AtxHeading) {
        node.visit_children_with(self);
    }
    fn visit_setext_heading(&mut self, node: &SetextHeading) {
        node.visit_children_with(self);
    }
    fn visit_setext_heading_underline(&mut self, node: &SetextHeadingUnderline) {
        node.visit_children_with(self);
    }
    fn visit_indented_code_block(&mut self, node: &IndentedCodeBlock) {
        node.visit_children_with(self);
    }
    fn visit_fenced_code_block(&mut self, node: &FencedCodeBlock) {
        node.visit_children_with(self);
    }
    fn visit_code_block_content(&mut self, node: &CodeBlockContent) {
        node.visit_children_with(self);
    }
    fn visit_code_block_info_string(&mut self, node: &CodeBlockInfoString) {
        node.visit_children_with(self);
    }
    fn visit_any_inline_node(&mut self, node: &AnyInlineNode) {
        node.visit_children_with(self);
    }
    fn visit_text_span(&mut self, node: &TextSpan) {
        node.visit_children_with(self);
    }
    fn visit_emphasis(&mut self, node: &Emphasis) {
        node.visit_children_with(self);
    }
    fn visit_strong(&mut self, node: &Strong) {
        node.visit_children_with(self);
    }
    fn visit_link(&mut self, node: &Link) {
        node.visit_children_with(self);
    }
    fn visit_image(&mut self, node: &Image) {
        node.visit_children_with(self);
    }
    fn visit_autolink(&mut self, node: &Autolink) {
        node.visit_children_with(self);
    }
    fn visit_code_span(&mut self, node: &CodeSpan) {
        node.visit_children_with(self);
    }
    fn visit_hook(&mut self, node: &Hook) {
        node.visit_children_with(self);
    }
    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        node.visit_children_with(self);
    }
    fn visit_icu(&mut self, node: &Icu) {
        node.visit_children_with(self);
    }
    fn visit_icu_pound(&mut self, node: &IcuPound) {
        node.visit_children_with(self);
    }
    fn visit_link_resource(&mut self, node: &LinkResource) {
        node.visit_children_with(self);
    }
    fn visit_any_link_destination(&mut self, node: &AnyLinkDestination) {
        node.visit_children_with(self);
    }
    fn visit_link_title(&mut self, node: &LinkTitle) {
        node.visit_children_with(self);
    }
    fn visit_static_link_destination(&mut self, node: &StaticLinkDestination) {
        node.visit_children_with(self);
    }
    fn visit_dynamic_link_destination(&mut self, node: &DynamicLinkDestination) {
        node.visit_children_with(self);
    }
    fn visit_click_handler_link_destination(&mut self, node: &ClickHandlerLinkDestination) {
        node.visit_children_with(self);
    }
    fn visit_link_title_content(&mut self, node: &LinkTitleContent) {
        node.visit_children_with(self);
    }
    fn visit_code_span_content(&mut self, node: &CodeSpanContent) {
        node.visit_children_with(self);
    }
    fn visit_hook_name(&mut self, node: &HookName) {
        node.visit_children_with(self);
    }
    fn visit_any_icu_expression(&mut self, node: &AnyIcuExpression) {
        node.visit_children_with(self);
    }
    fn visit_icu_placeholder(&mut self, node: &IcuPlaceholder) {
        node.visit_children_with(self);
    }
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        node.visit_children_with(self);
    }
    fn visit_icu_select_ordinal(&mut self, node: &IcuSelectOrdinal) {
        node.visit_children_with(self);
    }
    fn visit_icu_select(&mut self, node: &IcuSelect) {
        node.visit_children_with(self);
    }
    fn visit_icu_date(&mut self, node: &IcuDate) {
        node.visit_children_with(self);
    }
    fn visit_icu_time(&mut self, node: &IcuTime) {
        node.visit_children_with(self);
    }
    fn visit_icu_number(&mut self, node: &IcuNumber) {
        node.visit_children_with(self);
    }
    fn visit_icu_plural_arms(&mut self, node: &IcuPluralArms) {
        node.visit_children_with(self);
    }
    fn visit_icu_plural_arm(&mut self, node: &IcuPluralArm) {
        node.visit_children_with(self);
    }
    fn visit_icu_plural_value(&mut self, node: &IcuPluralValue) {
        node.visit_children_with(self);
    }
    fn visit_icu_date_time_style(&mut self, node: &IcuDateTimeStyle) {
        node.visit_children_with(self);
    }
    fn visit_icu_number_style(&mut self, node: &IcuNumberStyle) {
        node.visit_children_with(self);
    }
}
pub trait Fold {
    fn fold_any_document(&mut self, node: AnyDocument) -> AnyDocument;
    fn fold_block_document(&mut self, node: BlockDocument) -> BlockDocument;
    fn fold_inline_content(&mut self, node: InlineContent) -> InlineContent;
    fn fold_any_block_node(&mut self, node: AnyBlockNode) -> AnyBlockNode;
    fn fold_paragraph(&mut self, node: Paragraph) -> Paragraph;
    fn fold_thematic_break(&mut self, node: ThematicBreak) -> ThematicBreak;
    fn fold_any_heading(&mut self, node: AnyHeading) -> AnyHeading;
    fn fold_any_code_block(&mut self, node: AnyCodeBlock) -> AnyCodeBlock;
    fn fold_block_space(&mut self, node: BlockSpace) -> BlockSpace;
    fn fold_atx_heading(&mut self, node: AtxHeading) -> AtxHeading;
    fn fold_setext_heading(&mut self, node: SetextHeading) -> SetextHeading;
    fn fold_setext_heading_underline(
        &mut self,
        node: SetextHeadingUnderline,
    ) -> SetextHeadingUnderline;
    fn fold_indented_code_block(&mut self, node: IndentedCodeBlock) -> IndentedCodeBlock;
    fn fold_fenced_code_block(&mut self, node: FencedCodeBlock) -> FencedCodeBlock;
    fn fold_code_block_content(&mut self, node: CodeBlockContent) -> CodeBlockContent;
    fn fold_code_block_info_string(&mut self, node: CodeBlockInfoString) -> CodeBlockInfoString;
    fn fold_any_inline_node(&mut self, node: AnyInlineNode) -> AnyInlineNode;
    fn fold_text_span(&mut self, node: TextSpan) -> TextSpan;
    fn fold_emphasis(&mut self, node: Emphasis) -> Emphasis;
    fn fold_strong(&mut self, node: Strong) -> Strong;
    fn fold_link(&mut self, node: Link) -> Link;
    fn fold_image(&mut self, node: Image) -> Image;
    fn fold_autolink(&mut self, node: Autolink) -> Autolink;
    fn fold_code_span(&mut self, node: CodeSpan) -> CodeSpan;
    fn fold_hook(&mut self, node: Hook) -> Hook;
    fn fold_strikethrough(&mut self, node: Strikethrough) -> Strikethrough;
    fn fold_icu(&mut self, node: Icu) -> Icu;
    fn fold_icu_pound(&mut self, node: IcuPound) -> IcuPound;
    fn fold_link_resource(&mut self, node: LinkResource) -> LinkResource;
    fn fold_any_link_destination(&mut self, node: AnyLinkDestination) -> AnyLinkDestination;
    fn fold_link_title(&mut self, node: LinkTitle) -> LinkTitle;
    fn fold_static_link_destination(
        &mut self,
        node: StaticLinkDestination,
    ) -> StaticLinkDestination;
    fn fold_dynamic_link_destination(
        &mut self,
        node: DynamicLinkDestination,
    ) -> DynamicLinkDestination;
    fn fold_click_handler_link_destination(
        &mut self,
        node: ClickHandlerLinkDestination,
    ) -> ClickHandlerLinkDestination;
    fn fold_link_title_content(&mut self, node: LinkTitleContent) -> LinkTitleContent;
    fn fold_code_span_content(&mut self, node: CodeSpanContent) -> CodeSpanContent;
    fn fold_hook_name(&mut self, node: HookName) -> HookName;
    fn fold_any_icu_expression(&mut self, node: AnyIcuExpression) -> AnyIcuExpression;
    fn fold_icu_placeholder(&mut self, node: IcuPlaceholder) -> IcuPlaceholder;
    fn fold_icu_plural(&mut self, node: IcuPlural) -> IcuPlural;
    fn fold_icu_select_ordinal(&mut self, node: IcuSelectOrdinal) -> IcuSelectOrdinal;
    fn fold_icu_select(&mut self, node: IcuSelect) -> IcuSelect;
    fn fold_icu_date(&mut self, node: IcuDate) -> IcuDate;
    fn fold_icu_time(&mut self, node: IcuTime) -> IcuTime;
    fn fold_icu_number(&mut self, node: IcuNumber) -> IcuNumber;
    fn fold_icu_plural_arms(&mut self, node: IcuPluralArms) -> IcuPluralArms;
    fn fold_icu_plural_arm(&mut self, node: IcuPluralArm) -> IcuPluralArm;
    fn fold_icu_plural_value(&mut self, node: IcuPluralValue) -> IcuPluralValue;
    fn fold_icu_date_time_style(&mut self, node: IcuDateTimeStyle) -> IcuDateTimeStyle;
    fn fold_icu_number_style(&mut self, node: IcuNumberStyle) -> IcuNumberStyle;
}
pub trait VisitWith<V: ?Sized + Visit> {
    fn visit_with(&self, visitor: &mut V);
    fn visit_children_with(&self, visitor: &mut V);
}
impl<V: ?Sized + Visit, T: VisitWith<V>> VisitWith<V> for Option<T> {
    fn visit_with(&self, visitor: &mut V) {
        self.as_ref().map(|v| v.visit_with(visitor));
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.as_ref().map(|v| v.visit_children_with(visitor));
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyDocument {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_document(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::BlockDocument(node) => node.visit_with(visitor),
            Self::InlineContent(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BlockDocument {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_block_document(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        for field in self.children() {
            field.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for InlineContent {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_inline_content(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        for field in self.children() {
            field.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyBlockNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_block_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::Paragraph(node) => node.visit_with(visitor),
            Self::ThematicBreak(node) => node.visit_with(visitor),
            Self::Heading(node) => node.visit_with(visitor),
            Self::CodeBlock(node) => node.visit_with(visitor),
            Self::BlockSpace(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Paragraph {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_paragraph(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ThematicBreak {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_thematic_break(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyHeading {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_heading(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::AtxHeading(node) => node.visit_with(visitor),
            Self::SetextHeading(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyCodeBlock {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_code_block(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::IndentedCodeBlock(node) => node.visit_with(visitor),
            Self::FencedCodeBlock(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for BlockSpace {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_block_space(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AtxHeading {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_atx_heading(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SetextHeading {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_setext_heading(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
        self.underline().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for SetextHeadingUnderline {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_setext_heading_underline(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IndentedCodeBlock {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_indented_code_block(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for FencedCodeBlock {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_fenced_code_block(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.info_string().visit_with(visitor);
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CodeBlockContent {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_block_content(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CodeBlockInfoString {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_block_info_string(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyInlineNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_inline_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::TextSpan(node) => node.visit_with(visitor),
            Self::Emphasis(node) => node.visit_with(visitor),
            Self::Strong(node) => node.visit_with(visitor),
            Self::Link(node) => node.visit_with(visitor),
            Self::Image(node) => node.visit_with(visitor),
            Self::Autolink(node) => node.visit_with(visitor),
            Self::CodeSpan(node) => node.visit_with(visitor),
            Self::Hook(node) => node.visit_with(visitor),
            Self::Strikethrough(node) => node.visit_with(visitor),
            Self::Icu(node) => node.visit_with(visitor),
            Self::IcuPound(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TextSpan {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_text_span(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Emphasis {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_emphasis(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Strong {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_strong(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Link {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
        self.resource().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Image {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_image(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
        self.resource().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Autolink {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_autolink(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CodeSpan {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_span(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Hook {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_hook(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
        self.name().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Strikethrough {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_strikethrough(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Icu {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.value().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPound {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_pound(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for LinkResource {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link_resource(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.destination().visit_with(visitor);
        self.title().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyLinkDestination {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_link_destination(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::StaticLinkDestination(node) => node.visit_with(visitor),
            Self::DynamicLinkDestination(node) => node.visit_with(visitor),
            Self::ClickHandlerLinkDestination(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for LinkTitle {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link_title(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for StaticLinkDestination {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_static_link_destination(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for DynamicLinkDestination {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_dynamic_link_destination(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.url().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for ClickHandlerLinkDestination {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_click_handler_link_destination(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for LinkTitleContent {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link_title_content(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CodeSpanContent {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_span_content(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for HookName {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_hook_name(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for AnyIcuExpression {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_icu_expression(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::IcuPlaceholder(node) => node.visit_with(visitor),
            Self::IcuPlural(node) => node.visit_with(visitor),
            Self::IcuSelectOrdinal(node) => node.visit_with(visitor),
            Self::IcuSelect(node) => node.visit_with(visitor),
            Self::IcuDate(node) => node.visit_with(visitor),
            Self::IcuTime(node) => node.visit_with(visitor),
            Self::IcuNumber(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPlaceholder {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_placeholder(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPlural {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_plural(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.arms().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuSelectOrdinal {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_select_ordinal(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.arms().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuSelect {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_select(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.arms().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuDate {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_date(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.style().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuTime {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_time(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.style().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuNumber {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_number(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.style().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPluralArms {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_plural_arms(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        for field in self.children() {
            field.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPluralArm {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_plural_arm(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.value().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPluralValue {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_plural_value(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.content().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuDateTimeStyle {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_date_time_style(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuNumberStyle {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_number_style(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        let _ = visitor;
    }
}
