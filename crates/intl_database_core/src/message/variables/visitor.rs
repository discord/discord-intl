use intl_markdown::{
    CodeBlock, CodeSpan, DEFAULT_TAG_NAMES, Emphasis, Heading, Hook, IcuDate, IcuNumber, IcuPlural,
    IcuSelect, IcuTime, IcuVariable, Link, Paragraph, Strikethrough, Strong,
    TextOrPlaceholder,
};
use intl_markdown_visitor::{Visit, VisitWith};

use crate::database::symbol::key_symbol;
use crate::KeySymbol;

use super::{MessageVariables, MessageVariableType};

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

impl Visit for MessageVariablesVisitor {
    fn visit_code_block(&mut self, _code_block: &CodeBlock) {
        // This presumes that code blocks can't contain variables, which _should_ always be true
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.code_block()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_code_span(&mut self, _node: &CodeSpan) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.code()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_emphasis(&mut self, node: &Emphasis) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.emphasis()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_heading(&mut self, heading: &Heading) {
        let heading_tag = DEFAULT_TAG_NAMES.heading(heading.level());
        self.variables.add_instance(
            key_symbol(&heading_tag),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        heading.visit_children_with(self);
    }

    fn visit_hook(&mut self, hook: &Hook) {
        self.variables.add_instance(
            key_symbol(hook.name()),
            MessageVariableType::HookFunction,
            // Hooks are always user-defined.
            false,
            None,
        );
        hook.visit_children_with(self);
    }

    fn visit_icu_date(&mut self, date: &IcuDate) {
        self.current_variable_type = Some(MessageVariableType::Date);
        date.visit_children_with(self);
    }

    fn visit_icu_number(&mut self, number: &IcuNumber) {
        self.current_variable_type = Some(MessageVariableType::Number);
        number.visit_children_with(self);
    }

    fn visit_icu_plural(&mut self, plural: &IcuPlural) {
        let name_symbol = key_symbol(plural.name());
        self.current_plural_variable_name = Some(name_symbol);
        self.variables
            .add_instance(name_symbol, MessageVariableType::Plural, false, None);
        plural.visit_children_with(self);
    }

    fn visit_icu_select(&mut self, select: &IcuSelect) {
        let name_symbol = key_symbol(select.name());
        self.current_plural_variable_name = Some(name_symbol);
        // TODO(faulty): change this to ::Enum.
        self.current_variable_type = Some(MessageVariableType::Plural);
        select.visit_children_with(self);
    }

    fn visit_icu_time(&mut self, time: &IcuTime) {
        self.current_variable_type = Some(MessageVariableType::Time);
        time.visit_children_with(self);
    }

    fn visit_icu_variable(&mut self, variable: &IcuVariable) {
        self.variables.add_instance(
            key_symbol(variable.name()),
            self.current_variable_type
                .take()
                .unwrap_or(MessageVariableType::Any),
            false,
            None,
        );
    }

    fn visit_link(&mut self, link: &Link) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.link()),
            MessageVariableType::LinkFunction,
            // Links themselves are builtins, since they define the
            // handling of the link tag itself, while the destination
            // or content may still contain user-defined variables.
            true,
            None,
        );
        link.visit_children_with(self);
    }

    fn visit_link_destination(&mut self, node: &TextOrPlaceholder) {
        match node {
            TextOrPlaceholder::Text(_) => {
                // When the link has a static text destination, an empty sentinel value is
                // used to separate the destination from the content text. This is a special
                // `$_` variable that must be provided at render time, so it counts as a
                // variable for the message.
                self.variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.empty()),
                    MessageVariableType::Any,
                    true,
                    None,
                );
            }
            TextOrPlaceholder::Handler(handler_name) => {
                self.variables.add_instance(
                    key_symbol(&handler_name),
                    MessageVariableType::HandlerFunction,
                    false,
                    None,
                );
            }
            TextOrPlaceholder::Placeholder(_) => node.visit_children_with(self),
        }
    }

    fn visit_paragraph(&mut self, node: &Paragraph) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.paragraph()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_strikethrough(&mut self, node: &Strikethrough) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.strike_through()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_strong(&mut self, node: &Strong) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.strong()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
        node.visit_children_with(self);
    }

    fn visit_thematic_break(&mut self) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.hr()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_hard_line_break(&mut self) {
        self.variables.add_instance(
            key_symbol(DEFAULT_TAG_NAMES.br()),
            MessageVariableType::HookFunction,
            true,
            None,
        );
    }

    fn visit_icu_pound(&mut self) {
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
}
