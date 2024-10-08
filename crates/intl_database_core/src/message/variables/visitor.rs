use intl_markdown::{
    BlockNode, DEFAULT_TAG_NAMES, Document, Icu, InlineContent, TextOrPlaceholder,
};

use crate::database::symbol::key_symbol;
use crate::error::DatabaseResult;

use super::{MessageVariables, MessageVariableType};

pub struct MessageVariablesVisitor;

impl MessageVariablesVisitor {
    pub fn visit(ast: &Document, variables: &mut MessageVariables) -> DatabaseResult<()> {
        for child in ast.blocks() {
            Self::visit_block(child, variables)?;
        }
        Ok(())
    }

    fn visit_block(block_node: &BlockNode, variables: &mut MessageVariables) -> DatabaseResult<()> {
        match block_node {
            BlockNode::InlineContent(content) => Self::visit_inline_children(content, variables),
            BlockNode::Paragraph(paragraph) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.paragraph()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Self::visit_inline_children(paragraph.content(), variables)
            }
            BlockNode::Heading(heading) => {
                let heading_tag = DEFAULT_TAG_NAMES.heading(heading.level());
                variables.add_instance(
                    key_symbol(&heading_tag),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Self::visit_inline_children(heading.content(), variables)
            }
            // This presumes that code blocks can't contain variables, which _should_ always be true
            BlockNode::CodeBlock(_) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.code_block()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Ok(())
            }
            BlockNode::ThematicBreak => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.hr()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Ok(())
            }
        }
    }

    fn visit_inline_children(
        content: &Vec<InlineContent>,
        variables: &mut MessageVariables,
    ) -> DatabaseResult<()> {
        for child in content {
            Self::visit_inline_content(child, variables)?;
        }
        Ok(())
    }

    fn visit_inline_content(
        element: &InlineContent,
        variables: &mut MessageVariables,
    ) -> DatabaseResult<()> {
        match element {
            InlineContent::Text(_) => Ok(()),
            // # is just a reference to an existing outer variable. It doesn't add anything new.
            // TODO: Make this add an instance of the outer variable.
            InlineContent::IcuPound => Ok(()),
            InlineContent::Icu(icu) => Self::visit_icu(icu, variables),
            // Everything else introduces a new tag directly before checking the inner content.
            InlineContent::Emphasis(emphasis) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.emphasis()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Self::visit_inline_children(emphasis.content(), variables)
            }
            InlineContent::Strong(strong) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.strong()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Self::visit_inline_children(strong.content(), variables)
            }
            InlineContent::Strikethrough(strikethrough) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.strike_through()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Self::visit_inline_children(strikethrough.content(), variables)
            }
            InlineContent::HardLineBreak => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.br()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Ok(())
            }
            InlineContent::CodeSpan(_) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.code()),
                    MessageVariableType::HookFunction,
                    true,
                    None,
                );
                Ok(())
            }
            // Links and hooks introduce known variables.
            InlineContent::Hook(hook) => {
                variables.add_instance(
                    key_symbol(hook.name()),
                    MessageVariableType::HookFunction,
                    // Hooks are always user-defined.
                    false,
                    None,
                );
                Self::visit_inline_children(hook.content(), variables)
            }
            InlineContent::Link(link) => {
                variables.add_instance(
                    key_symbol(DEFAULT_TAG_NAMES.link()),
                    MessageVariableType::LinkFunction,
                    // Links themselves are builtins, since they define the
                    // handling of the link tag itself, while the destination
                    // or content may still contain user-defined variables.
                    true,
                    None,
                );
                Self::visit_inline_children(link.label(), variables)?;
                match link.destination() {
                    TextOrPlaceholder::Placeholder(icu) => Self::visit_icu(icu, variables),
                    TextOrPlaceholder::Text(_) => {
                        // When the link has a static text destination, an empty sentinel value is
                        // used to separate the destination from the content text. This is a special
                        // `$_` variable that must be provided at render time, so it counts as a
                        // variable for the message.
                        variables.add_instance(
                            key_symbol(DEFAULT_TAG_NAMES.empty()),
                            MessageVariableType::Any,
                            true,
                            None,
                        );
                        Ok(())
                    }
                    TextOrPlaceholder::Handler(handler_name) => {
                        variables.add_instance(
                            key_symbol(&handler_name),
                            MessageVariableType::HandlerFunction,
                            false,
                            None,
                        );
                        Ok(())
                    }
                }
            }
        }
    }

    fn visit_icu(icu: &Icu, variables: &mut MessageVariables) -> DatabaseResult<()> {
        match icu {
            Icu::IcuVariable(variable) => {
                variables.add_instance(
                    key_symbol(variable.name()),
                    MessageVariableType::Any,
                    false,
                    None,
                );
                Ok(())
            }
            Icu::IcuPlural(plural) => {
                variables.add_instance(
                    key_symbol(plural.name()),
                    MessageVariableType::Plural,
                    false,
                    None,
                );
                for arm in plural.arms() {
                    Self::visit_inline_children(arm.content(), variables)?;
                }
                Ok(())
            }
            Icu::IcuSelect(select) => {
                variables.add_instance(
                    key_symbol(select.name()),
                    // TODO(faulty): change this to ::Enum.
                    MessageVariableType::Plural,
                    false,
                    None,
                );
                for arm in select.arms() {
                    Self::visit_inline_children(arm.content(), variables)?;
                }
                Ok(())
            }
            Icu::IcuDate(date) => {
                variables.add_instance(
                    key_symbol(date.name()),
                    MessageVariableType::Date,
                    false,
                    None,
                );
                Ok(())
            }
            Icu::IcuTime(time) => {
                variables.add_instance(
                    key_symbol(time.name()),
                    MessageVariableType::Time,
                    false,
                    None,
                );
                Ok(())
            }
            Icu::IcuNumber(number) => {
                variables.add_instance(
                    key_symbol(number.name()),
                    MessageVariableType::Number,
                    false,
                    None,
                );
                Ok(())
            }
        }
    }
}
