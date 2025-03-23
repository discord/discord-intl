use ignore::WalkBuilder;
use intl_database_core::{
    key_symbol, DatabaseError, DatabaseInsertStrategy, DatabaseResult, DefinitionFile, KeySymbol,
    KeySymbolSet, MessageDefinitionSource, MessageTranslationSource, MessagesDatabase,
    RawMessageDefinition, RawMessageTranslation, SourceFile, SourceFileMeta, TranslationFile,
};
use intl_database_js_source::JsMessageSource;
use intl_database_json_source::JsonMessageSource;
use intl_message_utils::{is_any_messages_file, is_message_translations_file};
use rustc_hash::FxHashSet;
use serde::Serialize;
use std::path::PathBuf;

struct SourceFileKeyTracker {
    inserted_keys: KeySymbolSet,
    /// Keys that used to exist in the file but are not found on this iteration are _removed_,
    /// meaning they will have their entry in the database taken out, and won't be considered
    /// part of the source file anymore.
    /// To do that, this takes the existing list of keys and removes each found entry from it,
    /// then uses those keys to delete values from the database.
    removed_keys: KeySymbolSet,
}

impl SourceFileKeyTracker {
    fn new(existing_keys: KeySymbolSet) -> Self {
        Self {
            inserted_keys: KeySymbolSet::default(),
            removed_keys: existing_keys,
        }
    }

    fn track(&mut self, key: KeySymbol) {
        self.inserted_keys.insert(key);
        self.removed_keys.remove(&key);
    }

    fn contains(&self, key: &KeySymbol) -> bool {
        self.inserted_keys.contains(key)
    }
}

#[derive(Default, Debug)]
pub struct SourceFileInsertionData {
    pub file_key: KeySymbol,
    pub locale_key: KeySymbol,
    pub errors: Vec<DatabaseError>,
    pub inserted_keys: KeySymbolSet,
    pub removed_keys: KeySymbolSet,
}

impl SourceFileInsertionData {
    pub(crate) fn new(file_key: KeySymbol, locale_key: KeySymbol) -> Self {
        Self {
            file_key,
            locale_key,
            errors: vec![],
            inserted_keys: KeySymbolSet::default(),
            removed_keys: KeySymbolSet::default(),
        }
    }

    pub(crate) fn add_error(&mut self, error: DatabaseError) {
        self.errors.push(error);
    }

    fn take_from_key_tracker(&mut self, tracker: SourceFileKeyTracker) {
        self.inserted_keys = tracker.inserted_keys;
        self.removed_keys = tracker.removed_keys;
    }
}

pub type SourceFileInsertionResult = DatabaseResult<SourceFileInsertionData>;

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

