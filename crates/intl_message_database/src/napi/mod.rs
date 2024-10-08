//! Native addon bindings for using intl-message-extractor in Node, using
//! napi-rs and neon. This lets the library use the full power of native
//! compilation when running locally on a machine rather than in a browser,
//! including parallelism for processing multiple files at once.
//!
//! This is the preferred way of using the library wherever possible.
use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi::JsUnknown;
use napi_derive::napi;
use rustc_hash::FxHashMap;

use intl_database_core::{
    DatabaseError, DatabaseResult, DEFAULT_LOCALE, get_key_symbol, key_symbol, KeySymbol,
    MessagesDatabase, RawMessageTranslation,
};
use intl_database_exporter::{ExportTranslations, IntlMessagePreCompiler};
use intl_database_service::IntlDatabaseService;
use intl_database_types_generator::IntlTypesGenerator;
use intl_validator::validate_message;

use crate::napi::types::{IntlCompiledMessageFormat, IntlDiagnostic};
use crate::threading::run_in_thread_pool;

mod types;

fn get_key_symbol_or_error(value: String) -> DatabaseResult<KeySymbol> {
    get_key_symbol(&value).ok_or(DatabaseError::ValueNotInterned(value))
}

#[napi]
pub struct IntlMessagesDatabase {
    database: MessagesDatabase,
}

#[napi]
impl IntlMessagesDatabase {
    #[napi(constructor)]
    pub fn new() -> Self {
        IntlMessagesDatabase {
            database: MessagesDatabase::new(),
        }
    }

    #[napi]
    pub fn process_definitions_file(
        &mut self,
        file_path: String,
        locale: Option<String>,
    ) -> anyhow::Result<String> {
        let content = std::fs::read_to_string(&file_path)?;
        self.process_definitions_file_content(file_path, content, locale)
    }

    #[napi]
    pub fn process_definitions_file_content(
        &mut self,
        file_path: String,
        content: String,
        locale: Option<String>,
    ) -> anyhow::Result<String> {
        let source_file = crate::sources::process_definitions_file(
            &mut self.database,
            &file_path,
            &content,
            locale.as_ref().map_or(DEFAULT_LOCALE, |locale| &locale),
        )?;
        Ok(source_file.to_string())
    }

