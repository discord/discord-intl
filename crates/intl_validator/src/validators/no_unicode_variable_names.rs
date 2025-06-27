use intl_database_core::MessageValue;
use intl_markdown::{IcuVariable, Visit, VisitWith};

use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::validators::validator::Validator;
use crate::DiagnosticSeverity;

pub struct NoUnicodeVariableNames {
    diagnostics: Vec<ValueDiagnostic>,
}

impl NoUnicodeVariableNames {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
        }
    }
}

impl Validator for NoUnicodeVariableNames {
    fn validate_ast(&mut self, message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        message.cst.visit_with(self);
        Some(self.diagnostics.clone())
    }
}

impl Visit for NoUnicodeVariableNames {
    fn visit_icu_variable(&mut self, node: &IcuVariable) {
        let ident = node.ident_token();
        let name = ident.text();
        if !name.is_ascii() {
            let help_text = format!("\"{name}\" should be renamed to only use ASCII characters. If this is a translation, ensure the name matches the expected name in the source text");
            self.diagnostics.push(ValueDiagnostic {
                name: DiagnosticName::NoUnicodeVariableNames,
                span: None,
                severity: DiagnosticSeverity::Error,
                description: "Variable names should not contain unicode characters to avoid ambiguity during translation".into(),
                help: Some(help_text),
            });
        }
    }
}
