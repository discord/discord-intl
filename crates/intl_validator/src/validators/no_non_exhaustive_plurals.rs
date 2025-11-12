use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::{util, DiagnosticCategory};
use intl_markdown::{IcuPlural, Visit, VisitWith};
use intl_markdown_syntax::Syntax;

cst_validation_rule!(NoNonExhaustivePlurals);

impl Visit for NoNonExhaustivePlurals {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        node.visit_children_with(self);

        if !node.arms().children().all(|arm| arm.is_exact_selector()) {
            return;
        }

        self.context.report(ValueDiagnostic {
            name: DiagnosticName::NoNonExhaustivePlurals,
            span: Some(node.syntax().source_position()),
            category: DiagnosticCategory::Suspicious,
            description:
                "Use `select` instead of `plural` when the set of possible options is limited"
                    .into(),
            help: None,
            fixes: util::ops::replace_plural_with_select(node),
        });
    }
}
