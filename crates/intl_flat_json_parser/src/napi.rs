use crate::{JsonMessage, JsonPosition};
use napi::{Env, JsObject, Property};
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

fn collect_messages(
    env: Env,
    iterator: impl Iterator<Item = JsonMessage>,
) -> napi::Result<Vec<JsObject>> {
    // This is an arbitrary size hint that should be suitable for a lot of use
    // cases. While it may inadvertently allocate extra memory for some,
    // avoiding repeated re-allocations that we're pretty confident will happen
    // ends up saving a lot more time in the end.
    let mut result = Vec::with_capacity(1024);
    // NAPI does not have an API for creating multiple instances of the same
    // object structure, but we can get as close as possible by pre-defining
    // object properties and cloning them to avoid allocating extra space for
    // the same key many times over.
    // See https://github.com/nodejs/node/issues/45905 for future updates.
    let line_prop = Property::new("line")?;
    let col_prop = Property::new("col")?;
    let key_prop = Property::new("key")?;
    let value_prop = Property::new("value")?;
    let position_prop = Property::new("position")?;
    for message in iterator {
        let key = env.create_string(&message.key)?;
        let value = env.create_string(&message.value)?;
        let mut position = env.create_object()?;
        position.define_properties(&[
            line_prop
                .clone()
                .with_value(&env.create_uint32(message.position.line)?),
            col_prop
                .clone()
                .with_value(&env.create_uint32(message.position.col)?),
        ])?;

        let mut converted = env.create_object()?;
        converted.define_properties(&[
            key_prop.clone().with_value(&key),
            value_prop.clone().with_value(&value),
            position_prop.clone().with_value(&position),
        ])?;

        result.push(converted);
    }

    Ok(result)
}

#[napi(object)]
pub struct Message {
    pub key: String,
    pub value: String,
    pub position: Position,
}

#[napi(ts_return_type = "Message[]")]
pub fn parse_json(env: Env, text: String) -> napi::Result<Vec<JsObject>> {
    let messages = crate::parse_flat_translation_json(&text);
    // Ok(messages.map(Message::from).collect())
    Ok(collect_messages(env, messages)?)
}

#[napi]
pub fn parse_json_file(env: Env, file_path: String) -> napi::Result<Vec<JsObject>> {
    let content = std::fs::read_to_string(&file_path)?;
    parse_json(env, content)
}
