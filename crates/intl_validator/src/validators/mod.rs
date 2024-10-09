use intl_markdown::{
    BlockNode, CodeBlock, CodeSpan, Document, Emphasis, Heading, Hook, Icu, IcuDate,
    IcuDateTimeStyle, IcuNumber, IcuNumberStyle, IcuPlural, IcuPluralArm, IcuSelect, IcuTime,
    IcuVariable, InlineContent, Link, Paragraph, Strikethrough, Strong, TextOrPlaceholder, Visitor,
};
pub use no_repeated_plural_names::NoRepeatedPluralNames;
pub use no_unicode_variable_names::NoUnicodeVariableNames;

mod no_repeated_plural_names;
mod no_unicode_variable_names;

pub struct ValidationVisitor {
    visitors: Vec<Box<dyn Visitor>>,
}

impl ValidationVisitor {
    pub fn new(visitors: Vec<Box<dyn Visitor>>) -> Self {
        Self {
            visitors: Vec::from(visitors),
        }
    }
}

macro_rules! visit_with_all {
    ($visit_name:ident, $exit_name:ident, $kind:ty) => {
        fn $visit_name(&mut self, node: &$kind) {
            for visitor in self.visitors.iter_mut() {
                visitor.$visit_name(node);
            }
        }
        fn $exit_name(&mut self, node: &$kind) {
            for visitor in self.visitors.iter_mut() {
                visitor.$exit_name(node);
            }
        }
    };
    ($visit_name:ident, $exit_name:ident) => {
        fn $visit_name(&mut self) {
            for visitor in self.visitors.iter_mut() {
                visitor.$visit_name();
            }
        }
        fn $exit_name(&mut self) {
            for visitor in self.visitors.iter_mut() {
                visitor.$exit_name();
            }
        }
    };
}

impl Visitor for ValidationVisitor {
    visit_with_all!(visit_block_node, exit_block_node, BlockNode);
    visit_with_all!(visit_code_block, exit_code_block, CodeBlock);
    visit_with_all!(visit_code_span, exit_code_span, CodeSpan);
    visit_with_all!(visit_document, exit_document, Document);
    visit_with_all!(visit_emphasis, exit_emphasis, Emphasis);
    visit_with_all!(visit_heading, exit_heading, Heading);
    visit_with_all!(visit_hook, exit_hook, Hook);
    visit_with_all!(visit_icu, exit_icu, Icu);
    visit_with_all!(visit_icu_date, exit_icu_date, IcuDate);
    visit_with_all!(
        visit_icu_date_time_style,
        exit_icu_date_time_style,
        IcuDateTimeStyle
    );
    visit_with_all!(visit_icu_number, exit_icu_number, IcuNumber);
    visit_with_all!(
        visit_icu_number_style,
        exit_icu_number_style,
        IcuNumberStyle
    );
    visit_with_all!(visit_icu_plural, exit_icu_plural, IcuPlural);
    visit_with_all!(visit_icu_plural_arm, exit_icu_plural_arm, IcuPluralArm);
    visit_with_all!(visit_icu_select, exit_icu_select, IcuSelect);
    visit_with_all!(visit_icu_time, exit_icu_time, IcuTime);
    visit_with_all!(visit_icu_variable, exit_icu_variable, IcuVariable);
    visit_with_all!(visit_inline_content, exit_inline_content, InlineContent);
    visit_with_all!(visit_link, exit_link, Link);
    visit_with_all!(
        visit_link_destination,
        exit_link_destination,
        TextOrPlaceholder
    );
    visit_with_all!(visit_paragraph, exit_paragraph, Paragraph);
    visit_with_all!(visit_strikethrough, exit_strikethrough, Strikethrough);
    visit_with_all!(visit_strong, exit_strong, Strong);
    visit_with_all!(
        visit_text_or_placeholder,
        exit_text_or_placeholder,
        TextOrPlaceholder
    );
    visit_with_all!(visit_text, exit_text, String);

    visit_with_all!(visit_thematic_break, exit_thematic_break);
    visit_with_all!(visit_hard_line_break, exit_hard_line_break);
    visit_with_all!(visit_icu_pound, exit_icu_pound);
}
