use crate::messages::FilePosition;
use intl_markdown::{parse_intl_message, Document};
use serde::Serialize;

use super::message_variables_visitor::{MessageVariables, MessageVariablesVisitor};

#[derive(Debug, Serialize)]
pub struct MessageValue {
    pub raw: String,
    pub parsed: Document,
    pub variables: Option<MessageVariables>,
    pub file_position: Option<FilePosition>,
}

impl MessageValue {
    /// Creates a new value including the original raw content as given and
    /// parsing the content to a compiled AST.
    pub fn from_raw(content: &str) -> Self {
        // By convention, messages starting immediately with a newline character are considered
        // blocks and have paragraphs parsed in their content, while normal messages with no leading
        // newline are treated as fully-inline.
        let include_blocks = content.starts_with("\n");
        let document = parse_intl_message(&content, include_blocks);

        let mut variables = MessageVariables::new();
        let variables = match MessageVariablesVisitor::visit(&document, &mut variables) {
            Ok(_) => Some(variables),
            _ => None,
        };

        Self {
            raw: content.into(),
            parsed: document,
            variables,
            file_position: None,
        }
    }

    pub fn with_file_position(mut self, position: FilePosition) -> Self {
        self.file_position = Some(position);
        self
    }
}

// Messages are equal if they have the same starting raw content. Everything
// else about a message is derived from that original string.
impl PartialEq for MessageValue {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
