use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::util::visitors::pound_finder::IcuPoundFinder;
use crate::{DiagnosticCategory, DiagnosticFix};
use intl_markdown::{AnyIcuExpression, Icu, Visit, VisitWith};
use intl_markdown_syntax::{EqIgnoreSpan, Syntax};

cst_validation_rule!(NoUnnecessaryPlural);

impl Visit for NoUnnecessaryPlural {
    // Need to use `visit_icu` because we want to capture the outer `{}`, which are part of the
    // `Icu` node rather than the inner ICU expression itself.
    fn visit_icu(&mut self, icu_node: &Icu) {
        icu_node.visit_children_with(self);
        let node = match icu_node.value() {
            AnyIcuExpression::IcuPlural(plural) => plural,
            _ => return,
        };

        // If the `other` arm isn't specified, it's technically an error, but we can't guarantee
        // that all cases will be covered, so the plural can't be undoubtably unnecessary.
        let Some(other_arm) = node.other_arm() else {
            return;
        };

        // If the plural expression contains a pound, it'll need to be replaced with a plain ICU
        // number expression.
        let pounds = {
            let mut finder = IcuPoundFinder::new();
            other_arm.visit_children_with(&mut finder);
            finder
        };

        let are_all_identical = node
            .arms()
            .children()
            .all(|arm| arm.value().eq_ignore_span(&other_arm.value()));
        if !are_all_identical {
            return;
        }

        let pound_replacement = format!("{{{name}, number}}", name = node.ident_token().text());
        let mut fixes = Vec::with_capacity(pounds.instances().len() + 2);
        for pound in pounds.instances() {
            fixes.push(DiagnosticFix::replace_node(
                pound.syntax(),
                &pound_replacement,
            ));
        }

        let (icu_start, icu_end) = icu_node.syntax().source_position();
        let (other_start, other_end) = other_arm.value().content().syntax().source_position();
        fixes.push(DiagnosticFix::remove_text((icu_start, other_start)));
        fixes.push(DiagnosticFix::remove_text((other_end, icu_end)));

        self.context.report(ValueDiagnostic {
            name: DiagnosticName::NoUnnecessaryPlural,
            span: Some(icu_node.syntax().source_position()),
            category: DiagnosticCategory::Suspicious,
            description: String::from(
                "All options in this plural are identical, it can be safely removed.",
            ),
            help: None,
            fixes,
        });
    }
}
