use crate::DiagnosticSeverity;
use intl_database_core::{FilePosition, KeySymbol};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DiagnosticName {
    NoExtraTranslationVariables,
    NoMissingSourceVariables,
    NoRepeatedPluralNames,
    NoRepeatedPluralOptions,
    NoTrimmableWhitespace,
    NoUnicodeVariableNames,
}

impl Display for DiagnosticName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DiagnosticName {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticName::NoExtraTranslationVariables => "NoExtraTranslationVariables",
            DiagnosticName::NoMissingSourceVariables => "NoMissingSourceVariables",
            DiagnosticName::NoRepeatedPluralNames => "NoRepeatedPluralNames",
            DiagnosticName::NoRepeatedPluralOptions => "NoRepeatedPluralOptions",
            DiagnosticName::NoTrimmableWhitespace => "NoTrimmableWhitespace",
            DiagnosticName::NoUnicodeVariableNames => "NoUnicodeVariableNames",
        }
    }
}

type TextRange = (usize, usize);

#[derive(Debug, Clone)]
pub struct DiagnosticFix {
    pub message: Option<String>,
    pub source_span: TextRange,
    pub replacement: String,
}

impl DiagnosticFix {
    pub fn remove_text(source_span: TextRange) -> Self {
        DiagnosticFix {
            message: None,
            source_span,
            replacement: "".into(),
        }
    }

    pub fn replace_text(source_span: TextRange, replacement: &str) -> Self {
        DiagnosticFix {
            message: None,
            source_span,
            replacement: replacement.into(),
        }
    }

    pub fn insert_text(start: usize, new_text: &str) -> Self {
        DiagnosticFix {
            message: None,
            source_span: (start, start),
            replacement: new_text.into(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct MessageDiagnostic {
    pub key: KeySymbol,
    pub file_position: FilePosition,
    pub locale: KeySymbol,
    pub name: DiagnosticName,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
    pub fixes: Vec<DiagnosticFix>,
}

#[derive(Debug, Clone)]
pub struct ValueDiagnostic {
    pub name: DiagnosticName,
    pub span: Option<(usize, usize)>,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
    pub fixes: Vec<DiagnosticFix>,
}

pub struct MessageDiagnosticsBuilder {
    pub diagnostics: Vec<MessageDiagnostic>,
    pub key: KeySymbol,
}

impl MessageDiagnosticsBuilder {
    pub fn new(key: KeySymbol) -> Self {
        Self {
            diagnostics: vec![],
            key,
        }
    }

    pub fn add(&mut self, diagnostic: MessageDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend_from_value_diagnostics(
        &mut self,
        value_diagnostics: Vec<ValueDiagnostic>,
        file_position: FilePosition,
        locale: KeySymbol,
    ) {
        let converted_diagnostics =
            value_diagnostics
                .into_iter()
                .map(|diagnostic| MessageDiagnostic {
                    key: self.key,
                    file_position,
                    locale,
                    name: diagnostic.name,
                    severity: diagnostic.severity,
                    description: diagnostic.description,
                    help: diagnostic.help,
                    fixes: diagnostic.fixes,
                });

        self.diagnostics.extend(converted_diagnostics);
    }
}
