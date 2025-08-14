use crate::diagnostic::{DiagnosticFix, DiagnosticName, ValueDiagnostic};
use crate::validators::validator::Validator;
use crate::DiagnosticSeverity;
use intl_database_core::MessageValue;
use intl_markdown::SyntaxElement;

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
                name: DiagnosticName::NoTrimmableWhitespace,
                span: None,
                severity: DiagnosticSeverity::Warning,
                description: "Avoid leading whitespace on messages".into(),
                help: Some("Leading whitespace is visually ambiguous when translating and leads to inconsistency".into()),
                fixes: vec![],
            })
        }
        if content.trim_end() != content {
            diagnostics.push(ValueDiagnostic {
                name: DiagnosticName::NoTrimmableWhitespace,
                span: None,
                severity: DiagnosticSeverity::Warning,
                description: "Avoid trailing whitespace on messages".into(),
                help: Some("Trailing whitespace is visually ambiguous when translating and leads to inconsistency".into()),
                fixes: vec![DiagnosticFix {
                    message: Some("Remove whitespace at ends of message".into()),
                    source_span: (0, 0),
                    replacement: SyntaxElement::Node()
                }],
            })
        }
        Some(diagnostics)
    }
}
