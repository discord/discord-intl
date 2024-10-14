use std::collections::HashMap;

use napi::{JsNumber, JsObject};
use napi_derive::napi;

use intl_database_exporter::CompiledMessageFormat;
use intl_validator::MessageDiagnostic;

#[napi(object)]
pub struct IntlDiagnostic {
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
    #[napi(js_name = "messageKeys")]
    pub message_keys: Vec<JsNumber>,
    pub meta: IntlMessageMeta,
    pub locale: Option<JsNumber>,
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlMessageMeta {
    pub secret: bool,
    pub translate: bool,
    #[napi(js_name = "bundleSecrets")]
    pub bundle_secrets: bool,
    #[napi(js_name = "translationsPath")]
    pub translations_path: String,
}

// This is an unused struct purely for generating functional TS types.
#[napi(object)]
pub struct IntlMessage {
    /// Original, plain text name of the message given in its definition.
    pub key: String,
    /// Hashed version of the key, used everywhere for minification and obfuscation.
    pub hashed_key: String,
    /// Map of all translations for this message, including the default.
    pub translations: HashMap<String, IntlMessageValue>,
    /// The source definition information for this message (locale and location).
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
