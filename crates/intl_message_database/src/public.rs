//! Public API for interacting with an intl message database.
//!
//! All logic for operations on the database should be implemented as functions here. Wrappers
//! around these functions, like for Node or Python bindings, should only ever implement type
//! casting to and from the caller types and then call one of these functions. Any implementation
//! of multiple calls should become a new function here rather than in the wrapper, unless it is
//! language-specific to the host (like constructing a host object for object-oriented languages).
use crate::sources::{get_locale_from_file_name, MessagesFileDescriptor, SourceFileInsertionData};
use crate::threading::run_in_thread_pool;
use intl_database_core::{
    get_key_symbol, key_symbol, DatabaseError, DatabaseInsertStrategy, DatabaseResult, KeySymbol,
    Message, MessageValue, MessagesDatabase, RawMessageDefinition, RawMessageTranslation,
    SourceFile, DEFAULT_LOCALE,
};
use intl_database_exporter::{ExportTranslations, IntlMessageBundler, IntlMessageBundlerOptions};
use intl_database_service::IntlDatabaseService;
use intl_database_types_generator::IntlTypesGenerator;
use intl_validator::{validate_message, MessageDiagnostic};
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

fn get_key_symbol_or_error(value: &str) -> DatabaseResult<KeySymbol> {
    get_key_symbol(value).ok_or(DatabaseError::ValueNotInterned(value.to_string()))
}

/// Scan the file system within the given `source_directories` for all messages files contained
/// within them. Each returned entry will have the file path and the locale it should represent,
/// defaulting to `default_definition_locale` for definitions.
///
/// For large repositories, this can be quite slow, as all folders are scanned, including
/// `node_modules` and others.
pub fn find_all_messages_files<A: AsRef<str>>(
    source_directories: impl Iterator<Item = A>,
    default_definition_locale: &str,
) -> Vec<MessagesFileDescriptor> {
    crate::sources::find_all_messages_files(source_directories, default_definition_locale).collect()
}

/// Given a list of sources files, filter out all files except for those that can be treated as
/// messages files, either definitions or translations. Each returned entry will have the file path
/// and the locale it should represent, defaulting to `default_definition_locale` for definitions.
pub fn filter_all_messages_files<A: AsRef<str>>(
    files: impl Iterator<Item = A>,
    default_definition_locale: &str,
) -> Vec<MessagesFileDescriptor> {
    let definition_locale_key = key_symbol(default_definition_locale);
    let mut result = vec![];
    for file in files {
        let file = file.as_ref();
        if is_compiled_messages_artifact(file)
            || (!is_message_definitions_file(file) && !is_message_translations_file(file))
        {
            continue;
        }
        let locale = get_locale_from_file_name(file, definition_locale_key);
        result.push(MessagesFileDescriptor {
            file_path: PathBuf::from(file),
            locale,
        });
    }
    result
}

/// Given a list of directories, scan their entire contents to find all messages files (both
/// definitions _and_ translations), then process their content into the database.
///
/// Returns a list of processing results containing the file key and information about whether it
/// was processed successfully.
pub fn process_all_messages_files(
    database: &mut MessagesDatabase,
    files: impl Iterator<Item = MessagesFileDescriptor> + ExactSizeIterator,
    strategy: DatabaseInsertStrategy,
) -> anyhow::Result<Vec<SourceFileInsertionData>> {
    let results = run_in_thread_pool(
        files,
        |descriptor| {
            let MessagesFileDescriptor { file_path, locale } = descriptor;
            let content = std::fs::read_to_string(&file_path).expect(&format!(
                "Failed to read messages file at {}",
                file_path.display()
            ));
            let file_path = key_symbol(&file_path.to_string_lossy());
            let data = SourceFileInsertionData::new(file_path, locale);

            let (definitions, translations) = if is_message_definitions_file(&file_path) {
                match crate::sources::extract_definitions_from_file(file_path, &content) {
                    Ok((meta, definitions)) => (
                        Some((meta, definitions.collect::<Vec<RawMessageDefinition>>())),
                        None,
                    ),
                    _ => (None, None),
                }
            } else {
                let translations = crate::sources::extract_translations_from_file(
                    key_symbol(&file_path),
                    &content,
                )
                .map(|translations| translations.collect::<Vec<RawMessageTranslation>>());
                (None, Some(translations))
            };
            (data, definitions, translations)
        },
        |(mut data, definitions, translations)| {
            if let Some((source_meta, definitions)) = definitions {
                return crate::sources::insert_definitions(
                    database,
                    data,
                    source_meta,
                    definitions.into_iter(),
                    strategy,
                );
            }
            if let Some(translations) = translations {
                return match translations {
                    Ok(translations) => crate::sources::insert_translations(
                        database,
                        data,
                        translations.into_iter(),
                        strategy,
                    ),
                    Err(error) => {
                        data.add_error(error);
                        data
                    }
                };
            };
            data.add_error(DatabaseError::NoExtractableValues(
                data.file_key.to_string(),
            ));
            data
        },
    )?;
    Ok(results.into())
}

