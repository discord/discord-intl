use std::ops::Range;

use crate::messages::symbols::KeySymbolMap;
use serde::Serialize;

use super::{symbols::KeySymbol, value::MessageValue, FilePosition};

/// A single message value in a specific locale.
#[derive(Debug, PartialEq, Serialize)]
pub struct Translation {
    /// Content of the message in the locale
    pub value: MessageValue,
    /// File where the translation is loaded from
    file_position: FilePosition,
}

impl Translation {
    pub fn new(value: MessageValue, file_position: FilePosition) -> Self {
        Self {
            value,
            file_position,
        }
    }

    /// Return the byte span of the entire message value within the source file.
    pub fn full_span(&self) -> Range<usize> {
        let start = self.file_position.offset as usize;
        let length = self.value.raw.len();
        start..(start + length)
    }

    /// Given an offset within the message value, return an absolute position
    /// to that same offset in the message within the source file.
    #[inline]
    pub fn file_position_for_offset(&self, offset: u32) -> u32 {
        self.file_position.offset + offset
    }

    /// Return the key symbol of the file this translation was sourced from.
    pub fn file(&self) -> KeySymbol {
        self.file_position.file
    }

    /// Return the byte offset of this translation within its source file.
    pub fn offset(&self) -> u32 {
        self.file_position.offset
    }
}

/// A Map of locales to the value for a message in that locale.
pub type TranslationsMap = KeySymbolMap<Translation>;

pub fn create_translation_map(
    default_value: MessageValue,
    default_locale: KeySymbol,
    file_position: FilePosition,
) -> TranslationsMap {
    let mut translations = TranslationsMap::default();
    translations.insert(
        default_locale,
        Translation::new(default_value, file_position),
    );
    translations
}
