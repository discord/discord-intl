use crate::{JsonMessage, JsonPosition};

use napi_derive::napi;

// Use the mimalloc allocator explicitly when building the node addon.
extern crate intl_allocator;

#[napi(object)]
pub struct Position {
    pub line: u32,
    pub col: u32,
}

impl From<JsonPosition> for Position {
    fn from(pos: JsonPosition) -> Self {
        Self {
            line: pos.line,
            col: pos.col,
        }
    }
}

#[napi(object)]
pub struct Message {
    pub key: String,
    pub value: String,
    pub position: Position,
}

impl From<JsonMessage> for Message {
    fn from(msg: JsonMessage) -> Self {
        Self {
            key: msg.key.to_string(),
            value: msg.value.to_string(),
            position: msg.position.into(),
        }
    }
}

#[napi]
pub fn parse_json(text: String) -> napi::Result<Vec<Message>> {
    let messages = crate::parse_flat_translation_json(&text);
    Ok(messages.map(Message::from).collect())
}

#[napi]
pub fn parse_json_file(file_path: String) -> napi::Result<Vec<Message>> {
    let content = std::fs::read_to_string(&file_path)?;
    parse_json(content)
}
