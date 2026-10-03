mod parser;
mod util;

pub use parser::{JsonMessage, JsonPosition, TranslationsJsonParser};

#[cfg(not(feature = "node_addon"))]
pub mod napi;

/// Parse the given `text` as a single, flat JSON object of message keys to
/// message values. The JSON is assumed to be well-formed, and minimal error
/// handling is implemented.
///
/// Since sources are able to accept an iterator of translation values, this
/// parser never stores the completed object in memory and instead yields each
/// translation as it is parsed.
///
/// This parser also handles tracking line and column positions within the
/// source, which is the primary reason for this implementation over existing
/// libraries like serde that only track that state internally.
///
/// Note that some extra assumptions are made here for the sake of simplicity
/// and efficient parsing that other implementations aren't able to make:
/// - The given text is a well-formed, flat JSON object
/// - Keys may not contain escaped quotes, `}`, or newlines.
/// - The only important positional information is the first character of the value.
/// - There will be no errors during parsing. The iterator will return None instead.
pub fn parse_flat_translation_json(text: &str) -> impl Iterator<Item = JsonMessage> + use<'_> {
    let mut parser = TranslationsJsonParser::new(text);
    parser.parse_start();
    parser.into_iter()
}
