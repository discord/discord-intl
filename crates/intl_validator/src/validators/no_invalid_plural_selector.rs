use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::util::plural_rules::get_valid_cardinal_selectors;
use crate::DiagnosticCategory;
use intl_markdown::{IcuPlural, Visit, VisitWith};
use intl_markdown_syntax::Syntax;

cst_validation_rule!(NoInvalidPluralSelector);

impl Visit for NoInvalidPluralSelector {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        let valid_selectors =
            get_valid_cardinal_selectors(&self.context.locale).unwrap_or_default();
        for arm in node.arms().children() {
            let selector = arm.selector_token();
            let is_exact = selector.text().starts_with("=");
            if is_exact {
                continue;
            }

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
                self.context.report(ValueDiagnostic {
                    name: DiagnosticName::NoInvalidPluralSelector,
                    span: Some(arm.syntax().source_position()),
                    category: DiagnosticCategory::Correctness,
                    description,
                    help: None,
                    fixes: vec![],
                });
            }
        }

        node.visit_children_with(self);
    }
}
