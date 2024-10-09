use intl_markdown::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate,
    IcuDateTimeStyle, IcuNumber, IcuNumberStyle, IcuPlural, IcuPluralArm, IcuSelect, IcuTime,
    IcuVariable, InlineContent, Link, Paragraph, Strikethrough, Strong, TextOrPlaceholder,
};

use crate::visitor::Visit;

pub trait VisitWith<V: ?Sized + Visit> {
    fn visit_with(&self, visitor: &mut V);
    fn visit_children_with(&self, visitor: &mut V);
}

#[inline(always)]
fn visit_list<V: ?Sized + Visit, T: VisitWith<V>>(nodes: &[T], visitor: &mut V) {
    for node in nodes {
        node.visit_with(visitor);
    }
}

impl<V: ?Sized + Visit> VisitWith<V> for BlockNode {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_block_node(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            BlockNode::Paragraph(paragraph) => paragraph.visit_with(visitor),
            BlockNode::Heading(heading) => heading.visit_with(visitor),
            BlockNode::CodeBlock(code_block) => code_block.visit_with(visitor),
            BlockNode::ThematicBreak => visitor.visit_thematic_break(),
            BlockNode::InlineContent(inline_content) => visit_list(&inline_content, visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CodeBlock {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_block(self);
    }

    fn visit_children_with(&self, _visitor: &mut V) {
        // No children
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for CodeSpan {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_code_span(self);
    }

    fn visit_children_with(&self, _visitor: &mut V) {
        // No children
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Document {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_document(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(&self.blocks(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Emphasis {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_emphasis(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(&self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Heading {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_heading(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Hook {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_hook(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Icu {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            Icu::IcuVariable(variable) => variable.visit_with(visitor),
            Icu::IcuPlural(plural) => plural.visit_with(visitor),
            Icu::IcuSelect(select) => select.visit_with(visitor),
            Icu::IcuDate(date) => date.visit_with(visitor),
            Icu::IcuTime(time) => time.visit_with(visitor),
            Icu::IcuNumber(number) => number.visit_with(visitor),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuDate {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_date(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        if let Some(style) = self.style() {
            style.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuDateTimeStyle {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_date_time_style(self);
    }

    fn visit_children_with(&self, _visitor: &mut V) {
        // No children
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuNumber {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_number(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        if let Some(style) = self.style() {
            style.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuNumberStyle {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_number_style(self);
    }

    fn visit_children_with(&self, _visitor: &mut V) {
        // No children
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPlural {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_plural(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        visit_list(self.arms(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuPluralArm {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_plural_arm(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuSelect {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_select(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        visit_list(self.arms(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuTime {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_time(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        self.variable().visit_with(visitor);
        if let Some(style) = self.style() {
            style.visit_with(visitor);
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for IcuVariable {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_icu_variable(self);
    }

    fn visit_children_with(&self, _visitor: &mut V) {
        // No children
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for InlineContent {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_inline_content(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            InlineContent::Text(text) => visitor.visit_text(text),
            InlineContent::Emphasis(emphasis) => emphasis.visit_with(visitor),
            InlineContent::Strong(strong) => strong.visit_with(visitor),
            InlineContent::Link(link) => link.visit_with(visitor),
            InlineContent::CodeSpan(code_span) => code_span.visit_with(visitor),
            InlineContent::HardLineBreak => visitor.visit_hard_line_break(),
            InlineContent::Hook(hook) => hook.visit_with(visitor),
            InlineContent::Strikethrough(strikethrough) => strikethrough.visit_with(visitor),
            InlineContent::Icu(icu) => icu.visit_with(visitor),
            InlineContent::IcuPound => visitor.visit_icu_pound(),
        }
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Link {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_link(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.label(), visitor);
        // LinkDestination is not actually a node type right now, it's virtually created here.
        visitor.visit_link_destination(self.destination());
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Paragraph {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_paragraph(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Strikethrough {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_strikethrough(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for Strong {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_strong(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        visit_list(self.content(), visitor);
    }
}
impl<V: ?Sized + Visit> VisitWith<V> for TextOrPlaceholder {
    fn visit_with(&self, visitor: &mut V) {
        visitor.visit_text_or_placeholder(self);
    }

    fn visit_children_with(&self, visitor: &mut V) {
        match self {
            // Only placeholders need to be visited, since Text and Handler are both just static
            // strings.
            TextOrPlaceholder::Placeholder(placeholder) => placeholder.visit_with(visitor),
            _ => {}
        }
    }
}