pub fn process_definitions_file(
    database: &mut MessagesDatabase,
    file_path: &str,
    locale: Option<&str>,
    strategy: DatabaseInsertStrategy,
) -> anyhow::Result<SourceFileInsertionData> {
    let content = std::fs::read_to_string(&file_path)?;
    Ok(process_definitions_file_content(
        database, file_path, &content, locale, strategy,
    ))
}

pub fn process_definitions_file_content(
    database: &mut MessagesDatabase,
    file_path: &str,
    content: &str,
    locale: Option<&str>,
    strategy: DatabaseInsertStrategy,
) -> SourceFileInsertionData {
    crate::sources::process_definitions_file(
        database,
        &file_path,
        &content,
        locale.as_ref().map_or(DEFAULT_LOCALE, |locale| &locale),
        strategy,
    )
}

pub fn process_all_translation_files(
    database: &mut MessagesDatabase,
    locale_map: HashMap<String, String>,
    strategy: DatabaseInsertStrategy,
) -> anyhow::Result<Vec<SourceFileInsertionData>> {
    let results = run_in_thread_pool(
        locale_map.into_iter(),
        |(locale, file_path)| {
            let content = std::fs::read_to_string(&file_path)
                .expect(&format!("Failed to read translation file at {}", file_path));
            (
                key_symbol(&locale),
                key_symbol(&file_path),
                crate::sources::extract_translations_from_file(key_symbol(&file_path), &content)
                    .map(|translations| translations.collect::<Vec<RawMessageTranslation>>()),
            )
        },
        |(locale, file_path, translations)| {
            let mut data = SourceFileInsertionData::new(file_path, locale);
            match translations {
                Ok(translations) => crate::sources::insert_translations(
                    database,
                    data,
                    translations.into_iter(),
                    strategy,
                ),
                Err(error) => {
                    data.add_error(error);
                    data
                }
            }
        },
    )?;
    Ok(results.into())
}

pub fn process_translation_file(
    database: &mut MessagesDatabase,
    file_path: &str,
    locale: &str,
    strategy: DatabaseInsertStrategy,
) -> anyhow::Result<SourceFileInsertionData> {
    let content = std::fs::read_to_string(&file_path)?;
    Ok(process_translation_file_content(
        database, file_path, &locale, &content, strategy,
    ))
}

pub fn process_translation_file_content(
    database: &mut MessagesDatabase,
    file_path: &str,
    locale: &str,
    content: &str,
    strategy: DatabaseInsertStrategy,
) -> SourceFileInsertionData {
    crate::sources::process_translations_file(database, &file_path, &locale, &content, strategy)
}

pub fn get_known_locales(database: &MessagesDatabase) -> Vec<KeySymbol> {
    let locales = &database.known_locales;

    Vec::from_iter(locales.clone())
}

pub fn get_source_file<'a>(
    database: &'a MessagesDatabase,
    file_path: &str,
) -> anyhow::Result<&'a SourceFile> {
    let file_symbol = get_key_symbol_or_error(file_path)?;
    let Some(source) = database.sources.get(&file_symbol) else {
        return Err(DatabaseError::SymbolNotFound(file_symbol).into());
    };

    Ok(source)
}

pub fn get_all_source_file_paths(database: &MessagesDatabase) -> anyhow::Result<Vec<KeySymbol>> {
    Ok(Vec::from_iter(database.sources.keys().map(Clone::clone)))
}

/// Return a map of all message keys contained in the given source file, where the key of the
/// map is the hashed name and the value is the original.
pub fn get_source_file_key_map(
    database: &MessagesDatabase,
    file_path: &str,
) -> anyhow::Result<FxHashMap<String, KeySymbol>> {
    let file_symbol = get_key_symbol_or_error(file_path)?;
    let Some(source) = database.sources.get(&file_symbol) else {
        return Err(DatabaseError::SymbolNotFound(file_symbol).into());
    };

    let mut hashes = FxHashMap::default();
    hashes.reserve(source.message_keys().len());

    for key in source.message_keys() {
        if let Some(message) = database.messages.get(key) {
            hashes.insert(message.hashed_key().clone(), message.key());
        }
    }

    Ok(hashes)
}

