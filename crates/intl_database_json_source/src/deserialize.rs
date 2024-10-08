use std::borrow::Cow;
use std::vec::IntoIter;

use serde::de::{Deserialize, Deserializer, MapAccess, Visitor};

use intl_database_core::{key_symbol, RawMessageTranslation};

/// Custom deserialization visitor that converts a map like {"key": "value"} into a vector of
/// entries [RawMessageTranslation]. This is much more efficient than reading the file as plain
/// JSON into a Map, then iterating the map to create another map of parsed message values, then
/// returning that and iterating _again_ to insert into the database.
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
        let mut entries: Vec<RawMessageTranslation> = vec![];
        loop {
            let entry: Option<(&str, Cow<str>)> = map.next_entry()?;
            if entry.is_none() {
                break;
            }
            let (key, value) = entry.unwrap();

            entries.push(RawMessageTranslation::new(key_symbol(key), 0u32, value))
        }

        Ok(Translations { entries })
    }
}

/// A newtype wrapping a Vec of translations so that we can validly create the
/// custom Deserialize implementation below.
pub struct Translations {
    pub entries: Vec<RawMessageTranslation>,
}

impl IntoIterator for Translations {
    type Item = RawMessageTranslation;
    type IntoIter = IntoIter<RawMessageTranslation>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'de> Deserialize<'de> for Translations {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(TranslationEntryVisitor)
    }
}
