use super::nodes::*;
pub trait Visit {
    fn visit_document(&self, node: &Document);
    fn visit_any_block_node(&self, node: &AnyBlockNode);
    fn visit_paragraph(&self, node: &Paragraph);
    fn visit_thematic_break(&self, node: &ThematicBreak);
    fn visit_any_heading(&self, node: &AnyHeading);
    fn visit_any_code_block(&self, node: &AnyCodeBlock);
    fn visit_inline_content(&self, node: &InlineContent);
    fn visit_atx_heading(&self, node: &AtxHeading);
    fn visit_setext_heading(&self, node: &SetextHeading);
    fn visit_setext_heading_underline(&self, node: &SetextHeadingUnderline);
    fn visit_indented_code_block(&self, node: &IndentedCodeBlock);
    fn visit_fenced_code_block(&self, node: &FencedCodeBlock);
    fn visit_code_block_content(&self, node: &CodeBlockContent);
    fn visit_any_inline_node(&self, node: &AnyInlineNode);
    fn visit_text_span(&self, node: &TextSpan);
    fn visit_entity_reference(&self, node: &EntityReference);
    fn visit_emphasis(&self, node: &Emphasis);
    fn visit_strong(&self, node: &Strong);
    fn visit_link(&self, node: &Link);
    fn visit_image(&self, node: &Image);
    fn visit_autolink(&self, node: &Autolink);
    fn visit_code_span(&self, node: &CodeSpan);
    fn visit_hook(&self, node: &Hook);
    fn visit_strikethrough(&self, node: &Strikethrough);
    fn visit_icu(&self, node: &Icu);
    fn visit_link_resource(&self, node: &LinkResource);
    fn visit_any_link_destination(&self, node: &AnyLinkDestination);
    fn visit_link_title(&self, node: &LinkTitle);
    fn visit_static_link_destination(&self, node: &StaticLinkDestination);
    fn visit_dynamic_link_destination(&self, node: &DynamicLinkDestination);
    fn visit_click_handler_link_destination(&self, node: &ClickHandlerLinkDestination);
    fn visit_link_title_content(&self, node: &LinkTitleContent);
    fn visit_code_span_content(&self, node: &CodeSpanContent);
    fn visit_hook_name(&self, node: &HookName);
    fn visit_any_icu_placeholder(&self, node: &AnyIcuPlaceholder);
    fn visit_icu_variable(&self, node: &IcuVariable);
    fn visit_icu_plural(&self, node: &IcuPlural);
    fn visit_icu_select_ordinal(&self, node: &IcuSelectOrdinal);
    fn visit_icu_select(&self, node: &IcuSelect);
    fn visit_icu_date(&self, node: &IcuDate);
    fn visit_icu_time(&self, node: &IcuTime);
    fn visit_icu_number(&self, node: &IcuNumber);
    fn visit_icu_plural_arms(&self, node: &IcuPluralArms);
    fn visit_icu_plural_arm(&self, node: &IcuPluralArm);
    fn visit_icu_plural_value(&self, node: &IcuPluralValue);
    fn visit_icu_date_time_style(&self, node: &IcuDateTimeStyle);
    fn visit_icu_number_style(&self, node: &IcuNumberStyle);
}
pub trait Fold {
    fn fold_document(&self, node: Document) -> Document;
    fn fold_any_block_node(&self, node: AnyBlockNode) -> AnyBlockNode;
    fn fold_paragraph(&self, node: Paragraph) -> Paragraph;
    fn fold_thematic_break(&self, node: ThematicBreak) -> ThematicBreak;
    fn fold_any_heading(&self, node: AnyHeading) -> AnyHeading;
    fn fold_any_code_block(&self, node: AnyCodeBlock) -> AnyCodeBlock;
    fn fold_inline_content(&self, node: InlineContent) -> InlineContent;
    fn fold_atx_heading(&self, node: AtxHeading) -> AtxHeading;
    fn fold_setext_heading(&self, node: SetextHeading) -> SetextHeading;
    fn fold_setext_heading_underline(&self, node: SetextHeadingUnderline)
        -> SetextHeadingUnderline;
    fn fold_indented_code_block(&self, node: IndentedCodeBlock) -> IndentedCodeBlock;
    fn fold_fenced_code_block(&self, node: FencedCodeBlock) -> FencedCodeBlock;
    fn fold_code_block_content(&self, node: CodeBlockContent) -> CodeBlockContent;
    fn fold_any_inline_node(&self, node: AnyInlineNode) -> AnyInlineNode;
    fn fold_text_span(&self, node: TextSpan) -> TextSpan;
    fn fold_entity_reference(&self, node: EntityReference) -> EntityReference;
    fn fold_emphasis(&self, node: Emphasis) -> Emphasis;
    fn fold_strong(&self, node: Strong) -> Strong;
    fn fold_link(&self, node: Link) -> Link;
    fn fold_image(&self, node: Image) -> Image;
    fn fold_autolink(&self, node: Autolink) -> Autolink;
    fn fold_code_span(&self, node: CodeSpan) -> CodeSpan;
    fn fold_hook(&self, node: Hook) -> Hook;
    fn fold_strikethrough(&self, node: Strikethrough) -> Strikethrough;
    fn fold_icu(&self, node: Icu) -> Icu;
    fn fold_link_resource(&self, node: LinkResource) -> LinkResource;
    fn fold_any_link_destination(&self, node: AnyLinkDestination) -> AnyLinkDestination;
    fn fold_link_title(&self, node: LinkTitle) -> LinkTitle;
    fn fold_static_link_destination(&self, node: StaticLinkDestination) -> StaticLinkDestination;
    fn fold_dynamic_link_destination(&self, node: DynamicLinkDestination)
        -> DynamicLinkDestination;
    fn fold_click_handler_link_destination(
        &self,
        node: ClickHandlerLinkDestination,
    ) -> ClickHandlerLinkDestination;
    fn fold_link_title_content(&self, node: LinkTitleContent) -> LinkTitleContent;
    fn fold_code_span_content(&self, node: CodeSpanContent) -> CodeSpanContent;
    fn fold_hook_name(&self, node: HookName) -> HookName;
    fn fold_any_icu_placeholder(&self, node: AnyIcuPlaceholder) -> AnyIcuPlaceholder;
    fn fold_icu_variable(&self, node: IcuVariable) -> IcuVariable;
    fn fold_icu_plural(&self, node: IcuPlural) -> IcuPlural;
    fn fold_icu_select_ordinal(&self, node: IcuSelectOrdinal) -> IcuSelectOrdinal;
    fn fold_icu_select(&self, node: IcuSelect) -> IcuSelect;
    fn fold_icu_date(&self, node: IcuDate) -> IcuDate;
    fn fold_icu_time(&self, node: IcuTime) -> IcuTime;
    fn fold_icu_number(&self, node: IcuNumber) -> IcuNumber;
    fn fold_icu_plural_arms(&self, node: IcuPluralArms) -> IcuPluralArms;
    fn fold_icu_plural_arm(&self, node: IcuPluralArm) -> IcuPluralArm;
    fn fold_icu_plural_value(&self, node: IcuPluralValue) -> IcuPluralValue;
    fn fold_icu_date_time_style(&self, node: IcuDateTimeStyle) -> IcuDateTimeStyle;
    fn fold_icu_number_style(&self, node: IcuNumberStyle) -> IcuNumberStyle;
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
impl<V: ?Sized + Visit> VisitWith<V> for Document {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_document(self);
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
            Self::InlineContent(node) => node.visit_with(visitor),
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
impl<V: ?Sized + Visit> VisitWith<V> for AnyInlineNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_inline_node(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::TextSpan(node) => node.visit_with(visitor),
            Self::EntityReference(node) => node.visit_with(visitor),
            Self::Emphasis(node) => node.visit_with(visitor),
            Self::Strong(node) => node.visit_with(visitor),
            Self::Link(node) => node.visit_with(visitor),
            Self::Image(node) => node.visit_with(visitor),
            Self::Autolink(node) => node.visit_with(visitor),
            Self::CodeSpan(node) => node.visit_with(visitor),
            Self::Hook(node) => node.visit_with(visitor),
            Self::Strikethrough(node) => node.visit_with(visitor),
            Self::Icu(node) => node.visit_with(visitor),
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
impl<V: ?Sized + Visit> VisitWith<V> for EntityReference {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_entity_reference(self);
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
impl<V: ?Sized + Visit> VisitWith<V> for AnyIcuPlaceholder {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_any_icu_placeholder(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Self::IcuVariable(node) => node.visit_with(visitor),
            Self::IcuPlural(node) => node.visit_with(visitor),
            Self::IcuSelectOrdinal(node) => node.visit_with(visitor),
            Self::IcuSelect(node) => node.visit_with(visitor),
            Self::IcuDate(node) => node.visit_with(visitor),
            Self::IcuTime(node) => node.visit_with(visitor),
            Self::IcuNumber(node) => node.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuVariable {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_variable(self);
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
        self.variable().visit_with(visitor);
        self.arms().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuSelectOrdinal {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_select_ordinal(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        self.arms().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuSelect {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_select(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        self.arms().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuDate {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_date(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        self.style().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuTime {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_time(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        self.style().visit_with(visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuNumber {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_number(self);
    }
    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
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
