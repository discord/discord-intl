use intl_database_core::{FilePosition, KeySymbol};

use crate::DiagnosticSeverity;

pub struct MessageDiagnostic {
    pub key: KeySymbol,
    pub file_position: FilePosition,
    pub locale: KeySymbol,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValueDiagnostic {
    pub span: Option<usize>,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
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
                    severity: diagnostic.severity,
                    description: diagnostic.description,
                    help: diagnostic.help,
                });

        self.diagnostics.extend(converted_diagnostics);
    }
}
