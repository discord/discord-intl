use crate::JsonMessage;
use napi::bindgen_prelude::Array;
use napi::Env;
use napi_derive::napi;

// Use the mimalloc allocator explicitly when building the node addon.
extern crate intl_allocator;

#[napi]
pub struct Message {
    pub key: String,
    pub value: String,
    // The Position properties are created directly inline on the message
    // because NAPI object construction is pretty slow (like 10x slower than
    // the engine creating an object directly), so moving these to direct
    // properties of the class makes the whole process ~30-35% faster.
    //
    // See https://github.com/nodejs/node/issues/45905 for future updates.
    pub line: u32,
    pub col: u32,
}

fn collect_messages(env: Env, iterator: impl Iterator<Item = JsonMessage>) -> napi::Result<Array> {
    // This is an arbitrary size hint that should be suitable for a lot of use
    // cases. While it may inadvertently allocate extra memory for some,
    // avoiding repeated re-allocations that we're pretty confident will happen
    // ends up saving a lot more time in the end.
    let mut result = env.create_array(1024)?;
    let mut index = 0;
    for message in iterator {
        result.set(
            index,
            Message {
                key: message.key.to_string(),
                value: message.value.to_string(),
                line: message.position.line,
                col: message.position.col,
            },
        )?;
        index += 1;
    }

    Ok(result)
}

#[napi(ts_return_type = "Message[]")]
pub fn parse_json(env: Env, text: String) -> napi::Result<Array> {
    let messages = crate::parse_flat_translation_json(&text);
    Ok(collect_messages(env, messages)?)
}

#[napi(ts_return_type = "Message[]")]
pub fn parse_json_file(env: Env, file_path: String) -> napi::Result<Array> {
    let content = std::fs::read_to_string(&file_path)?;
    parse_json(env, content)
}