pub(crate) fn get_locale_from_file_name(
    file_name: &str,
    default_definition_locale: KeySymbol,
) -> KeySymbol {
    if is_message_translations_file(file_name) {
        get_translation_source_from_file_name(file_name)
            .map_or(default_definition_locale, |source| {
                source.get_locale_from_file_name(file_name)
            })
    } else {
        get_definition_source_from_file_name(file_name)
            .map_or(default_definition_locale, |source| {
                source.get_default_locale(file_name)
            })
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagesFileDescriptor {
    pub file_path: PathBuf,
    pub locale: KeySymbol,
}

/// Discover all files that are presumed to contain message definitions or translations by scanning
/// the file system through the given `directories`. Each returned entry will have both the path
/// for the file and the locale that it should represent. For definitions files,
/// `default_definition_locale` will be used unless the source is able to provide more information
/// about what locale it represents.
pub fn find_all_messages_files<A: AsRef<str>>(
    mut directories: impl Iterator<Item = A>,
    default_definition_locale: &str,
) -> impl Iterator<Item = MessagesFileDescriptor> {
    let first_directory = directories
        .next()
        .expect("find_all_messages_files requires at least one directory to scan");
    let default_definition_locale = key_symbol(default_definition_locale);
    let mut builder = WalkBuilder::new(first_directory.as_ref());
    for directory in directories {
        builder.add(directory.as_ref());
    }
    let walker = builder.build();
    let mut found_files = FxHashSet::default();
    walker.into_iter().filter_map(move |item| {
        let Ok(item) = item else {
            return None;
        };
        let file_path = item.path().to_path_buf();
        if found_files.contains(&file_path) {
            return None;
        }
        found_files.insert(file_path.clone());

        let Some(basename) = file_path.file_name() else {
            return None;
        };
        let basename = &basename.to_string_lossy();
        if item.file_type().is_some_and(|file_type| file_type.is_dir())
            || !is_any_messages_file(basename)
        {
            return None;
        }
        let locale = get_locale_from_file_name(&basename, default_definition_locale);
        Some(MessagesFileDescriptor { file_path, locale })
    })
}

pub fn process_definitions_file(
    db: &mut MessagesDatabase,
    file_name: &str,
    content: &str,
    locale: &str,
) -> SourceFileInsertionData {
    let file_key = key_symbol(file_name);
    let locale_key = key_symbol(locale);
    let mut data = SourceFileInsertionData::new(file_key, locale_key);
    match extract_definitions_from_file(file_key, content) {
        Ok((file_meta, definitions)) => insert_definitions(db, data, file_meta, definitions),
        Err(error) => {
            data.add_error(error);
            data
        }
    }
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
    mut data: SourceFileInsertionData,
    source_file_meta: SourceFileMeta,
    definitions: impl Iterator<Item = RawMessageDefinition>,
) -> SourceFileInsertionData {
    let source_file = db.get_or_create_source_file(
        data.file_key,
        SourceFile::Definition(DefinitionFile::new(
            data.file_key.to_string(),
            source_file_meta,
            KeySymbolSet::default(),
        )),
    );
    let mut tracker = SourceFileKeyTracker::new(source_file.message_keys().clone());
    for definition in definitions {
        if tracker.contains(&definition.name) {
            let existing = db
                .get_message(&definition.name)
                .map(|message| message.definition())
                .expect("Source File repeats message but previous value did not exist");
            data.add_error(DatabaseError::AlreadyDefined {
                name: definition.name,
                existing: existing.clone(),
                replacement: definition.value,
            });
            continue;
        }
        let insertion_result = db.insert_definition(
            &definition.name,
            definition.value,
            data.locale_key,
            definition.meta,
            DatabaseInsertStrategy::UpdateSourceFile,
        );
        if let Err(error) = insertion_result {
            data.add_error(error);
        }

        tracker.track(definition.name);
    }

    data.take_from_key_tracker(tracker);
    match db.set_source_file_keys(data.file_key, data.inserted_keys.clone()) {
        Err(error) => data.add_error(error),
        Ok(_) => {}
    }
    for key in &data.removed_keys {
        db.remove_definition(*key);
    }

    data
}

pub fn process_translations_file(
    db: &mut MessagesDatabase,
    file_name: &str,
    locale: &str,
    content: &str,
) -> SourceFileInsertionData {
    let file_key = key_symbol(file_name);
    let locale_key = key_symbol(&locale);
    let mut data = SourceFileInsertionData::new(file_key, locale_key);
    match extract_translations_from_file(file_key, content) {
        Ok(translations) => insert_translations(db, data, translations),
        Err(error) => {
            data.add_error(error);
            data
        }
    }
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
    mut data: SourceFileInsertionData,
    translations: impl Iterator<Item = RawMessageTranslation>,
) -> SourceFileInsertionData {
    let source_file = db.get_or_create_source_file(
        data.file_key,
        SourceFile::Translation(TranslationFile::new(
            data.file_key.to_string(),
            data.locale_key,
            KeySymbolSet::default(),
        )),
    );

    let mut tracker = SourceFileKeyTracker::new(source_file.message_keys().clone());
    for translation in translations {
        if tracker.contains(&translation.name) {
            let existing = db
                .get_message(&translation.name)
                .and_then(|message| message.translations().get(&data.locale_key))
                .expect("Source File repeats message but previous value did not exist");
            data.add_error(DatabaseError::TranslationAlreadySet {
                name: translation.name,
                locale: data.locale_key,
                existing: existing.clone(),
                replacement: translation.value,
            });
            continue;
        }
        tracker.track(translation.name);
        let insertion_result = db.insert_translation(
            translation.name,
            data.locale_key,
            translation.value,
            DatabaseInsertStrategy::UpdateSourceFile,
        );
        if let Err(error) = insertion_result {
            data.add_error(error);
        }
    }

    data.take_from_key_tracker(tracker);
    match db.set_source_file_keys(data.file_key, data.inserted_keys.clone()) {
        Err(error) => data.add_error(error),
        Ok(_) => {}
    }
    for key in &data.removed_keys {
        db.remove_translation(*key, data.locale_key);
    }

    data
}
