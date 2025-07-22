use super::{MessageVariableType, MessageVariables};
use crate::database::symbol::key_symbol;
use crate::KeySymbol;
use intl_markdown::{
    AnyCodeBlock, AnyHeading, ClickHandlerLinkDestination, CodeSpan, Emphasis, Hook, IcuDate,
    IcuNumber, IcuPlural, IcuPound, IcuSelect, IcuTime, IcuVariable, Link, Paragraph,
    Strikethrough, Strong, SyntaxKind, TextSpan, ThematicBreak, Visit, VisitWith,
};
use intl_markdown_macros::header_tag_lookup_map;

pub struct MessageVariablesVisitor {
    variables: MessageVariables,
    current_plural_variable_name: Option<KeySymbol>,
    current_variable_type: Option<MessageVariableType>,
}

impl MessageVariablesVisitor {
    pub fn new() -> Self {
        Self {
            variables: MessageVariables::new(),
            current_plural_variable_name: None,
            current_variable_type: None,
        }
    }

    pub fn into_variables(self) -> MessageVariables {
        self.variables
    }
}

header_tag_lookup_map!(HEADER_VARIABLE_NAMES, get_header_variable_name, "${}");

impl Visit for MessageVariablesVisitor {
    fn visit_paragraph(&mut self, node: &Paragraph) {
        self.variables.add_instance(
            key_symbol("$p"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_thematic_break(&mut self, _: &ThematicBreak) {
        self.variables.add_instance(
            key_symbol("$hr"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_any_heading(&mut self, heading: &AnyHeading) {
        let heading_tag = get_header_variable_name(heading.level());
        self.variables.add_instance(
            key_symbol(&heading_tag),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        heading.visit_children_with(self);
    }

    fn visit_any_code_block(&mut self, _code_block: &AnyCodeBlock) {
        // This presumes that code blocks can't contain variables, which _should_ always be true
        self.variables.add_instance(
            key_symbol("$codeBlock"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_text_span(&mut self, node: &TextSpan) {
        for child in node.children() {
            if matches!(
                child.kind(),
                SyntaxKind::HARD_LINE_ENDING | SyntaxKind::BACKSLASH_BREAK
            ) {
                self.variables.add_instance(
                    key_symbol("$br"),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
            }
        }
    }

    fn visit_emphasis(&mut self, node: &Emphasis) {
        self.variables.add_instance(
            key_symbol("$i"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_strong(&mut self, node: &Strong) {
        self.variables.add_instance(
            key_symbol("$b"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_link(&mut self, link: &Link) {
        self.variables.add_instance(
            key_symbol("$link"),
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
        self.variables.add_instance(
            key_symbol("$code"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_hook(&mut self, hook: &Hook) {
        self.variables.add_instance(
            key_symbol(hook.name().name_token().text()),
            MessageVariableType::HookFunction,
            // Hooks are always user-defined.
            false,
            None,
        );
        hook.visit_children_with(self);
    }

    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        self.variables.add_instance(
            key_symbol("$del"),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_icu_pound(&mut self, _: &IcuPound) {
        debug_assert!(
            self.current_plural_variable_name.is_some(),
            "Encountered IcuPound without a current plural variable name set."
        );
        self.variables.add_instance(
            self.current_plural_variable_name.unwrap(),
            MessageVariableType::Number,
            false,
            None,
        );
    }

    fn visit_click_handler_link_destination(&mut self, node: &ClickHandlerLinkDestination) {
        self.variables.add_instance(
            key_symbol(&node.name_token().text()),
            MessageVariableType::HandlerFunction,
            false,
            None,
        );
    }

    fn visit_icu_variable(&mut self, variable: &IcuVariable) {
        self.variables.add_instance(
            key_symbol(variable.ident_token().text()),
            self.current_variable_type
                .take()
                .unwrap_or(MessageVariableType::Any),
            false,
            None,
        );
    }

    fn visit_icu_plural(&mut self, plural: &IcuPlural) {
        let name_symbol = key_symbol(plural.variable().ident_token().text());
        self.current_plural_variable_name = Some(name_symbol);
        self.variables
            .add_instance(name_symbol, MessageVariableType::Plural, false, None);
        plural.visit_children_with(self);
    }

    fn visit_icu_select(&mut self, select: &IcuSelect) {
        let name_symbol = key_symbol(select.variable().ident_token().text());
        self.current_plural_variable_name = Some(name_symbol);
        let arms = select.arms();
        let selectors = arms
            .children()
            .map(|arm| arm.selector_token().text().to_string());
        self.current_variable_type = Some(MessageVariableType::Enum(selectors.collect()));
        select.visit_children_with(self);
    }

    fn visit_icu_date(&mut self, date: &IcuDate) {
        self.current_variable_type = Some(MessageVariableType::Date);
        date.visit_children_with(self);
    }

    fn visit_icu_time(&mut self, time: &IcuTime) {
        self.current_variable_type = Some(MessageVariableType::Time);
        time.visit_children_with(self);
    }

    fn visit_icu_number(&mut self, number: &IcuNumber) {
        self.current_variable_type = Some(MessageVariableType::Number);
        number.visit_children_with(self);
    }
}
