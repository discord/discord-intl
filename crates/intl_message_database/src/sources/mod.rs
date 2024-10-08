use std::iter::FusedIterator;

use intl_database_core::{
    DatabaseError, DatabaseResult, DefinitionFile, FilePosition, key_symbol, KeySymbol,
    KeySymbolSet, MessageDefinitionSource, MessagesDatabase, MessageTranslationSource, RawMessage,
    RawMessageDefinition, RawMessageTranslation, SourceFile, SourceFileMeta, TranslationFile,
};
use intl_database_js_source::JsMessageSource;
use intl_database_json_source::JsonMessageSource;

struct SourceFileKeyTrackingIterator<T: RawMessage, I: Iterator<Item = T>> {
    iterator: I,
    inserted_keys: KeySymbolSet,
    /// Keys that used to exist in the file but are not found on this iteration are _removed_,
    /// meaning they will have their entry in the database taken out, and won't be considered
    /// part of the source file anymore.
    /// To do that, this takes the existing list of keys and removes each found entry from it,
    /// then uses those keys to delete values from the database.
    removed_keys: KeySymbolSet,
}

impl<T: RawMessage, I: Iterator<Item = T>> SourceFileKeyTrackingIterator<T, I> {
    fn new(existing_keys: KeySymbolSet, iterator: I) -> Self {
        Self {
            iterator,
            inserted_keys: KeySymbolSet::default(),
            removed_keys: existing_keys,
        }
    }
}

impl<T: RawMessage, I: Iterator<Item = T>> Iterator for &mut SourceFileKeyTrackingIterator<T, I> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(message) = self.iterator.next() else {
            return None;
        };

        let key = message.name();
        self.removed_keys.remove(&key);
        self.inserted_keys.insert(key);
        Some(message)
    }
}

impl<T: RawMessage, I: Iterator<Item = T>> FusedIterator
    for &mut SourceFileKeyTrackingIterator<T, I>
{
}

fn get_definition_source_from_file_name(file_name: &str) -> Option<impl MessageDefinitionSource> {
    if file_name.ends_with(".js") {
        Some(JsMessageSource)
    } else {
        None
    }
}

fn get_translation_source_from_file_name(file_name: &str) -> Option<impl MessageTranslationSource> {
    if file_name.ends_with(".json") || file_name.ends_with(".jsona") {
        Some(JsonMessageSource)
    } else {
        None
    }
}

pub fn process_definitions_file(
    db: &mut MessagesDatabase,
    file_name: &str,
    content: &str,
    locale: &str,
) -> DatabaseResult<KeySymbol> {
    let file_key = key_symbol(file_name);
    let locale_key = key_symbol(locale);
    let (file_meta, definitions) = extract_definitions_from_file(file_key, content)?;
    insert_definitions(db, file_key, locale_key, file_meta, definitions)
}

pub fn extract_definitions_from_file(
    file_key: KeySymbol,
    content: &str,
) -> DatabaseResult<(
    SourceFileMeta,
    impl Iterator<Item = RawMessageDefinition> + '_,
)> {
    let source = get_definition_source_from_file_name(&file_key)
        .ok_or(DatabaseError::NoSourceImplementation(file_key.to_string()))?;

    source
        .extract_definitions(file_key, content)
        .map_err(DatabaseError::SourceError)
}

pub fn insert_definitions(
    db: &mut MessagesDatabase,
    file_key: KeySymbol,
    locale_key: KeySymbol,
    source_file_meta: SourceFileMeta,
    definitions: impl Iterator<Item = RawMessageDefinition>,
) -> DatabaseResult<KeySymbol> {
    let source_file = db.get_or_create_source_file(
        file_key,
        SourceFile::Definition(DefinitionFile::new(
            file_key.to_string(),
            source_file_meta,
            KeySymbolSet::default(),
        )),
    );
    let mut iterator =
        SourceFileKeyTrackingIterator::new(source_file.message_keys().clone(), definitions);
    for definition in &mut iterator {
        let position = FilePosition {
            file: file_key,
            offset: definition.offset,
        };
        let value = definition.value.with_file_position(position);
        db.insert_definition(&definition.name, value, locale_key, definition.meta, true)?;
    }

    db.set_source_file_keys(file_key, iterator.inserted_keys)?;
    for key in iterator.removed_keys {
        db.remove_definition(key);
    }

    Ok(file_key)
}

pub fn process_translations_file(
    db: &mut MessagesDatabase,
    file_name: &str,
    locale: &str,
    content: &str,
) -> DatabaseResult<KeySymbol> {
    let file_key = key_symbol(file_name);
    let locale_key = key_symbol(&locale);
    let translations = extract_translations_from_file(file_key, content)?;
    insert_translations(db, file_key, locale_key, translations)
}

pub fn extract_translations_from_file(
    file_key: KeySymbol,
    content: &str,
) -> DatabaseResult<impl Iterator<Item = RawMessageTranslation> + '_> {
    let source = get_translation_source_from_file_name(&file_key)
        .ok_or(DatabaseError::NoSourceImplementation(file_key.to_string()))?;
    source
        .extract_translations(file_key, content)
        .map_err(DatabaseError::SourceError)
}

pub fn insert_translations(
    db: &mut MessagesDatabase,
    file_key: KeySymbol,
    locale_key: KeySymbol,
    translations: impl Iterator<Item = RawMessageTranslation>,
) -> DatabaseResult<KeySymbol> {
    let source_file = db.get_or_create_source_file(
        file_key,
        SourceFile::Translation(TranslationFile::new(
            file_key.to_string(),
            locale_key,
            KeySymbolSet::default(),
        )),
    );

    let mut iterator =
        SourceFileKeyTrackingIterator::new(source_file.message_keys().clone(), translations);
    for translation in &mut iterator {
        let position = FilePosition {
            file: file_key,
            offset: translation.offset,
        };
        let value = translation.value.with_file_position(position);
        db.insert_translation(translation.name, locale_key, value, true)?;
    }

    for key in iterator.removed_keys {
        db.remove_translation(key, locale_key);
    }

    db.set_source_file_keys(file_key, iterator.inserted_keys)?;
    Ok(file_key)
}
