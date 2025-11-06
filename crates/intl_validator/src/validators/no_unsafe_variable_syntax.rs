use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::macros::cst_validation_rule;
use crate::{DiagnosticCategory, DiagnosticFix};
use intl_markdown::{Icu, Visit, VisitWith};
use intl_markdown_syntax::Syntax;

cst_validation_rule!(NoUnsafeVariableSyntax);

impl Visit for NoUnsafeVariableSyntax {
    fn visit_icu(&mut self, node: &Icu) {
        if node.is_unsafe() {
            self.context.report(ValueDiagnostic {
                name: DiagnosticName::NoUnsafeVariableSyntax,
                span: Some(node.syntax().source_position()),
                category: DiagnosticCategory::Style,
                description: String::from(
                    "Unsafe syntax `!!{}!!` has no effect in discord-intl and should be removed.",
                ),
                help: None,
                fixes: vec![
                    DiagnosticFix::replace_token(&node.l_curly_token(), "{"),
                    DiagnosticFix::replace_token(&node.r_curly_token(), "}"),
                ],
            });
        }
        node.visit_children_with(self);
    }
}
