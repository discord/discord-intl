use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::DiagnosticCategory;
use intl_markdown::{IcuPlural, Visit, VisitWith};
use intl_markdown_syntax::Syntax;

cst_validation_rule!(NoMissingPluralOther);

impl Visit for NoMissingPluralOther {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        let has_other = node
            .arms()
            .children()
            .any(|arm| arm.selector_token().text() == "other");

        if !has_other {
            self.context.report(ValueDiagnostic {
                name: DiagnosticName::NoMissingPluralOther,
                span: Some(node.syntax().source_position()),
                category: DiagnosticCategory::Correctness,
                description: "Plural expressions must include an `other` selector to capture all possible values".into(),
                help: Some("To switch between a fixed set of values, use `select` instead".into()),
                fixes: vec![]
            });
        }

        node.visit_children_with(self);
    }
}
