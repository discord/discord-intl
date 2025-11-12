use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::{DiagnosticCategory, DiagnosticFix};
use intl_markdown::{IcuPlural, IcuPluralArm, Visit, VisitWith};
use intl_markdown_syntax::{EqIgnoreSpan, Syntax};
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
        let mut repeated_selectors: Vec<IcuPluralArm> = Vec::with_capacity(4);
        let mut duplicate_arms: Vec<IcuPluralArm> = Vec::with_capacity(4);

        let other_arm = node.other_arm();

        for arm in node.arms().children() {
            let selector = arm.selector_token();
            let name = selector.text();
            if seen.contains(name) {
                repeated_selectors.push(arm.clone());
            } else {
                seen.insert(name.to_string());
            }

            if other_arm
                .as_ref()
                .is_some_and(|other| arm.selector_token().text() != other.selector_token().text())
            {
                if arm
                    .value()
                    .eq_ignore_span(&other_arm.as_ref().unwrap().value())
                {
                    duplicate_arms.push(arm);
                }
            }
        }

        for arm in repeated_selectors {
            self.context.report(ValueDiagnostic {
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
            });
        }

        for arm in duplicate_arms {
            self.context.report(ValueDiagnostic {
                name: DiagnosticName::NoRepeatedPluralOptions,
                span: Some(arm.syntax().source_position()),
                category: DiagnosticCategory::Suspicious,
                description: String::from(
                    "Plural option matches `other` exactly and can be removed",
                ),
                help: None,
                fixes: vec![DiagnosticFix::remove_node(arm.syntax())],
            })
        }
    }
}
