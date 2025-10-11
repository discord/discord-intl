use crate::diagnostic::{DiagnosticFix, DiagnosticName, ValueDiagnostic};
use crate::validators::validator::Validator;
use crate::DiagnosticCategory;
use intl_database_core::MessageValue;

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
        let start_trimmed = content.trim_ascii_start();
        let end_trimmed = content.trim_ascii_end();

        if start_trimmed != content || end_trimmed != content {
            let mut fixes = vec![];
            if start_trimmed != content {
                fixes.push(
                    DiagnosticFix::remove_text((0, content.len() - start_trimmed.len()))
                        .with_message("Remove whitespace at the start of the message"),
                );
            }
            if end_trimmed != content {
                fixes.push(
                    DiagnosticFix::remove_text((end_trimmed.len(), content.len()))
                        .with_message("Remove whitespace at the end of the message"),
                );
            }

            diagnostics.push(ValueDiagnostic {
                name: DiagnosticName::NoTrimmableWhitespace,
                // Intentionally reporting no span, since this single diagnostic can apply to both
                // the start and the end of the message.
                span: None,
                category: DiagnosticCategory::Suspicious,
                description: "Avoid surrounding whitespace on messages".into(),
                help: Some("Leading and trailing whitespace are visually ambiguous when translating and leads to inconsistency".into()),
                fixes
            })
        }

        Some(diagnostics)
    }
}
