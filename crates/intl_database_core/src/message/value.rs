use serde::Serialize;

use intl_markdown::{parse_intl_message, Document};
use intl_message_utils::message_may_have_blocks;

use super::source_file::FilePosition;
use super::variables::{collect_message_variables, MessageVariables};

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
        let document = parse_intl_message(&content, message_may_have_blocks(content));

        let variables = match collect_message_variables(&document) {
            Ok(variables) => Some(variables),
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
