use rustc_hash::FxHashMap;

use crate::error::{DatabaseError, DatabaseResult};
use crate::message::meta::MessageMeta;
use crate::message::source_file::SourceFile;
use crate::message::value::MessageValue;

use self::message::Message;
use self::symbol::{get_key_symbol, key_symbol, KeySymbol, KeySymbolMap, KeySymbolSet};

pub mod message;
pub mod source;
pub mod symbol;

#[derive(Copy, Clone)]
pub enum DatabaseInsertStrategy {
    /// The insertion represents a source file being processed for the first time. No messages
    /// defined in the file should exist in the database yet, and any duplicates are considered
    /// errors that will be skipped and logged as failures.
    NewSourceFile,
    /// The insertion represents a source file that has already been processed and is now being
    /// updated with new content. Messages are allowed to exist already, but only definitions
    /// originating from the same source file will be overwritten. Keys with definitions in other
    /// source files will still be skipped and logged as failures.
    UpdateSourceFile,
    /// The insertion represents an update that should overwrite any existing value, regardless of
    /// which source file it originates from. This is best used for importing new translations from
    /// a remote source like a translation vendor, where the updates come from external files and
    /// need to be applied and then re-written to the existing source files.
    ReplaceExisting,
}

impl DatabaseInsertStrategy {
    #[inline]
    pub fn allows_same_file_replacement(&self) -> bool {
        match self {
            DatabaseInsertStrategy::NewSourceFile => false,
            DatabaseInsertStrategy::UpdateSourceFile => true,
            DatabaseInsertStrategy::ReplaceExisting => true,
        }
    }

    #[inline]
    pub fn allows_any_replacement(&self) -> bool {
        match self {
            DatabaseInsertStrategy::NewSourceFile => false,
            DatabaseInsertStrategy::UpdateSourceFile => false,
            DatabaseInsertStrategy::ReplaceExisting => true,
        }
    }

    /// Returns true if the strategy allows replacing a value from `old_file`
    /// with a value from `new_file`.
    pub fn can_replace(&self, old_file: KeySymbol, new_file: KeySymbol) -> bool {
        if old_file == new_file {
            self.allows_same_file_replacement()
        } else {
            self.allows_any_replacement()
        }
    }
}

#[derive(Debug)]
pub struct MessagesDatabase {
    pub messages: KeySymbolMap<Message>,
    pub sources: KeySymbolMap<SourceFile>,
    pub hash_lookup: FxHashMap<String, KeySymbol>,
    pub known_locales: KeySymbolSet,
}

impl MessagesDatabase {
    pub fn new() -> Self {
        Self {
            messages: KeySymbolMap::default(),
            sources: KeySymbolMap::default(),
            hash_lookup: FxHashMap::default(),
            known_locales: KeySymbolSet::default(),
        }
    }

    /// Return the complete message definition under a given key.
    pub fn get_message(&self, key: &str) -> Option<&Message> {
        get_key_symbol(key).and_then(|symbol| self.messages.get(&symbol))
    }

    //#region Source Files

    pub fn get_source_file(&self, file_key: KeySymbol) -> Option<&SourceFile> {
        self.sources.get(&file_key)
    }

    /// Insert the given `source_file` to the sources map of this database.
    pub fn create_source_file(
        &mut self,
        file_key: KeySymbol,
        source_file: SourceFile,
    ) -> &SourceFile {
        self.sources.insert(file_key, source_file);
        &self.sources[&file_key]
    }

    pub fn get_or_create_source_file(
        &mut self,
        file_key: KeySymbol,
        source_file: SourceFile,
    ) -> &SourceFile {
        if self.get_source_file(file_key).is_none() {
            self.create_source_file(file_key, source_file);
        }
        self.get_source_file(file_key).unwrap()
    }

    /// Immediately replace list of message keys owned by the given source file with the given set
    /// of keys. File membership is not updates when processing messages and must be applied after
    /// the fact using this method.
    pub fn set_source_file_keys(
        &mut self,
        file_key: KeySymbol,
        keys: KeySymbolSet,
    ) -> DatabaseResult<()> {
        self.sources
            .get_mut(&file_key)
            .map(|source| source.set_message_keys(keys))
            .ok_or(DatabaseError::UnknownSourceFile(file_key))
    }

