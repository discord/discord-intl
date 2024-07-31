use serde::de::{Deserialize, Deserializer, Error, MapAccess, Visitor};
use std::borrow::Cow;

use crate::messages::{
    global_intern_string, KeySymbol, MessageValue, MessagesError, MessagesResult,
};

/// A temporary type representing a string key and its parsed value from a
/// translation file. This structure can directly deserialize a translation
/// file into a list of entries, avoiding a temporary HashMap allocation.
pub struct TranslationEntry {
    pub key: KeySymbol,
    pub value: MessageValue,
    pub start_offset: usize,
}

/// Custom deserialization visitor that converts a map like {"key": "value"}
/// into a vector of entries [TranslationEntry(key, value)]. This is much more
/// efficient than reading the file as plain JSON into a Map, then iterating
/// the map to create another map of parsed message values, then returning that
/// and iterating _again_ to insert into the database.
struct TranslationEntryVisitor;

impl<'de> Visitor<'de> for TranslationEntryVisitor {
    type Value = Translations;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(r#"Translation entry ("key": "value")"#)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Translations, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut entries: Vec<TranslationEntry> = vec![];
        loop {
            let entry: Option<(&str, Cow<str>)> = map.next_entry()?;
            if entry.is_none() {
                break;
            }
            let (key, value) = entry.unwrap();

            let key = global_intern_string(key)
                .map_err(|_| Error::custom("Failed to read the global symbol store"))?;
            entries.push(TranslationEntry {
                key,
                value: MessageValue::from_raw(&value),
                start_offset: 0,
            })
        }

        Ok(Translations { entries })
    }
}

/// A newtype wrapping a Vec of translations so that we can validly create the
/// custom Deserialize implementation below.
pub struct Translations {
    pub entries: Vec<TranslationEntry>,
}

impl<'de> Deserialize<'de> for Translations {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TranslationEntryVisitor)
    }
}

/// Parse the given content string as JSON, interpreting each entry in the
/// object as a new Translation definition.
pub fn extract_message_translations(content: &str) -> MessagesResult<Translations> {
    serde_json::from_str(content).map_err(MessagesError::TranslationDeserializationError)
}