    #[napi]
    pub fn process_all_translation_files(
        &mut self,
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
                    crate::sources::extract_translations_from_file(
                        key_symbol(&file_path),
                        &content,
                    )
                    .map(|translations| translations.collect::<Vec<RawMessageTranslation>>()),
                ))
            },
            |(locale, file_path, translations)| {
                crate::sources::insert_translations(
                    &mut self.database,
                    file_path,
                    locale,
                    translations?.into_iter(),
                )?;
                Ok(())
            },
        )?;
        Ok(())
    }

    #[napi]
    pub fn process_translation_file(
        &mut self,
        file_path: String,
        locale: String,
    ) -> anyhow::Result<String> {
        let content = std::fs::read_to_string(&file_path)?;
        self.process_translation_file_content(file_path, locale, content)
    }

    #[napi]
    pub fn process_translation_file_content(
        &mut self,
        file_path: String,
        locale: String,
        content: String,
    ) -> anyhow::Result<String> {
        let source_file = crate::sources::process_translations_file(
            &mut self.database,
            &file_path,
            &locale,
            &content,
        )?;
        Ok(source_file.to_string())
    }

    #[napi]
    pub fn get_known_locales(&self) -> Vec<String> {
        let locales = &self.database.known_locales;

        Vec::from_iter(locales.iter().map(|locale| locale.to_string()))
    }

    #[napi(ts_return_type = "IntlSourceFile")]
    pub fn get_source_file(&self, env: Env, file_path: String) -> anyhow::Result<JsUnknown> {
        let file_symbol = get_key_symbol_or_error(file_path)?;
        let Some(source) = self.database.sources.get(&file_symbol) else {
            return Err(DatabaseError::SymbolNotFound(file_symbol).into());
        };

        Ok(env.to_js_value(source)?)
    }

    #[napi]
    pub fn get_all_source_file_paths(&self) -> anyhow::Result<Vec<String>> {
        Ok(Vec::from_iter(
            self.database.sources.keys().map(KeySymbol::to_string),
        ))
    }

    #[napi(ts_return_type = "Record<string, string>")]
    /// Return a map of all message keys contained in the given source file, where the key of the
    /// map is the hashed name and the value is the original.
    pub fn get_source_file_key_map(
        &self,
        env: Env,
        file_path: String,
    ) -> anyhow::Result<JsUnknown> {
        let file_symbol = get_key_symbol_or_error(file_path)?;
        let Some(source) = self.database.sources.get(&file_symbol) else {
            return Err(DatabaseError::SymbolNotFound(file_symbol).into());
        };

        let mut hashes = FxHashMap::default();
        hashes.reserve(source.message_keys().len());

        for key in source.message_keys() {
            if let Some(message) = self.database.messages.get(key) {
                hashes.insert(message.hashed_key(), message.key());
            }
        }

        Ok(env.to_js_value(&hashes)?)
    }

    #[napi(ts_return_type = "IntlMessage")]
    pub fn get_message(&self, env: Env, key: String) -> anyhow::Result<JsUnknown> {
        let definition = &self
            .database
            .get_message(&key)
            .ok_or_else(|| DatabaseError::ValueNotInterned(key))?;

        Ok(env.to_js_value(definition)?)
    }

    #[napi]
    pub fn generate_types(
        &self,
        source_file_path: String,
        output_file_path: String,
        allow_nullability: Option<bool>,
    ) -> anyhow::Result<()> {
        let mut output_file = std::fs::File::create(output_file_path)?;
        let source_file_key = get_key_symbol_or_error(source_file_path)?;
        IntlTypesGenerator::new(
            &self.database,
            source_file_key,
            &mut output_file,
            allow_nullability.unwrap_or(false),
        )
        .run()
    }

    #[napi]
    pub fn precompile(
        &self,
        file_path: String,
        locale: String,
        output_path: String,
        format: Option<IntlCompiledMessageFormat>,
    ) -> anyhow::Result<()> {
        let buffer = self.precompile_to_buffer(file_path, locale, format)?;
        std::fs::write(output_path, buffer)?;
        Ok(())
    }

    #[napi]
    pub fn precompile_to_buffer(
        &self,
        file_path: String,
        locale: String,
        format: Option<IntlCompiledMessageFormat>,
    ) -> anyhow::Result<Buffer> {
        let locale_key = get_key_symbol_or_error(locale)?;
        let source_key = get_key_symbol_or_error(file_path)?;
        let keys_count = self
            .database
            .get_source_file(source_key)
            .map_or(0, |source| source.message_keys().len());
        let mut result: Vec<u8> = Vec::with_capacity(keys_count * 80);
        IntlMessagePreCompiler::new(
            &self.database,
            &mut result,
            source_key,
            locale_key,
            format.unwrap_or(IntlCompiledMessageFormat::Json).into(),
        )
        .run()?;
        Ok(result.into())
    }

    #[napi]
    pub fn validate_messages(&self) -> anyhow::Result<Vec<IntlDiagnostic>> {
        let mut results = vec![];
        for message in self.database.messages.values() {
            let diagnostics = validate_message(&message);
            if diagnostics.is_empty() {
                continue;
            }

            for diagnostic in diagnostics.into_iter() {
                results.push(IntlDiagnostic::from(diagnostic))
            }
        }

        Ok(results)
    }

    #[napi]
    pub fn export_translations(
        &self,
        file_extension: Option<String>,
    ) -> anyhow::Result<Vec<String>> {
        let files = ExportTranslations::new(&self.database, file_extension).run()?;
        Ok(files)
    }

    #[napi(ts_return_type = "Record<string, IntlMessageValue | undefined>")]
    /// Return something specific
    pub fn get_source_file_message_values(
        &self,
        env: Env,
        file_path: String,
    ) -> anyhow::Result<JsUnknown> {
        let source_key = get_key_symbol_or_error(file_path)?;
        let key_value_pairs = self.database.get_source_file_message_values(source_key)?;
        let map = FxHashMap::from_iter(key_value_pairs);
        Ok(env.to_js_value(&map)?)
    }
}

#[napi]
pub fn hash_message_key(key: String) -> String {
    intl_message_utils::hash_message_key(&key)
}

#[napi]
pub fn is_message_definitions_file(key: String) -> bool {
    intl_message_utils::is_message_definitions_file(&key)
}

#[napi]
pub fn is_message_translations_file(key: String) -> bool {
    intl_message_utils::is_message_translations_file(&key)
}
