use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::validators::validator::Validator;
use crate::{DiagnosticFix, DiagnosticSeverity};
use intl_database_core::MessageValue;
use intl_markdown::{
    AnyIcuExpression, Icu, IcuPlural, IcuPluralArm, IcuSelectOrdinal, Visit, VisitWith,
};
use intl_markdown_syntax::{Syntax, SyntaxToken};

pub struct NoRepeatedPluralNames {
    diagnostics: Vec<ValueDiagnostic>,
    current_plural_name_stack: Vec<SyntaxToken>,
    plural_arm_depth: u8,
}

impl NoRepeatedPluralNames {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
            current_plural_name_stack: vec![],
            plural_arm_depth: 0,
        }
    }

    fn icu_can_be_converted_to_pound(&self, expression: &AnyIcuExpression) -> bool {
        match expression {
            // plain placeholders can always be converted.
            AnyIcuExpression::IcuPlaceholder(_) => true,
            // `number` with no style formatting can also be converted, like `{count, number}`.
            // Including a style in the number means it is overriding the default and should be
            // allowed to remain.
            AnyIcuExpression::IcuNumber(number) => number.style().is_none(),
            _ => false,
        }
    }

    fn report_repeated_plural(&mut self, node: &Icu) {
        let node_span = node.syntax().source_position();
        let diagnostic = ValueDiagnostic {
            name: DiagnosticName::NoRepeatedPluralNames,
            span: Some(node_span),
            severity: DiagnosticSeverity::Warning,
            description: String::from(
                "Plural variables should use # inside each arm instead of repeating the variable name",
            ),
            help: Some(String::from("Replace this placeholder with #")),
            fixes: vec![
                DiagnosticFix::replace_text(node_span, "#"),
            ],
        };

        self.diagnostics.push(diagnostic);
    }

    fn is_in_plural_arm(&self) -> bool {
        self.plural_arm_depth > 0
    }
}

impl Validator for NoRepeatedPluralNames {
    fn validate_cst(&mut self, message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        message.cst.visit_with(self);
        Some(self.diagnostics.clone())
    }
}

impl Visit for NoRepeatedPluralNames {
    fn visit_icu(&mut self, node: &Icu) {
        if !self.is_in_plural_arm() {
            node.visit_children_with(self);
            return;
        }

        let Some(current_plural) = self.current_plural_name_stack.last() else {
            return;
        };

        if current_plural.text().eq(node.ident_token().text())
            && self.icu_can_be_converted_to_pound(&node.value())
        {
            self.report_repeated_plural(node);
        }

        node.visit_children_with(self);
    }

    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        self.current_plural_name_stack.push(node.ident_token());
        node.visit_children_with(self);
        self.current_plural_name_stack.pop();
    }
    fn visit_icu_select_ordinal(&mut self, node: &IcuSelectOrdinal) {
        self.current_plural_name_stack.push(node.ident_token());
        node.visit_children_with(self);
        self.current_plural_name_stack.pop();
    }

    // NOTE: `select` does not count as a plural, so it's not included here.

    /// The name node of the ICU segments will always be included in the traversal, but we don't
    /// want to trigger repetition validations for those, since they are the original. We only care
    /// about instances of the same name within the plural _arms_.
    fn visit_icu_plural_arm(&mut self, node: &IcuPluralArm) {
        self.plural_arm_depth += 1;
        node.visit_children_with(self);
        self.plural_arm_depth -= 1;
    }
}
