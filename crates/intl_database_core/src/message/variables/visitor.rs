use super::{MessageVariableType, MessageVariables};
use crate::database::symbol::key_symbol;
use intl_markdown::{
    AnyCodeBlock, AnyHeading, ClickHandlerLinkDestination, CodeSpan, Emphasis, Hook, Icu, IcuDate,
    IcuNumber, IcuPlaceholder, IcuPlural, IcuPound, IcuSelect, IcuSelectOrdinal, IcuTime, Link,
    Paragraph, Strikethrough, Strong, SyntaxKind, TextSpan, ThematicBreak, Visit, VisitWith,
};
use intl_markdown_macros::header_tag_lookup_map;
use intl_markdown_syntax::{Syntax, SyntaxToken};

pub struct MessageVariablesVisitor {
    variables: MessageVariables,
    plural_name_stack: Vec<SyntaxToken>,
}

impl MessageVariablesVisitor {
    pub fn new() -> Self {
        Self {
            variables: MessageVariables::new(),
            plural_name_stack: vec![],
        }
    }

    pub fn into_variables(self) -> MessageVariables {
        self.variables
    }
}

header_tag_lookup_map!(HEADER_VARIABLE_NAMES, get_header_variable_name, "${}");

impl Visit for MessageVariablesVisitor {
    fn visit_paragraph(&mut self, node: &Paragraph) {
        self.variables
            .add_instance("$p", MessageVariableType::HookFunction, true, None);
        node.visit_children_with(self);
    }

    fn visit_thematic_break(&mut self, _: &ThematicBreak) {
        self.variables
            .add_instance("$hr", MessageVariableType::HookFunction, true, None);
    }

    fn visit_any_heading(&mut self, heading: &AnyHeading) {
        let heading_tag = get_header_variable_name(heading.level());
        self.variables
            .add_instance(&heading_tag, MessageVariableType::HookFunction, true, None);
        heading.visit_children_with(self);
    }

    fn visit_any_code_block(&mut self, _code_block: &AnyCodeBlock) {
        // This presumes that code blocks can't contain variables, which _should_ always be true
        self.variables
            .add_instance("$codeBlock", MessageVariableType::HookFunction, true, None);
    }

    fn visit_text_span(&mut self, node: &TextSpan) {
        for child in node.children() {
            if matches!(
                child.kind(),
                SyntaxKind::HARD_LINE_ENDING | SyntaxKind::BACKSLASH_BREAK
            ) {
                self.variables
                    .add_instance("$br", MessageVariableType::HookFunction, true, None);
            }
        }
    }

    fn visit_emphasis(&mut self, node: &Emphasis) {
        self.variables
            .add_instance("$i", MessageVariableType::HookFunction, true, None);
        node.visit_children_with(self);
    }

    fn visit_strong(&mut self, node: &Strong) {
        self.variables
            .add_instance("$b", MessageVariableType::HookFunction, true, None);
        node.visit_children_with(self);
    }

    fn visit_link(&mut self, link: &Link) {
        self.variables.add_instance(
            "$link",
            MessageVariableType::LinkFunction,
            // Links themselves are builtins, since they define the
            // handling of the link tag itself, while the destination
            // or content may still contain user-defined variables.
            true,
            None,
        );
        link.visit_children_with(self);
    }

    fn visit_code_span(&mut self, _node: &CodeSpan) {
        self.variables
            .add_instance("$code", MessageVariableType::HookFunction, true, None);
    }

