use intl_markdown::{IcuVariable, Visitor};

use crate::DiagnosticSeverity;
use crate::message_diagnostic::Diagnostics;

pub struct NoUnicodeVariableNames {
    diagnostics: Diagnostics,
}

impl NoUnicodeVariableNames {
    pub fn new(diagnostics: Diagnostics) -> Self {
        Self { diagnostics }
    }
}

impl Visitor for NoUnicodeVariableNames {
    fn visit_icu_variable(&mut self, node: &IcuVariable) {
        let name = node.name();
        if !name.is_ascii() {
            let help_text = format!("\"{name}\" should be renamed to only use ASCII characters. If this is a translation, ensure the name matches the expected name in the source text");
            self.diagnostics.borrow_mut().create(
                DiagnosticSeverity::Error,
                "Variable names should not contain unicode characters to avoid ambiguity during translation",
                Some(help_text)
            );
        }
    }
}
