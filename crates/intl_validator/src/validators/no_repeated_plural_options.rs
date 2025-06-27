use intl_database_core::MessageValue;
use intl_markdown::{IcuPlural, Visit, VisitWith};
use std::collections::HashSet;

use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::validators::validator::Validator;
use crate::DiagnosticSeverity;

pub struct NoRepeatedPluralOptions {
    diagnostics: Vec<ValueDiagnostic>,
}

impl NoRepeatedPluralOptions {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
        }
    }
}

impl Validator for NoRepeatedPluralOptions {
    fn validate_ast(&mut self, message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        message.cst.visit_with(self);
        Some(self.diagnostics.clone())
    }
}

impl Visit for NoRepeatedPluralOptions {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        let ident = node.variable().ident_token();
        let plural_name = ident.text();
        let mut seen = HashSet::new();
        // Allotting enough capacity to handle basically every possible case. More than 4
        // repetitions is egregious and there will almost never be more than 1, but this just
        // ensures it's always consistent allocation.
        let mut repeated_names: Vec<String> = Vec::with_capacity(4);

        for arm in node.arms().children() {
            let selector = arm.selector_token();
            let name = selector.text().to_string();
            if seen.contains(&name) {
                repeated_names.push(name);
            } else {
                seen.insert(name);
            }
        }

        for name in repeated_names {
            let diagnostic = ValueDiagnostic {
                name: DiagnosticName::NoRepeatedPluralOptions,
                span: None,
                severity: DiagnosticSeverity::Error,
                description: String::from(
                    "Plural options must be unique within the plural selector",
                ),
                help: Some(format!("The option '{name}' is present more than once in the plural value '{plural_name}'. Remove or rename one of these options to fix it.")),
            };

            self.diagnostics.push(diagnostic);
        }
    }
}
