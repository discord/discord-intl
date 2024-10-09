use intl_database_core::MessageValue;

use crate::diagnostic::ValueDiagnostic;
use crate::DiagnosticSeverity;
use crate::validators::validator::Validator;

pub struct NoTrimmableWhitespace;
impl NoTrimmableWhitespace {
    pub fn new() -> Self {
        Self
    }
}

impl Validator for NoTrimmableWhitespace {
    fn validate_raw(&mut self, message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        let mut diagnostics = vec![];
        let content = &message.raw;
        if content.trim_start() != content {
            diagnostics.push(ValueDiagnostic {
                span: None,
                severity: DiagnosticSeverity::Warning,
                description: "Avoid leading whitespace on messages".into(),
                help: Some("Leading whitespace is visually ambiguous when translating and leads to inconsistency".into())
            })
        }
        if content.trim_end() != content {
            diagnostics.push(ValueDiagnostic {
                span: None,
                severity: DiagnosticSeverity::Warning,
                description: "Avoid trailing whitespace on messages".into(),
                help: Some("Trailing whitespace is visually ambiguous when translating and leads to inconsistency".into())
            })
        }
        Some(diagnostics)
    }
}