    /// Return an iterator over all of the message values owned by the given
    /// source file. The returned values are Options of references to the
    /// message for each key. Keys with no value will still be returned in the
    /// map, but with their values as None.
    pub fn get_source_file_message_values(
        &self,
        file_key: KeySymbol,
    ) -> DatabaseResult<impl Iterator<Item = (&KeySymbol, Option<&MessageValue>)>> {
        let source = self
            .get_source_file(file_key)
            .ok_or(DatabaseError::UnknownSourceFile(file_key))?;

        let source_locale = match source {
            SourceFile::Definition(_) => None,
            SourceFile::Translation(translation) => Some(translation.locale()),
        };

        Ok(source.message_keys().into_iter().map(move |key| {
            let Some(message) = self.get_message(key) else {
                return (key, None);
            };

            let value = match source_locale {
                Some(locale) => message.translations().get(&locale),
                _ => message.get_source_translation(),
            };

            (key, value)
        }))
    }

    //#endregion

    //#region Definitions

    /// Insert a new message definition into the database. If a definition with the same key already
    /// exists and the given `strategy` does not allow replacement, this method will return an Error
    /// that the message is already defined and cannot be replaced. Otherwise, as long as the
    /// definition is from the same source file, or if no definition exists, this method will update
    /// the message with the given definition.
    ///
    /// Currently, no strategy will allow a definition to replace another from a _different_ source
    /// file. Such cases will _always_ return an error to be handled elsewhere.
    pub fn insert_definition(
        &mut self,
        name: &str,
        value: MessageValue,
        locale: KeySymbol,
        meta: MessageMeta,
        strategy: DatabaseInsertStrategy,
    ) -> DatabaseResult<&Message> {
        let key = key_symbol(name);
        match self.messages.get_mut(&key) {
            Some(existing) => {
                if existing.is_defined() {
                    let definition = existing.definition();
                    let old_file = definition.file_position.file;
                    let new_file = value.file_position.file;

                    if !strategy.can_replace(old_file, new_file) {
                        return Err(DatabaseError::AlreadyDefined {
                            name: key,
                            existing: definition.clone(),
                            replacement: value,
                        });
                    }
                }

                existing.set_definition(value, locale, meta);
            }
            _ => {
                // Otherwise this is an entirely new message that gets created.
                let message = Message::from_definition(key, value, locale, meta);
                self.known_locales.insert(locale);
                self.hash_lookup.insert(message.hashed_key().clone(), key);
                self.messages.insert(key, message);
            }
        }
        Ok(&self.messages[&key])
    }

    /// If a message with the given `message_key` exists and has a source definition from the file
    /// with the given `file_key`, remove only the definition from the database. If there are
    /// existing translations for that message, they are preserved and the definition becomes
    /// Undefined. Otherwise, if there are no other translations, the message is removed entirely.
    pub fn remove_definition(&mut self, message_key: KeySymbol) -> Option<MessageValue> {
        self.messages
            .get_mut(&message_key)
            .and_then(|message| message.remove_definition().0)
    }

    //#endregion

    //#region Translations

    /// Insert a new message definition into the database. If a Normal
    /// entry with the same key already exists, this method will return an
    /// Error. However, if the existing entry is Undefined, this method will
    /// convert that entry to a Normal entry and return Ok.
    pub fn insert_translation(
        &mut self,
        key: KeySymbol,
        locale: KeySymbol,
        value: MessageValue,
        strategy: DatabaseInsertStrategy,
    ) -> DatabaseResult<&Message> {
        match self.messages.get_mut(&key) {
            // If the key has an existing message at all, it just gets a new
            // translation entry in the map. The type of the entry does not
            // change here.
            Some(existing) => {
                if let Some(translation) = existing.translations().get(&locale) {
                    let old_file = translation.file_position.file;
                    let new_file = value.file_position.file;
                    if !strategy.can_replace(old_file, new_file) {
                        return Err(DatabaseError::TranslationAlreadySet {
                            name: key,
                            locale,
                            existing: translation.clone(),
                            replacement: value,
                        });
                    }
                }

                self.known_locales.insert(locale);
                existing.set_translation(locale, value);
            }
            // If it doesn't already exist, add a new message to hold the translation until a
            // definition is found.
            _ => {
                // Otherwise this is an entirely new message that gets created.
                let message = Message::from_translation(key, locale, value);
                self.known_locales.insert(locale);
                self.hash_lookup.insert(message.hashed_key().clone(), key);
                self.messages.insert(key, message);
            }
        }

        Ok(&self.messages[&key])
    }

