use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::{DiagnosticCategory, DiagnosticFix};
use intl_markdown::{IcuPlural, Visit, VisitWith};
use intl_markdown_syntax::Syntax;

cst_validation_rule!(NoLimitedPlurals);

impl Visit for NoLimitedPlurals {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        node.visit_children_with(self);

        if node.arms().children().any(|arm| !arm.is_exact_selector()) {
            return;
        }

        let mut fixes = node
            .arms()
            .children()
            .map(|arm| {
                DiagnosticFix::replace_token(
                    &arm.selector_token(),
                    &arm.selector_token().text()[1..],
                )
            })
            .collect::<Vec<_>>();
        fixes.push(DiagnosticFix::replace_token(&node.format_token(), "select"));

        self.context.report(ValueDiagnostic {
            name: DiagnosticName::NoLimitedPlurals,
            span: Some(node.syntax().source_position()),
            category: DiagnosticCategory::Suspicious,
            description:
                "Use `select` instead of `plural` when the set of possible options is limited"
                    .into(),
            help: None,
            fixes,
        });
    }
}