    fn visit_hook(&mut self, hook: &Hook) {
        let name = hook.name().name_token();
        self.variables.add_instance(
            name.text(),
            MessageVariableType::HookFunction,
            // Hooks are always user-defined.
            false,
            Some(name.span()),
        );
        hook.visit_children_with(self);
    }

    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        self.variables
            .add_instance("$del", MessageVariableType::HookFunction, true, None);
        node.visit_children_with(self);
    }

    fn visit_icu(&mut self, node: &Icu) {
        let ident = key_symbol(node.ident_token().text());
        node.visit_children_with(self);
        if !self.variables.has_key(&ident) {
            let icu_kind = node.value().syntax().kind();
            unreachable!("MessageVariablesVisitor passed over an ICU block for `{}` (kind: {:?}) without adding it as a variable instance", ident, icu_kind);
        }
    }

    fn visit_icu_pound(&mut self, node: &IcuPound) {
        let Some(plural_name) = self.plural_name_stack.last() else {
            unreachable!("Encountered IcuPound without a current plural variable name set.");
        };
        self.variables.add_instance(
            plural_name.text(),
            MessageVariableType::Number,
            false,
            Some(node.hash_token().span()),
        );
    }

    fn visit_click_handler_link_destination(&mut self, node: &ClickHandlerLinkDestination) {
        self.variables.add_instance(
            &node.name_token().text(),
            MessageVariableType::HandlerFunction,
            false,
            Some(node.name_token().span()),
        );
    }

    fn visit_icu_placeholder(&mut self, placeholder: &IcuPlaceholder) {
        let ident = placeholder.ident_token();
        self.variables.add_instance(
            ident.text(),
            MessageVariableType::Any,
            false,
            Some(ident.span()),
        );
    }

    fn visit_icu_plural(&mut self, plural: &IcuPlural) {
        let name = plural.ident_token();
        self.variables.add_instance(
            name.text(),
            MessageVariableType::Plural,
            false,
            Some(name.span()),
        );
        self.plural_name_stack.push(name.clone());
        plural.visit_children_with(self);
        self.plural_name_stack.pop();
    }

    fn visit_icu_select_ordinal(&mut self, node: &IcuSelectOrdinal) {
        let name = node.ident_token();
        self.variables.add_instance(
            name.text(),
            MessageVariableType::Plural,
            false,
            Some(name.span()),
        );
        self.plural_name_stack.push(name.clone());
        node.visit_children_with(self);
        self.plural_name_stack.pop();
    }

    fn visit_icu_select(&mut self, select: &IcuSelect) {
        let name = select.ident_token();
        let arms = select.arms();
        let mut selectors = vec![];
        let mut numeric_selectors = vec![];
        let mut has_other = false;
        let mut is_all_numeric = true;
        for arm in arms.children() {
            if arm.is_other_selector() {
                has_other = true;
                continue;
            }
            // If the value can be treated as a numeric literal, then it can be added to the
            // type set as a number. e.g., `{count, select, 1 {foo} 2 {bar}}` would yield the
            // enum `1 | 2 | "1" | "2"`, so that messages with these expression can be formatted
            // like `intl.format(message, {count: 1})` or `intl.format(message, {count: "2"})`.
            if let Ok(numeric_selector) = arm.selector_token().text().parse::<usize>() {
                numeric_selectors.push(numeric_selector);
            } else {
                is_all_numeric = false;
                selectors.push(arm.selector_token().text().to_string());
            }
        }

        let kind = if is_all_numeric {
            MessageVariableType::NumericEnum {
                values: numeric_selectors,
                allow_other: has_other,
            }
        } else {
            MessageVariableType::Enum {
                values: selectors,
                allow_other: has_other,
            }
        };

        self.variables
            .add_instance(name.text(), kind, false, Some(name.span()));
        self.plural_name_stack.push(name.clone());
        select.visit_children_with(self);
        self.plural_name_stack.pop();
    }

    fn visit_icu_date(&mut self, node: &IcuDate) {
        self.variables.add_instance(
            node.ident_token().text(),
            MessageVariableType::Date,
            false,
            Some(node.ident_token().span()),
        );
        node.visit_children_with(self);
    }

    fn visit_icu_time(&mut self, node: &IcuTime) {
        self.variables.add_instance(
            node.ident_token().text(),
            MessageVariableType::Time,
            false,
            Some(node.ident_token().span()),
        );
        node.visit_children_with(self);
    }

    fn visit_icu_number(&mut self, node: &IcuNumber) {
        self.variables.add_instance(
            node.ident_token().text(),
            MessageVariableType::Number,
            false,
            Some(node.ident_token().span()),
        );
        node.visit_children_with(self);
    }
}
