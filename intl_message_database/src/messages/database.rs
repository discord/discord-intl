use rustc_hash::{FxHashMap, FxHashSet};

use crate::messages::symbols::KeySymbolMap;
use crate::sources::Translations;

use super::{
    FilePosition, global_get_symbol, global_intern_string, KeySymbol, LocaleId,
    Message, MessageMeta, MessagesError, MessagesResult, MessageValue, read_global_symbol_store, SourceFile,
};

#[derive(Debug)]
pub struct MessagesDatabase {
    pub messages: KeySymbolMap<Message>,
    pub sources: KeySymbolMap<SourceFile>,
    pub hash_lookup: FxHashMap<String, KeySymbol>,
    pub known_locales: FxHashSet<KeySymbol>,
}

impl MessagesDatabase {
    pub fn new() -> Self {
        Self {
            messages: KeySymbolMap::default(),
            sources: KeySymbolMap::default(),
            hash_lookup: FxHashMap::default(),
            known_locales: FxHashSet::default(),
        }
    }

    /// Return the complete message definition under a given key.
    pub fn get_message(&self, key: &str) -> Option<&Message> {
        global_get_symbol(key)
            .ok()
            .and_then(|symbol| self.messages.get(&symbol))
    }

    /// Returns owned copies of all message names that are currently in the database.
    pub fn get_message_keys(&self) -> MessagesResult<Vec<String>> {
        let store = read_global_symbol_store()?;

        let mut result = vec![];
        for key in self.messages.keys() {
            match store.resolve(*key) {
                Some(value) => result.push(value.into()),
                _ => {}
            }
        }
        Ok(result)
    }

    /// Return the string value of a Symbol that has been interned.
    /// If the value is not known.
    pub fn get_symbol(&self, value: &str) -> MessagesResult<KeySymbol> {
        global_get_symbol(value)
    }

    pub fn get_source_file(&self, file_key: KeySymbol) -> Option<&SourceFile> {
        self.sources.get(&file_key)
    }

    pub fn create_source_file(
        &mut self,
        file_key: KeySymbol,
        file_name: &str,
        meta: MessageMeta,
    ) -> &SourceFile {
        self.sources.insert(
            file_key,
            SourceFile::Definition {
                file: file_name.into(),
                message_keys: FxHashSet::default(),
                meta,
            },
        );
        &self.sources[&file_key]
    }

    pub fn set_source_file_keys(
        &mut self,
        file_key: KeySymbol,
        keys: FxHashSet<KeySymbol>,
    ) -> MessagesResult<()> {
        self.sources
            .get_mut(&file_key)
            .map(|source| source.set_message_keys(keys))
            .ok_or(MessagesError::UnknownSourceFile(file_key))
    }

    /// Insert a new message definition into the database. If a Normal entry with the same key
    /// already exists and `replace_existing` is `false`, this method will return an Error that the
    /// message is already defined and cannot be replaced. However, if `replace_existing` is `true`
    /// and the existing definition comes from the same source file, _or_ if the existing entry is
    /// Undefined, this method will update and convert that entry to a Normal entry and return Ok.
    ///
    /// An Ok response from this method will _always_ return a `MessageDefinition::Normal` value.
    pub fn insert_definition(
        &mut self,
        name: &str,
        value: MessageValue,
        locale: KeySymbol,
        meta: MessageMeta,
        replace_existing: bool,
    ) -> MessagesResult<&Message> {
        let key = global_intern_string(name)?;
        match self.messages.get_mut(&key) {
            Some(existing) => {
                // Complete messages that already exist can not be re-added, since
                // that would mean two definitions exist. Instead, they can be
                // _updated_, for example when a definition file changes.
                if existing.is_defined() && !replace_existing {
                    return Err(MessagesError::AlreadyDefined(key));
                }

                existing.set_definition(value, locale, meta);
            }
            _ => {
                // Otherwise this is an entirely new message that gets created.
                let message = Message::from_definition(key, name.into(), value, locale, meta);
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

    /// Insert a new message definition into the database. If a Normal
    /// entry with the same key already exists, this method will return an
    /// Error. However, if the existing entry is Undefined, this method will
    /// convert that entry to a Normal entry and return Ok.
    ///
    /// An Ok response from this method will _always_ return a
    /// `MessageDefinition::Normal` value.
    fn insert_translation(
        &mut self,
        key: KeySymbol,
        locale: KeySymbol,
        value: MessageValue,
        replace_existing: bool,
    ) -> MessagesResult<&Message> {
        match self.messages.get_mut(&key) {
            // If the key has an existing message at all, it just gets a new
            // translation entry in the map. The type of the entry does not
            // change here.
            Some(message) => {
                if message.translations().contains_key(&locale) && !replace_existing {
                    let store = read_global_symbol_store()?;
                    let name = store
                        .resolve(key)
                        .ok_or_else(|| MessagesError::SymbolNotFound(key))?;
                    return Err(MessagesError::TranslationAlreadySet(name.into(), locale));
                }

                self.known_locales.insert(locale);
                message.set_translation(locale, value);
            }
            // If it doesn't already exist, add a new Undefined message to hold
            // the translation until a definition is found.
            _ => {
                // Otherwise this is an entirely new message that gets created.
                let store = read_global_symbol_store()?;
                let name = store
                    .resolve(key)
                    .ok_or_else(|| MessagesError::SymbolNotFound(key))?;
                let message = Message::from_translation(key, name.into(), locale, value);
                self.known_locales.insert(locale);
                self.hash_lookup.insert(message.hashed_key().clone(), key);
                self.messages.insert(key, message);
            }
        }

        Ok(&self.messages[&key])
    }

    fn remove_translation(
        &mut self,
        message_key: KeySymbol,
        locale: KeySymbol,
    ) -> Option<MessageValue> {
        self.messages
            .get_mut(&message_key)
            .and_then(|message| message.remove_translation(locale))
    }

    /// Similar to [MessagesDatabase::process_translations_file], but where the
    /// caller has already done the work to read the file and parse it into a
    /// set of translations. Useful for multithreading on native platforms.
    pub fn insert_translations_from_file(
        &mut self,
        file_name: &str,
        locale: &LocaleId,
        translations: Translations,
    ) -> MessagesResult<&SourceFile> {
        let file_key = global_intern_string(file_name)?;
        let locale_symbol = global_intern_string(&locale)?;

        let mut inserted_translations = FxHashSet::default();

        for entry in translations.entries {
            let value = entry.value.with_file_position(FilePosition {
                file: file_key,
                offset: entry.start_offset as u32,
            });
            if let Ok(inserted) = self.insert_translation(entry.key, locale_symbol, value, false) {
                inserted_translations.insert(inserted.key_symbol());
            }
        }

        let source_file = SourceFile::Translation {
            file: file_name.into(),
            message_keys: inserted_translations,
            locale: locale.clone(),
        };

        self.sources.insert(file_key, source_file);

        Ok(&self.sources[&file_key])
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use intl_message_utils::RUNTIME_PACKAGE_NAME;

    use crate::messages::MessagesDatabase;

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