    pub fn remove_translation(
        &mut self,
        message_key: KeySymbol,
        locale: KeySymbol,
    ) -> Option<MessageValue> {
        self.messages
            .get_mut(&message_key)
            .and_then(|message| message.remove_translation(locale))
    }

    //#endregion
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use intl_message_utils::RUNTIME_PACKAGE_NAME;

    use crate::database::MessagesDatabase;

    fn new_database() -> MessagesDatabase {
        MessagesDatabase::new()
    }

    #[derive(Clone)]
    pub struct SourceMessagesBuilder {
        meta: Option<String>,
        messages: Vec<(String, String)>,
    }

    impl SourceMessagesBuilder {
        fn new() -> Self {
            Self {
                meta: None,
                messages: vec![],
            }
        }

        fn with_meta(mut self, meta: Option<String>) -> Self {
            self.meta = meta;
            self
        }

        fn with_message(mut self, key: &str, value: &str) -> Self {
            self.messages.push((key.into(), value.into()));
            self
        }

        fn remove_message(mut self, key: &str) -> Self {
            self.messages = Vec::from_iter(self.messages.into_iter().filter(|(k, _)| *k != key));
            self
        }

        fn count(&self) -> usize {
            self.messages.len()
        }

        fn to_definitions(&self) -> String {
            let mut buffer = String::new();
            buffer.push_str("import {defineMessages} from '");
            buffer.push_str(RUNTIME_PACKAGE_NAME);
            buffer.push_str("';\n");

            if let Some(meta) = &self.meta {
                write!(buffer, "export const meta = {};\n", meta).ok();
            }
            buffer.push_str("export default defineMessages({\n");
            for (key, value) in &self.messages {
                write!(buffer, "\"{}\": \"{}\",\n", key, value).ok();
            }
            buffer.push_str("});\n");

            buffer
        }

        fn to_translations(&self) -> String {
            let mut buffer = String::new();
            buffer.push_str("{\n");
            let mut is_first = true;
            for (key, value) in &self.messages {
                if is_first {
                    is_first = false;
                    buffer.push('\n');
                } else {
                    buffer.push_str(",\n");
                }
                write!(buffer, "\"{}\":\"{}\"", key, value).ok();
            }
            buffer.push('}');

            buffer
        }
    }

    fn base_messages_file() -> SourceMessagesBuilder {
        SourceMessagesBuilder::new()
            .with_meta(Some("{secret: false}".into()))
            .with_message("CUSTOM_STATUS", "This is a custom status")
            .with_message("ANOTHER_STATUS", "This one is a _separate_ message")
    }

    // #[test]
    // fn test_definitions_removed_message() {
    //     let mut database = new_database();
    //
    //     let source_file_name = "SomeModule.messages.js";
    //
    //     let original = base_messages_file();
    //     let original_result =
    //         database.process_definitions_file(source_file_name, &original.to_definitions());
    //     assert_eq!(
    //         original_result.unwrap().message_keys().len(),
    //         original.count()
    //     );
    //
    //     let file_with_removed = original.clone().remove_message("CUSTOM_STATUS");
    //     println!("{}", file_with_removed.to_definitions());
    //     let updated_result = database
    //         .process_definitions_file(source_file_name, &file_with_removed.to_definitions());
    //     println!("{:#?}", updated_result);
    //     assert_eq!(
    //         updated_result.unwrap().message_keys().len(),
    //         original.count() - 1,
    //     );
    // }
}
