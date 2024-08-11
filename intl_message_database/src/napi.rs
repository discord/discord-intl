//! Native addon bindings for using intl-message-extractor in Node, using
//! napi-rs and neon. This lets the library use the full power of native
//! compilation when running locally on a machine rather than in a browser,
//! including parallelism for processing multiple files at once.
//!
//! This is the preferred way of using the library wherever possible.
use std::collections::HashMap;

use napi::{JsNumber, JsUnknown};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::messages::{
    global_get_symbol, global_intern_string, KeySymbol, MessagesDatabase, MessagesError,
    read_global_symbol_store,
};
use crate::services::IntlService;
use crate::services::precompile::{CompiledMessageFormat, IntlMessagePreCompiler};
use crate::services::types::IntlTypesGenerator;
use crate::services::validator;
use crate::sources::extract_message_translations;
use crate::threading::run_in_thread_pool;

#[napi]
pub struct IntlMessagesDatabase {
    database: MessagesDatabase,
}

#[napi(object)]
pub struct IntlDiagnostic {
    pub key: String,
    pub diagnostics: JsUnknown,
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlSourceFile {
    #[napi(js_name = "type")]
    pub ty: String,
    pub file: String,
    #[napi(js_name = "messageKeys")]
    pub message_keys: Vec<JsNumber>,
    pub meta: IntlMessageMeta,
    pub locale: Option<JsNumber>,
}

#[napi(object)]
pub struct IntlMessageMeta {
    pub secret: bool,
    pub translate: bool,
    #[napi(js_name = "bundleSecrets")]
    pub bundle_secrets: bool,
    #[napi(js_name = "translationsPath")]
    pub translations_path: String,
}

#[napi]
pub enum IntlCompiledMessageFormat {
    Json,
    KeylessJson,
}

impl From<IntlCompiledMessageFormat> for CompiledMessageFormat {
    fn from(value: IntlCompiledMessageFormat) -> Self {
        match value {
            IntlCompiledMessageFormat::Json => CompiledMessageFormat::Json,
            IntlCompiledMessageFormat::KeylessJson => CompiledMessageFormat::KeylessJson,
        }
    }
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
    pub fn process_definitions_file(&mut self, file_path: String) -> anyhow::Result<u32> {
        let content = std::fs::read_to_string(&file_path)?;

        let source_file =
            crate::sources::process_definitions_file(&mut self.database, &file_path, &content)?;
        Ok(source_file.value() as u32)
    }

    #[napi]
    pub fn process_definitions_file_content(
        &mut self,
        file_path: String,
        content: String,
    ) -> anyhow::Result<u32> {
        let source_file =
            crate::sources::process_definitions_file(&mut self.database, &file_path, &content)?;
        Ok(source_file.value() as u32)
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
                Ok((locale, file_path, extract_message_translations(&content)))
            },
            |(locale, file_path, translations)| {
                let translations = translations?;
                self.database
                    .insert_translations_from_file(&file_path, &locale, translations)?;
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
    ) -> anyhow::Result<u32> {
        let content = std::fs::read_to_string(&file_path)?;
        self.process_translation_file_content(file_path, locale, content)
    }

    #[napi]
    pub fn process_translation_file_content(
        &mut self,
        file_path: String,
        locale: String,
        content: String,
    ) -> anyhow::Result<u32> {
        let translations = extract_message_translations(&content)?;
        self.database
            .insert_translations_from_file(&file_path, &locale, translations)?;
        let file_key = global_intern_string(&file_path)?;
        Ok(file_key.value() as u32)
    }

    #[napi]
    pub fn get_known_locales(&self) -> anyhow::Result<Vec<String>> {
        let locales = &self.database.known_locales;

        let mut result = vec![];

        for locale in locales.iter() {
            // Safety: We _shouldn't_ be able to have a locale symbol without that
            // locale being interned already. If we do, something must be very
            // corrupt, so we can just panic.
            let store = read_global_symbol_store()?;

            let locale = store
                .resolve(*locale)
                .ok_or_else(|| MessagesError::SymbolNotFound(*locale))?;

            result.push(locale.into());
        }

        Ok(result)
    }

