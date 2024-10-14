//! Public API for interacting with an intl message database.
//!
//! All logic for operations on the database should be implemented as functions here. Wrappers
//! around these functions, like for Node or Python bindings, should only ever implement type
//! casting to and from the caller types and then call one of these functions. Any implementation
//! of multiple calls should become a new function here rather than in the wrapper, unless it is
//! language-specific to the host (like constructing a host object for object-oriented languages).
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::io::Write;

use intl_database_core::{
    get_key_symbol, key_symbol, DatabaseError, DatabaseResult, KeySymbol, Message, MessageValue,
    MessagesDatabase, RawMessageTranslation, SourceFile, DEFAULT_LOCALE,
};
use intl_database_exporter::{CompiledMessageFormat, ExportTranslations, IntlMessagePreCompiler};
use intl_database_service::IntlDatabaseService;
use intl_database_types_generator::IntlTypesGenerator;
use intl_validator::{validate_message, MessageDiagnostic};

use crate::threading::run_in_thread_pool;

fn get_key_symbol_or_error(value: &str) -> DatabaseResult<KeySymbol> {
    get_key_symbol(value).ok_or(DatabaseError::ValueNotInterned(value.to_string()))
}

pub fn process_definitions_file(
    database: &mut MessagesDatabase,
    file_path: &str,
    locale: Option<&str>,
) -> anyhow::Result<KeySymbol> {
    let content = std::fs::read_to_string(&file_path)?;
    process_definitions_file_content(database, file_path, &content, locale)
}

pub fn process_definitions_file_content(
    database: &mut MessagesDatabase,
    file_path: &str,
    content: &str,
    locale: Option<&str>,
) -> anyhow::Result<KeySymbol> {
    let source_file = crate::sources::process_definitions_file(
        database,
        &file_path,
        &content,
        locale.as_ref().map_or(DEFAULT_LOCALE, |locale| &locale),
    )?;
    Ok(source_file)
}

pub fn process_all_translation_files(
    database: &mut MessagesDatabase,
    locale_map: HashMap<String, String>,
) -> anyhow::Result<()> {
    run_in_thread_pool(
        locale_map.into_iter(),
        |(locale, file_path)| {
            let content = std::fs::read_to_string(&file_path)
                .expect(&format!("Failed to read translation file at {}", file_path));
            Ok((
                key_symbol(&locale),
                key_symbol(&file_path),
                crate::sources::extract_translations_from_file(key_symbol(&file_path), &content)
                    .map(|translations| translations.collect::<Vec<RawMessageTranslation>>()),
            ))
        },
        |(locale, file_path, translations)| {
            crate::sources::insert_translations(
                database,
                file_path,
                locale,
                translations?.into_iter(),
            )?;
            Ok(())
        },
    )?;
    Ok(())
}

pub fn process_translation_file(
    database: &mut MessagesDatabase,
    file_path: &str,
    locale: &str,
) -> anyhow::Result<KeySymbol> {
    let content = std::fs::read_to_string(&file_path)?;
    process_translation_file_content(database, file_path, &locale, &content)
}

pub fn process_translation_file_content(
    database: &mut MessagesDatabase,
    file_path: &str,
    locale: &str,
    content: &str,
) -> anyhow::Result<KeySymbol> {
    let source_file =
        crate::sources::process_translations_file(database, &file_path, &locale, &content)?;
    Ok(source_file)
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
    allow_nullability: Option<bool>,
) -> anyhow::Result<()> {
    let mut output_file = std::fs::File::create(&output_file_path)?;
    let source_file_key = get_key_symbol_or_error(source_file_path)?;
    let mut generator = IntlTypesGenerator::new(
        &database,
        source_file_key,
        &mut output_file,
        output_file_path.to_string(),
        allow_nullability.unwrap_or(false),
    );
    generator.run()?;
    let map_file_path = String::from(output_file_path) + ".map";
    let mut source_map_file = std::fs::File::create(map_file_path)?;
    let source_map = generator.into_sourcemap()?;
    source_map_file.write(source_map.as_bytes())?;
    Ok(())
}

pub fn precompile(
    database: &MessagesDatabase,
    file_path: &str,
    locale: &str,
    output_path: &str,
    format: Option<CompiledMessageFormat>,
) -> anyhow::Result<()> {
    let buffer = precompile_to_buffer(database, file_path, locale, format)?;
    std::fs::write(output_path, buffer)?;
    Ok(())
}

pub fn precompile_to_buffer(
    database: &MessagesDatabase,
    file_path: &str,
    locale: &str,
    format: Option<CompiledMessageFormat>,
) -> anyhow::Result<Vec<u8>> {
    let locale_key = get_key_symbol_or_error(&locale)?;
    let source_key = get_key_symbol_or_error(file_path)?;
    let keys_count = database
        .get_source_file(source_key)
        .map_or(0, |source| source.message_keys().len());
    let mut result: Vec<u8> = Vec::with_capacity(keys_count * 80);
    IntlMessagePreCompiler::new(
        &database,
        &mut result,
        source_key,
        locale_key,
        format.unwrap_or(CompiledMessageFormat::Json).into(),
    )
    .run()?;
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
pub fn is_message_translations_file(key: &str) -> bool {
    intl_message_utils::is_message_translations_file(key)
}