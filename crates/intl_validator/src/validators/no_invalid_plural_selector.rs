use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::util::plural_rules::get_valid_cardinal_selectors;
use crate::{DiagnosticCategory, DiagnosticFix};
use intl_markdown::{IcuPlural, Visit, VisitWith};
use intl_markdown_syntax::Syntax;

cst_validation_rule!(NoInvalidPluralSelector);

impl Visit for NoInvalidPluralSelector {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        let valid_selectors =
            get_valid_cardinal_selectors(&self.context.locale).unwrap_or_default();
        for arm in node.arms().children() {
            if arm.is_exact_selector() {
                continue;
            }

            let selector = arm.selector_token();
            if !valid_selectors.contains(selector.text()) {
                let description = format!(
                    "`{}` is not a valid plural selector in locale `{}`. Valid options are: {}",
                    selector.text(),
                    self.context.locale,
                    valid_selectors
                        .iter()
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let replacement = match selector.text() {
                    "zero" => Some("=0"),
                    "one" => Some("=1"),
                    _ => None,
                };
                self.context.report(ValueDiagnostic {
                    name: DiagnosticName::NoInvalidPluralSelector,
                    span: Some(arm.syntax().source_position()),
                    category: DiagnosticCategory::Correctness,
                    description,
                    help: None,
                    fixes: replacement.map_or(vec![], |replacement| {
                        vec![DiagnosticFix::replace_token(&selector, replacement)]
                    }),
                });
            }
        }

        node.visit_children_with(self);
    }
}