    #[napi(ts_return_type = "IntlSourceFile")]
    pub fn get_source_file(&self, env: Env, file_path: String) -> anyhow::Result<JsUnknown> {
        let file_symbol = global_get_symbol(&file_path)?;
        let Some(source) = self.database.sources.get(&file_symbol) else {
            return Err(MessagesError::SymbolNotFound(file_symbol).into());
        };

        Ok(env.to_js_value(source)?)
    }

    #[napi]
    pub fn get_all_source_file_paths(&self) -> anyhow::Result<Vec<String>> {
        let sources = self.database.sources.keys();
        let mut paths = Vec::with_capacity(sources.len());
        for key in sources {
            if let Some(path) = resolve_symbol(key.value() as u32)? {
                paths.push(path)
            }
        }

        Ok(paths)
    }

    #[napi(ts_return_type = "string[]")]
    pub fn get_source_file_hashed_keys(
        &self,
        env: Env,
        file_path: String,
    ) -> anyhow::Result<JsUnknown> {
        let file_symbol = global_get_symbol(&file_path)?;
        let Some(source) = self.database.sources.get(&file_symbol) else {
            return Err(MessagesError::SymbolNotFound(file_symbol).into());
        };

        let mut hashes = Vec::with_capacity(source.message_keys().len());

        for key in source.message_keys() {
            if let Some(message) = self.database.messages.get(key) {
                hashes.push(message.hashed_key());
            }
        }

        Ok(env.to_js_value(&hashes)?)
    }

    #[napi]
    pub fn get_message(&self, env: Env, key: String) -> anyhow::Result<JsUnknown> {
        let definition = &self
            .database
            .get_message(&key)
            .ok_or_else(|| MessagesError::ValueNotInterned(key))?;

        Ok(env.to_js_value(definition)?)
    }

    #[napi]
    pub fn generate_types(
        &self,
        source_file_path: String,
        output_file_path: String,
    ) -> anyhow::Result<()> {
        let mut output_file = std::fs::File::create(output_file_path)?;
        let source_file_key = global_get_symbol(&source_file_path)?;
        IntlTypesGenerator::new(&self.database, source_file_key, &mut output_file).run()
    }

    #[napi]
    pub fn precompile(
        &self,
        source_path: String,
        locale: String,
        output_path: String,
        format: Option<IntlCompiledMessageFormat>,
    ) -> anyhow::Result<()> {
        let buffer = self.precompile_to_buffer(source_path, locale, format)?;
        std::fs::write(output_path, buffer)?;
        Ok(())
    }

    #[napi]
    pub fn precompile_to_buffer(
        &self,
        source_path: String,
        locale: String,
        format: Option<IntlCompiledMessageFormat>,
    ) -> anyhow::Result<Buffer> {
        let locale_key = global_get_symbol(&locale)?;
        let source_key = global_get_symbol(&source_path)?;
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
    pub fn validate_messages(&self, env: Env) -> anyhow::Result<Vec<IntlDiagnostic>> {
        let key_store = read_global_symbol_store()?;

        let mut results = vec![];
        for (key, message) in self.database.messages.iter() {
            let diagnostics = validator::validate_message(&message);
            if diagnostics.is_empty() {
                continue;
            }

            let key = key_store
                .resolve(*key)
                .ok_or_else(|| MessagesError::SymbolNotFound(*key))?;

            results.push(IntlDiagnostic {
                key: key.into(),
                diagnostics: env.to_js_value(&diagnostics)?,
            });
        }

        Ok(results)
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

#[napi]
pub fn resolve_symbol(symbol: u32) -> anyhow::Result<Option<String>> {
    let store = read_global_symbol_store()?;

    Ok(KeySymbol::from_usize(symbol as usize)
        .and_then(|key| store.resolve(key))
        .map(String::from))
}
