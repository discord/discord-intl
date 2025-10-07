use intl_markdown::{IcuPlaceholder, Visit, VisitWith};

use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::DiagnosticSeverity;

cst_validation_rule!(NoUnicodeVariableNames);

impl Visit for NoUnicodeVariableNames {
    fn visit_icu_placeholder(&mut self, node: &IcuPlaceholder) {
        let ident = node.ident_token();
        let name = ident.text();
        if !name.is_ascii() {
            let help_text = format!("\"{name}\" should be renamed to only use ASCII characters. If this is a translation, ensure the name matches the expected name in the source text");
            self.diagnostics.push(ValueDiagnostic {
                name: DiagnosticName::NoUnicodeVariableNames,
                span: Some(ident.source_position()),
                severity: DiagnosticSeverity::Error,
                description: "Variable names should not contain unicode characters to avoid ambiguity during translation".into(),
                help: Some(help_text),
                fixes: vec![]
            });
        }
    }
}
