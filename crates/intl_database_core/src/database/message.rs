use serde::Serialize;

use intl_message_utils::hash_message_key;

use crate::database::symbol::{KeySymbol, KeySymbolMap};
use crate::message::meta::MessageMeta;
use crate::message::value::MessageValue;
use crate::message::variables::MessageVariables;

/// Any message that is defined through `defineMessage` will be a `Normal`
/// message definition.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    /// Original, plain text name of the message given in its definition.
    key: KeySymbol,
    /// Hashed version of the key, used everywhere for minification and obfuscation.
    hashed_key: String,
    /// Map of all translations for this message, including the default.
    translations: KeySymbolMap<MessageValue>,
    /// The source definition information for this message (locale and location).
    source_locale: Option<KeySymbol>,
    /// Meta information about how to handle and process this message.
    meta: MessageMeta,
}

impl Message {
    pub fn from_definition(
        key: KeySymbol,
        value: MessageValue,
        source_locale: KeySymbol,
        meta: MessageMeta,
    ) -> Self {
        let mut message = Self {
            key,
            hashed_key: hash_message_key(&key),
            translations: KeySymbolMap::default(),
            source_locale: Some(source_locale),
            meta,
        };
        message.translations.insert(source_locale, value);
        message
    }

    pub fn from_translation(key: KeySymbol, locale: KeySymbol, value: MessageValue) -> Self {
        let mut message = Self {
            key,
            hashed_key: hash_message_key(&key),
            translations: KeySymbolMap::default(),
            source_locale: None,
            meta: MessageMeta::default(),
        };
        message.translations.insert(locale, value);
        message
    }

    //#region Accessors
    pub fn translations(&self) -> &KeySymbolMap<MessageValue> {
        &self.translations
    }
    pub fn key(&self) -> KeySymbol {
        self.key
    }
    pub fn hashed_key(&self) -> &String {
        &self.hashed_key
    }
    pub fn source_locale(&self) -> &Option<KeySymbol> {
        &self.source_locale
    }
    pub fn meta(&self) -> &MessageMeta {
        &self.meta
    }
    //#endregion

    //#region Mutation

    /// Create or update the definition for this message with the given information.
    pub fn set_definition(&mut self, source: MessageValue, locale: KeySymbol, meta: MessageMeta) {
        self.translations.insert(locale, source);
        self.source_locale = Some(locale);
        self.meta = meta;
    }

    /// Removes the source definition of this message, including both the translation and the
    /// source locale key, which are returned as a tuple from this method if they were present.
    pub fn remove_definition(&mut self) -> (Option<MessageValue>, Option<KeySymbol>) {
        let translation = match &self.source_locale {
            Some(locale) => self.translations.remove(locale),
            None => None,
        };
        (translation, self.source_locale.take())
    }

    pub fn set_translation(&mut self, locale: KeySymbol, value: MessageValue) {
        self.translations.insert(locale, value);
    }

    pub fn remove_translation(&mut self, locale: KeySymbol) -> Option<MessageValue> {
        self.translations.remove(&locale)
    }

    //#endregion

    //#region Queries
    pub fn is_defined(&self) -> bool {
        self.source_locale.is_some()
    }

    /// Return a direct reference to the definition of this message.
    ///
    /// Panics if there is no entry for the source locale of the message. Use [Self::is_defined] to
    /// check for the definition before calling this method.
    pub fn definition(&self) -> &MessageValue {
        self.get_source_translation()
            .expect("Message should be defined when calling `.definition()`")
    }

    /// Return the translation entry for the default locale for this message.
    pub fn get_source_translation(&self) -> Option<&MessageValue> {
        self.source_locale
            .as_ref()
            .and_then(|locale| self.translations.get(locale))
    }

    /// Returns true if the definition of this message _does not_ contain any dynamic variables,
    /// meaning it can be treated as a static string and bypass any extra processing.
    pub fn is_static_definition(&self) -> bool {
        !self
            .source_variables()
            .as_ref()
            .is_some_and(|variables| variables.len() > 0)
    }

    /// Returns a set of variables present in the source translation of this message.
    pub fn source_variables(&self) -> Option<&MessageVariables> {
        match self.get_source_translation() {
            Some(translation) => translation.variables.as_ref(),
            _ => None,
        }
    }

    /// Returns a merged set of all variables present in the message, across all translations.
    pub fn all_variables(&self) -> MessageVariables {
        let mut merged = self
            .source_variables()
            .map_or_else(|| MessageVariables::new(), Clone::clone);

        for (_, translation) in self.translations() {
            match &translation.variables {
                Some(variables) => {
                    merged.merge(variables);
                }
                None => continue,
            }
        }

        merged
    }

    //#endregion
}
