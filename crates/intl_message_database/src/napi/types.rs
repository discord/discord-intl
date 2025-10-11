use crate::sources::{MessagesFileDescriptor, SourceFileInsertionData};
use intl_database_core::{key_symbol, DatabaseError, DatabaseInsertStrategy};
use intl_database_exporter::CompiledMessageFormat;
use intl_validator::{DiagnosticFix, MessageDiagnostic};
use napi::{JsObject, JsString};
use napi_derive::napi;
use std::collections::HashMap;
use std::path::PathBuf;

#[napi(object)]
#[derive(Default)]
pub struct IntlMessageBundlerOptions {
    pub format: Option<IntlCompiledMessageFormat>,
    #[napi(js_name = "bundleSecrets")]
    pub bundle_secrets: Option<bool>,
}

impl Into<intl_database_exporter::IntlMessageBundlerOptions> for IntlMessageBundlerOptions {
    fn into(self) -> intl_database_exporter::IntlMessageBundlerOptions {
        let mut options = intl_database_exporter::IntlMessageBundlerOptions::default();
        if let Some(bundle_secrets) = self.bundle_secrets {
            options = options.with_bundle_secrets(bundle_secrets);
        }
        if let Some(format) = self.format {
            options = options.with_format(format.into());
        }
        options
    }
}

#[napi(object)]
pub struct IntlDiagnosticFix {
    pub message: Option<String>,
    pub start: u32,
    pub end: u32,
    pub replacement: String,
}

impl From<DiagnosticFix> for IntlDiagnosticFix {
    fn from(fix: DiagnosticFix) -> Self {
        Self {
            message: fix.message,
            start: fix.source_span.0 as u32,
            end: fix.source_span.1 as u32,
            replacement: fix.replacement,
        }
    }
}

#[napi(object)]
pub struct IntlDiagnostic {
    pub name: String,
    pub key: String,
    pub file: String,
    pub message_line: u32,
    pub message_col: u32,
    // JS Character index of the start of the diagnostic within the message.
    pub start: u32,
    // JS Character index of the end of the diagnostic within the message.
    pub end: u32,
    pub locale: String,
    pub category: String,
    pub description: String,
    pub help: Option<String>,
    pub fixes: Vec<IntlDiagnosticFix>,
}

impl From<MessageDiagnostic> for IntlDiagnostic {
    fn from(value: MessageDiagnostic) -> Self {
        Self {
            name: value.name.to_string(),
            key: value.key.to_string(),
            file: value.file_position.file.to_string(),
            message_line: value.file_position.line,
            message_col: value.file_position.col,
            start: value.span.map_or(0, |s| s.0 as u32),
            end: value.span.map_or(1, |s| s.1 as u32),
            locale: value.locale.to_string(),
            category: value.category.to_string(),
            description: value.description,
            help: value.help,
            fixes: value.fixes.into_iter().map(|fix| fix.into()).collect(),
        }
    }
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlSourceFile {
    #[napi(js_name = "type")]
    pub ty: String,
    pub file: String,
    pub locale: Option<JsString>,
    #[napi(js_name = "messageKeys")]
    pub message_keys: Vec<JsString>,
    pub meta: IntlSourceFileMeta,
}

#[napi(object)]
pub struct IntlSourceFileMeta {
    pub description: String,
    pub secret: bool,
    pub translate: bool,
    #[napi(js_name = "translationsPath")]
    pub translations_path: String,
    pub source_file_path: String,
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlMessageMeta {
    pub description: String,
    pub secret: bool,
    pub translate: bool,
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlMessage {
    /// Original, plain text name of the message given in its definition.
    pub key: String,
    /// Hashed version of the key, used everywhere for minification and obfuscation.
    #[napi(js_name = "hashedKey")]
    pub hashed_key: String,
    /// Map of all translations for this message, including the default.
    pub translations: HashMap<String, IntlMessageValue>,
    /// The source definition information for this message (locale and location).
    #[napi(js_name = "sourceLocale")]
    pub source_locale: Option<String>,
    /// Meta information about how to handle and process this message.
    pub meta: IntlMessageMeta,
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlMessageValue {
    pub raw: String,
    pub parsed: JsObject,
    pub variables: JsObject,
    #[napi(js_name = "filePosition")]
    pub file_position: JsObject,
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

#[napi(object)]
pub struct IntlMessagesFileDescriptor {
    #[napi(js_name = "filePath")]
    pub file_path: String,
    pub locale: String,
}

impl From<&IntlMessagesFileDescriptor> for MessagesFileDescriptor {
    fn from(value: &IntlMessagesFileDescriptor) -> Self {
        MessagesFileDescriptor {
            file_path: PathBuf::from(&value.file_path),
            locale: key_symbol(&value.locale),
        }
    }
}

impl From<MessagesFileDescriptor> for IntlMessagesFileDescriptor {
    fn from(value: MessagesFileDescriptor) -> Self {
        IntlMessagesFileDescriptor {
            file_path: value.file_path.to_string_lossy().to_string(),
            locale: value.locale.to_string(),
        }
    }
}

#[napi(object)]
pub struct IntlSourceFileError {
    pub name: String,
    pub key: Option<String>,
    pub locale: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub col: Option<u32>,
    pub message: String,
}

impl From<DatabaseError> for IntlSourceFileError {
    fn from(value: DatabaseError) -> Self {
        Self {
            name: value.name(),
            key: value.key().map(|s| s.to_string()),
            locale: value.locale().map(|s| s.to_string()),
            file: value.file().map(|s| s.to_string()),
            line: value.line(),
            col: value.col(),
            message: value.to_string(),
        }
    }
}

#[napi(object)]
pub struct IntlSourceFileInsertionData {
    pub file_key: String,
    pub inserted_count: u32,
    pub removed_count: u32,
    pub errors: Vec<IntlSourceFileError>,
}

impl From<SourceFileInsertionData> for IntlSourceFileInsertionData {
    fn from(value: SourceFileInsertionData) -> Self {
        IntlSourceFileInsertionData {
            file_key: value.file_key.to_string(),
            inserted_count: value.inserted_keys.len() as u32,
            removed_count: value.removed_keys.len() as u32,
            errors: value
                .errors
                .into_iter()
                .map(IntlSourceFileError::from)
                .collect(),
        }
    }
}

#[napi]
pub enum IntlDatabaseInsertStrategy {
    Create,
    Update,
    Replace,
}

impl From<DatabaseInsertStrategy> for IntlDatabaseInsertStrategy {
    fn from(value: DatabaseInsertStrategy) -> Self {
        match value {
            DatabaseInsertStrategy::NewSourceFile => IntlDatabaseInsertStrategy::Create,
            DatabaseInsertStrategy::UpdateSourceFile => IntlDatabaseInsertStrategy::Update,
            DatabaseInsertStrategy::ReplaceExisting => IntlDatabaseInsertStrategy::Replace,
        }
    }
}

impl From<IntlDatabaseInsertStrategy> for DatabaseInsertStrategy {
    fn from(value: IntlDatabaseInsertStrategy) -> Self {
        match value {
            IntlDatabaseInsertStrategy::Create => DatabaseInsertStrategy::NewSourceFile,
            IntlDatabaseInsertStrategy::Update => DatabaseInsertStrategy::UpdateSourceFile,
            IntlDatabaseInsertStrategy::Replace => DatabaseInsertStrategy::ReplaceExisting,
        }
    }
}
