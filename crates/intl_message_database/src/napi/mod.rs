//! Native addon bindings for using intl-message-extractor in Node, using
//! napi-rs and neon. This lets the library use the full power of native
//! compilation when running locally on a machine rather than in a browser,
//! including parallelism for processing multiple files at once.
//!
//! This is the preferred way of using the library wherever possible.
use napi::bindgen_prelude::*;
use napi::JsUnknown;
use napi_derive::napi;
use std::collections::HashMap;

pub use crate::napi::types::{
    IntlDatabaseInsertStrategy, IntlDiagnostic, IntlMessageBundlerOptions,
    IntlMessagesFileDescriptor, IntlSourceFileInsertionData,
};
use crate::public;
use crate::sources::MessagesFileDescriptor;
use intl_database_core::MessagesDatabase;

mod types;

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
    pub fn find_all_messages_files(
        &mut self,
        directories: Vec<String>,
        default_definition_locale: String,
    ) -> anyhow::Result<Vec<IntlMessagesFileDescriptor>> {
        let sources = public::find_all_messages_files(
            directories.iter().map(String::as_str),
            &default_definition_locale,
        );
        Ok(sources
            .into_iter()
            .map(IntlMessagesFileDescriptor::from)
            .collect())
    }

    #[napi]
    pub fn filter_all_messages_files(
        &mut self,
        files: Vec<String>,
        default_definition_locale: String,
    ) -> anyhow::Result<Vec<IntlMessagesFileDescriptor>> {
        let sources = public::filter_all_messages_files(
            files.iter().map(String::as_str),
            &default_definition_locale,
        );
        Ok(sources
            .into_iter()
            .map(IntlMessagesFileDescriptor::from)
            .collect())
    }

    #[napi]
    pub fn process_all_messages_files(
        &mut self,
        directories: Vec<IntlMessagesFileDescriptor>,
        strategy: IntlDatabaseInsertStrategy,
    ) -> anyhow::Result<Vec<IntlSourceFileInsertionData>> {
        let sources = public::process_all_messages_files(
            &mut self.database,
            directories.iter().map(MessagesFileDescriptor::from),
            strategy.into(),
        )?;
        Ok(sources
            .into_iter()
            .map(IntlSourceFileInsertionData::from)
            .collect())
    }

    #[napi]
    pub fn process_definitions_file(
        &mut self,
        file_path: String,
        locale: Option<String>,
        strategy: IntlDatabaseInsertStrategy,
    ) -> anyhow::Result<IntlSourceFileInsertionData> {
        let result = public::process_definitions_file(
            &mut self.database,
            &file_path,
            locale.as_ref().map(String::as_str),
            strategy.into(),
        )?;
        Ok(result.into())
    }

    #[napi]
    pub fn process_definitions_file_content(
        &mut self,
        file_path: String,
        content: String,
        locale: Option<String>,
        strategy: IntlDatabaseInsertStrategy,
    ) -> IntlSourceFileInsertionData {
        public::process_definitions_file_content(
            &mut self.database,
            &file_path,
            &content,
            locale.as_ref().map(String::as_str),
            strategy.into(),
        )
        .into()
    }

    #[napi]
    pub fn process_all_translation_files(
        &mut self,
        locale_map: HashMap<String, String>,
        strategy: IntlDatabaseInsertStrategy,
    ) -> anyhow::Result<Vec<IntlSourceFileInsertionData>> {
        let result =
            public::process_all_translation_files(&mut self.database, locale_map, strategy.into())?;
        Ok(result
            .into_iter()
            .map(IntlSourceFileInsertionData::from)
            .collect())
    }

    #[napi]
    pub fn process_translation_file(
        &mut self,
        file_path: String,
        locale: String,
        strategy: IntlDatabaseInsertStrategy,
    ) -> anyhow::Result<IntlSourceFileInsertionData> {
        let result = public::process_translation_file(
            &mut self.database,
            &file_path,
            &locale,
            strategy.into(),
        )?;
        Ok(result.into())
    }

    #[napi]
    pub fn process_translation_file_content(
        &mut self,
        file_path: String,
        locale: String,
        content: String,
        strategy: IntlDatabaseInsertStrategy,
    ) -> IntlSourceFileInsertionData {
        public::process_translation_file_content(
            &mut self.database,
            &file_path,
            &locale,
            &content,
            strategy.into(),
        )
        .into()
    }

    #[napi]
    pub fn get_known_locales(&self) -> Vec<String> {
        let locales = public::get_known_locales(&self.database);
        Vec::from_iter(locales.into_iter().map(|locale| locale.to_string()))
    }

    #[napi(ts_return_type = "IntlSourceFile")]
    pub fn get_source_file(&self, env: Env, file_path: String) -> anyhow::Result<JsUnknown> {
        let source = public::get_source_file(&self.database, &file_path)?;
        Ok(env.to_js_value(source)?)
    }

    #[napi]
    pub fn get_all_source_file_paths(&self) -> anyhow::Result<Vec<String>> {
        let paths = public::get_all_source_file_paths(&self.database)?;
        Ok(paths.into_iter().map(|path| path.to_string()).collect())
    }

    #[napi(ts_return_type = "Record<string, string>")]
    /// Return a map of all message keys contained in the given source file, where the key of the
    /// map is the hashed name and the value is the original.
    pub fn get_source_file_key_map(
        &self,
        env: Env,
        file_path: String,
    ) -> anyhow::Result<JsUnknown> {
        let hashes = public::get_source_file_key_map(&self.database, &file_path)?;
        Ok(env.to_js_value(&hashes)?)
    }

    #[napi]
    /// Return file paths for all definitions files with a translations path meta value that would
    /// include the given `translationPath`. This can be used to add reverse dependencies, and
    /// safely cache translations files with appropriate change detection.
    pub fn get_definitions_files_for_translations_path(
        &self,
        translations_path: String,
    ) -> anyhow::Result<Vec<String>> {
        let paths =
            public::get_definitions_files_for_translations_path(&self.database, &translations_path);
        Ok(paths.into_iter().map(|path| path.to_string()).collect())
    }

    #[napi(ts_return_type = "IntlMessage")]
    pub fn get_message(&self, env: Env, key: String) -> anyhow::Result<JsUnknown> {
        let definition = public::get_message(&self.database, &key)?;
        Ok(env.to_js_value(definition)?)
    }

    #[napi]
    pub fn generate_types(
        &self,
        source_file_path: String,
        output_file_path: String,
    ) -> anyhow::Result<()> {
        public::generate_types(&self.database, &source_file_path, &output_file_path)
    }

    #[napi]
    pub fn precompile(
        &self,
        file_path: String,
        locale: String,
        output_path: String,
        options: Option<IntlMessageBundlerOptions>,
    ) -> anyhow::Result<()> {
        public::precompile(
            &self.database,
            &file_path,
            &locale,
            &output_path,
            options.unwrap_or_default().into(),
        )
    }

    #[napi]
    pub fn precompile_to_buffer(
        &self,
        file_path: String,
        locale: String,
        options: Option<IntlMessageBundlerOptions>,
    ) -> anyhow::Result<Buffer> {
        let result = public::precompile_to_buffer(
            &self.database,
            &file_path,
            &locale,
            options.unwrap_or_default().into(),
        )?;
        Ok(result.into())
    }

    #[napi]
    pub fn validate_messages(&self) -> anyhow::Result<Vec<IntlDiagnostic>> {
        let result = public::validate_messages(&self.database)?;
        Ok(result.into_iter().map(IntlDiagnostic::from).collect())
    }

    #[napi]
    pub fn export_translations(
        &self,
        file_extension: Option<String>,
    ) -> anyhow::Result<Vec<String>> {
        public::export_translations(&self.database, file_extension)
    }

    #[napi(ts_return_type = "Record<string, IntlMessageValue | undefined>")]
    pub fn get_source_file_message_values(
        &self,
        env: Env,
        file_path: String,
    ) -> anyhow::Result<JsUnknown> {
        let result = public::get_source_file_message_values(&self.database, &file_path)?;
        Ok(env.to_js_value(&result)?)
    }
}

#[napi]
pub fn hash_message_key(key: String) -> String {
    public::hash_message_key(&key)
}

#[napi]
pub fn is_message_definitions_file(key: String) -> bool {
    public::is_message_definitions_file(&key)
}

#[napi]
pub fn is_message_translations_file(key: String) -> bool {
    public::is_message_translations_file(&key)
}
