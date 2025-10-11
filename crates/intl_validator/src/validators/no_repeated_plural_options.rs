use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::DiagnosticCategory;
use intl_markdown::{IcuPlural, IcuPluralArm, Visit, VisitWith};
use intl_markdown_syntax::Syntax;
use std::collections::HashSet;

cst_validation_rule!(NoRepeatedPluralOptions);

impl Visit for NoRepeatedPluralOptions {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        let ident = node.ident_token();
        let plural_name = ident.text();
        let mut seen = HashSet::new();
        // Allotting enough capacity to handle basically every possible case. More than 4
        // repetitions is egregious and there will almost never be more than 1, but this just
        // ensures it's always consistent allocation.
        let mut repeated_names: Vec<IcuPluralArm> = Vec::with_capacity(4);

        for arm in node.arms().children() {
            let selector = arm.selector_token();
            let name = selector.text();
            if seen.contains(name) {
                repeated_names.push(arm);
            } else {
                seen.insert(name.to_string());
            }
        }

        for arm in repeated_names {
            let diagnostic = ValueDiagnostic {
                name: DiagnosticName::NoRepeatedPluralOptions,
                span: Some(arm.syntax().source_position()),
                category: DiagnosticCategory::Correctness,
                description: String::from(
                    "Plural options must not be repeated within the same plural selector",
                ),
                help: Some(
                    format!(
                        "The option '{name}' is present more than once in the plural value '{plural_name}'. Remove or rename one of these options to fix it.",
                        name = arm.selector_token().text()
                    )
                ),
                fixes: vec![],
            };

            self.diagnostics.push(diagnostic);
        }
    }
}
