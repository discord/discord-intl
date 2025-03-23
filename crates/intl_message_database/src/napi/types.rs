use crate::sources::{MessagesFileDescriptor, SourceFileInsertionData};
use intl_database_core::{key_symbol, DatabaseError};
use intl_database_exporter::CompiledMessageFormat;
use intl_validator::MessageDiagnostic;
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
pub struct IntlDiagnostic {
    pub name: String,
    pub key: String,
    pub file: String,
    pub line: u32,
    pub col: u32,
    pub locale: String,
    pub severity: String,
    pub description: String,
    pub help: Option<String>,
}

impl From<MessageDiagnostic> for IntlDiagnostic {
    fn from(value: MessageDiagnostic) -> Self {
        Self {
            name: value.name.to_string(),
            key: value.key.to_string(),
            file: value.file_position.file.to_string(),
            line: value.file_position.line,
            col: value.file_position.col,
            locale: value.locale.to_string(),
            severity: value.severity.to_string(),
            description: value.description,
            help: value.help,
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
