use intl_database_core::MessageValue;
use intl_markdown::IcuPlural;
use intl_markdown_visitor::{visit_with_mut, Visit};
use std::collections::HashSet;

use crate::diagnostic::ValueDiagnostic;
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
        visit_with_mut(&message.parsed, self);
        Some(self.diagnostics.clone())
    }
}

impl Visit for NoRepeatedPluralOptions {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        let plural_name = node.name();
        let arm_names = node.arms().iter().map(|arm| arm.selector().as_str());
        let mut seen = HashSet::new();
        // Allotting enough capacity to handle basically every possible case. More than 4
        // repetitions is egregious and there will almost never be more than 1, but this just
        // ensures it's always consistent allocation.
        let mut repeated_names: Vec<&str> = Vec::with_capacity(4);

        for name in arm_names {
            if seen.contains(name) {
                repeated_names.push(name);
            } else {
                seen.insert(name);
            }
        }

        for name in repeated_names {
            let diagnostic = ValueDiagnostic {
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
