use serde::Serialize;

use super::source_file::FilePosition;
use super::variables::{collect_message_variables, MessageVariables};
use crate::SourceOffsetList;
use intl_markdown::compiler::CompiledElement;
use intl_markdown::{parse_intl_message, AnyDocument};
use intl_message_utils::message_may_have_blocks;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageValue {
    pub raw: String,
    pub parsed: CompiledElement,
    pub variables: MessageVariables,
    pub file_position: FilePosition,
    #[serde(skip)]
    pub cst: AnyDocument,
    #[serde(skip)]
    pub source_offsets: SourceOffsetList,
}

impl MessageValue {
    /// Creates a new value including the original raw content as given and
    /// parsing the content to a compiled AST.
    pub fn from_raw(
        content: &str,
        file_position: FilePosition,
        source_offsets: SourceOffsetList,
    ) -> Self {
        let document = parse_intl_message(&content, message_may_have_blocks(content));

        Self {
            raw: content.into(),
            parsed: document.compiled,
            variables: collect_message_variables(&document.cst),
            file_position,
            cst: document.cst,
            source_offsets,
        }
    }
}

// Messages are equal if they have the same starting raw content. Everything
// else about a message is derived from that original string.
impl PartialEq for MessageValue {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}
