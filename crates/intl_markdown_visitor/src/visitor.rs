use intl_markdown::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate,
    IcuDateTimeStyle, IcuNumber, IcuNumberStyle, IcuPlural, IcuPluralArm, IcuSelect, IcuTime,
    IcuVariable, InlineContent, Link, LinkDestination, Paragraph, Strikethrough, Strong,
};

use crate::visit_with::VisitWith;

pub trait Visit {
    fn visit_block_node(&mut self, node: &BlockNode) {
        node.visit_children_with(self);
    }
    fn visit_code_block(&mut self, node: &CodeBlock) {
        node.visit_children_with(self);
    }
    fn visit_code_span(&mut self, node: &CodeSpan) {
        node.visit_children_with(self);
    }
    fn visit_document(&mut self, node: &Document) {
        node.visit_children_with(self);
    }
    fn visit_emphasis(&mut self, node: &Emphasis) {
        node.visit_children_with(self);
    }
    fn visit_heading(&mut self, node: &Heading) {
        node.visit_children_with(self);
    }
    fn visit_hook(&mut self, node: &Hook) {
        node.visit_children_with(self);
    }
    fn visit_icu(&mut self, node: &Icu) {
        node.visit_children_with(self);
    }
    fn visit_icu_date(&mut self, node: &IcuDate) {
        node.visit_children_with(self);
    }
    fn visit_icu_date_time_style(&mut self, node: &IcuDateTimeStyle) {
        node.visit_children_with(self);
    }
    fn visit_icu_number(&mut self, node: &IcuNumber) {
        node.visit_children_with(self);
    }
    fn visit_icu_number_style(&mut self, node: &IcuNumberStyle) {
        node.visit_children_with(self);
    }
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        node.visit_children_with(self);
    }
    fn visit_icu_plural_arm(&mut self, node: &IcuPluralArm) {
        node.visit_children_with(self);
    }
    fn visit_icu_select(&mut self, node: &IcuSelect) {
        node.visit_children_with(self);
    }
    fn visit_icu_time(&mut self, node: &IcuTime) {
        node.visit_children_with(self);
    }
    fn visit_icu_variable(&mut self, node: &IcuVariable) {
        node.visit_children_with(self);
    }
    fn visit_inline_content(&mut self, node: &InlineContent) {
        node.visit_children_with(self);
    }
    fn visit_link(&mut self, node: &Link) {
        node.visit_children_with(self);
    }
    fn visit_link_destination(&mut self, node: &LinkDestination) {
        node.visit_children_with(self);
    }
    fn visit_paragraph(&mut self, node: &Paragraph) {
        node.visit_children_with(self);
    }
    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        node.visit_children_with(self);
    }
    fn visit_strong(&mut self, node: &Strong) {
        node.visit_children_with(self);
    }
    fn visit_text(&mut self, _node: &String) {
        // Not a node type, just visible text of the message.
    }

    fn visit_thematic_break(&mut self) {}
    fn visit_hard_line_break(&mut self) {}
    fn visit_icu_pound(&mut self) {}
}
