use crate::public::MultiProcessingResult;
use crate::sources::MessagesFileDescriptor;
use intl_database_core::key_symbol;
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
pub struct IntlMultiProcessingFailure {
    pub file: String,
    pub error: String,
}

#[napi(object)]
pub struct IntlMultiProcessingResult {
    pub processed: Vec<String>,
    pub failed: Vec<IntlMultiProcessingFailure>,
}

impl From<MultiProcessingResult> for IntlMultiProcessingResult {
    fn from(value: MultiProcessingResult) -> Self {
        IntlMultiProcessingResult {
            processed: value
                .processed
                .into_iter()
                .map(|value| value.to_string())
                .collect(),
            failed: value
                .failed
                .into_iter()
                .map(|(key, error)| IntlMultiProcessingFailure {
                    file: key.to_string(),
                    error: error.to_string(),
                })
                .collect(),
        }
    }
}
