use discord_intl_database_core::{
    file_key_symbol, DiagnosticCode, FilePosition, MessageMeta, MessageValue, SourceDiagnostic,
    SourceFileMeta,
};
use discord_intl_database_js_source::JsMessageSource;
use discord_intl_database_json_source::JsonMessageSource;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedFileMeta {
    pub description: Option<String>,
    pub secret: bool,
    pub translate: bool,
    pub tags: Option<Vec<String>>,
    pub translations_path: String,
}
impl From<SourceFileMeta> for ExtractedFileMeta {
    fn from(meta: SourceFileMeta) -> Self {
        Self {
            description: meta.description,
            secret: meta.secret,
            translate: meta.translate,
            tags: meta.tags,
            translations_path: meta.translations_path.to_string_lossy().into_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractedMessage {
    pub key: String,
    pub text: String,
    pub meta: Option<MessageMeta>,
    pub position: FilePosition,
    pub source_offsets: Vec<(u32, u32)>,
}
impl ExtractedMessage {
    fn new(key: &str, value: MessageValue, meta: Option<MessageMeta>) -> Self {
        Self {
            key: key.into(),
            text: value.raw,
            meta,
            position: value.file_position,
            source_offsets: value.source_offsets.as_slice().to_vec(),
        }
    }
}

/// Diagnostics observe dropped or lossy input; payloads preserve the legacy interpreter's results.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceExtractionReport {
    pub meta: Option<ExtractedFileMeta>,
    pub messages: Vec<ExtractedMessage>,
    pub diagnostics: Vec<SourceDiagnostic>,
    pub complete: bool,
}

/// Extract static JS definitions or flat JSON/JSONA translations without database writes.
pub fn extract_file_content_with_diagnostics(
    file_path: &str,
    content: &str,
) -> SourceExtractionReport {
    let file = file_key_symbol(file_path);
    if file_path.ends_with(".js") {
        match JsMessageSource.extract_definitions_with_diagnostics(file, content) {
            Ok(report) => SourceExtractionReport {
                meta: Some(ExtractedFileMeta::from(report.meta)),
                messages: report
                    .definitions
                    .into_iter()
                    .map(|d| ExtractedMessage::new(d.name.as_str(), d.value, Some(d.meta)))
                    .collect(),
                diagnostics: report.diagnostics,
                complete: report.complete,
            },
            Err(error) => SourceExtractionReport {
                meta: None,
                messages: Vec::new(),
                diagnostics: error.diagnostics,
                complete: false,
            },
        }
    } else if file_path.ends_with(".json") || file_path.ends_with(".jsona") {
        let report = JsonMessageSource.extract_translations_with_diagnostics(file, content);
        SourceExtractionReport {
            meta: None,
            messages: report
                .messages
                .into_iter()
                .map(|m| ExtractedMessage::new(m.name.as_str(), m.value, None))
                .collect(),
            diagnostics: report.diagnostics,
            complete: report.complete,
        }
    } else {
        SourceExtractionReport {
            meta: None,
            messages: Vec::new(),
            diagnostics: vec![SourceDiagnostic::new(
                DiagnosticCode::UnsupportedForm,
                file,
                content,
                (0, 0),
                "No source implementation for this file extension",
            )],
            complete: false,
        }
    }
}