/// Return file paths for all definitions files with a translations path meta value that would
/// include the given `translationPath`. This can be used to add reverse dependencies, and
/// safely cache translations files with appropriate change detection.
pub fn get_definitions_files_for_translations_path(
    database: &MessagesDatabase,
    translations_path: &str,
) -> Vec<KeySymbol> {
    let expected_path = PathBuf::from(translations_path);
    // Very cheap to allocate initially, and saves repeated allocations that
    // are very likely to be hit.
    let mut result = Vec::with_capacity(8);
    for (path, file) in &database.sources {
        match file {
            SourceFile::Translation(_) => continue,
            SourceFile::Definition(definition) => {
                if definition.meta().translations_path == expected_path {
                    result.push(*path)
                }
            }
        }
    }
    result
}

pub fn get_message<'a>(database: &'a MessagesDatabase, key: &str) -> anyhow::Result<&'a Message> {
    let definition = database
        .get_message(&key)
        .ok_or_else(|| DatabaseError::ValueNotInterned(key.to_string()))?;

    Ok(definition)
}

pub fn generate_types(
    database: &MessagesDatabase,
    source_file_path: &str,
    output_file_path: &str,
) -> anyhow::Result<()> {
    let source_file_key = get_key_symbol_or_error(source_file_path)?;
    let mut generator =
        IntlTypesGenerator::new(&database, source_file_key, output_file_path.to_string());
    generator.run()?;
    std::fs::write(&output_file_path, generator.take_buffer())?;
    let map_file_path = String::from(output_file_path) + ".map";
    let mut source_map_file = std::fs::File::create(map_file_path)?;
    let source_map = generator.into_sourcemap()?;
    source_map_file.write_all(source_map.as_bytes())?;
    Ok(())
}

pub fn precompile(
    database: &MessagesDatabase,
    file_path: &str,
    locale: &str,
    output_path: &str,
    options: IntlMessageBundlerOptions,
) -> anyhow::Result<()> {
    let buffer = precompile_to_buffer(database, file_path, locale, options)?;
    std::fs::write(output_path, buffer)?;
    Ok(())
}

pub fn precompile_to_buffer(
    database: &MessagesDatabase,
    file_path: &str,
    locale: &str,
    options: IntlMessageBundlerOptions,
) -> anyhow::Result<Vec<u8>> {
    let locale_key = get_key_symbol_or_error(&locale)?;
    let source_key = get_key_symbol_or_error(file_path)?;
    let keys_count = database
        .get_source_file(source_key)
        .map_or(0, |source| source.message_keys().len());
    let mut result: Vec<u8> = Vec::with_capacity(keys_count * 80);
    IntlMessageBundler::new(&database, &mut result, source_key, locale_key, options).run()?;
    Ok(result.into())
}

pub fn validate_messages(database: &MessagesDatabase) -> anyhow::Result<Vec<MessageDiagnostic>> {
    let mut results = vec![];
    for message in database.messages.values() {
        let diagnostics = validate_message(&message);
        if diagnostics.is_empty() {
            continue;
        }

        results.extend(diagnostics);
    }

    Ok(results)
}

pub fn export_translations(
    database: &MessagesDatabase,
    file_extension: Option<String>,
) -> anyhow::Result<Vec<String>> {
    let files = ExportTranslations::new(&database, file_extension).run()?;
    Ok(files)
}

pub fn get_source_file_message_values<'a>(
    database: &'a MessagesDatabase,
    file_path: &str,
) -> anyhow::Result<FxHashMap<&'a KeySymbol, Option<&'a MessageValue>>> {
    let source_key = get_key_symbol_or_error(file_path)?;
    let key_value_pairs = database.get_source_file_message_values(source_key)?;
    Ok(FxHashMap::from_iter(key_value_pairs))
}

#[inline(always)]
pub fn hash_message_key(key: &str) -> String {
    intl_message_utils::hash_message_key(key)
}

#[inline(always)]
pub fn is_message_definitions_file(key: &str) -> bool {
    intl_message_utils::is_message_definitions_file(key)
}

#[inline(always)]
pub fn is_compiled_messages_artifact(key: &str) -> bool {
    intl_message_utils::is_compiled_messages_artifact(key)
}

#[inline(always)]
pub fn is_message_translations_file(key: &str) -> bool {
    intl_message_utils::is_message_translations_file(key)
}
