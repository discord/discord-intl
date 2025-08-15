use intl_database_core::{key_symbol, KeySymbol, MessageValue};
use intl_markdown::{IcuPlural, IcuPluralArm, IcuSelect, IcuVariable, Visit, VisitWith};

use crate::diagnostic::{DiagnosticName, ValueDiagnostic};
use crate::validators::validator::Validator;
use crate::DiagnosticSeverity;

pub struct NoRepeatedPluralNames {
    diagnostics: Vec<ValueDiagnostic>,
    current_plural_name_stack: Vec<Option<KeySymbol>>,
    is_in_plural_arm: bool,
}

impl NoRepeatedPluralNames {
    pub fn new() -> Self {
        Self {
            diagnostics: vec![],
            current_plural_name_stack: Vec::with_capacity(2),
            is_in_plural_arm: false,
        }
    }
}

impl Validator for NoRepeatedPluralNames {
    fn validate_ast(&mut self, message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        message.cst.visit_with(self);
        Some(self.diagnostics.clone())
    }
}

impl Visit for NoRepeatedPluralNames {
    fn visit_icu_variable(&mut self, node: &IcuVariable) {
        if !self.is_in_plural_arm {
            return;
        }

        let Some(current_plural_entry) = self.current_plural_name_stack.last() else {
            return;
        };

        // If the current stack value is empty, then repetition can't be validated.
        let Some(plural_name) = current_plural_entry else {
            return;
        };

        if plural_name.eq(&node.ident_token().text()) {
            let diagnostic = ValueDiagnostic {
                name: DiagnosticName::NoRepeatedPluralNames,
                span: None,
                severity: DiagnosticSeverity::Warning,
                description: String::from("Plural variable names should use # instead of repeating the name of the variable"),
                help: Some(String::from("Replace this variable name with #")),
                fixes: vec![],
            };

            self.diagnostics.push(diagnostic);
        }
    }

    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        self.current_plural_name_stack
            .push(Some(key_symbol(node.variable().ident_token().text())));
        node.visit_children_with(self);
        self.current_plural_name_stack.pop();
    }

    // `select` does not count as a plural, but contains IcuPluralArms, so we need a temporary
    // block on the stack to prevent it from getting read.
    fn visit_icu_select(&mut self, node: &IcuSelect) {
        self.current_plural_name_stack.push(None);
        node.visit_children_with(self);
        self.current_plural_name_stack.pop();
    }

    /// The name node of the ICU segments will always be included in the traversal, but we don't
    /// want to trigger repetition validations for those, since they are the original. We only care
    /// about instances of the same name within the plural _arms_.
    fn visit_icu_plural_arm(&mut self, node: &IcuPluralArm) {
        self.is_in_plural_arm = true;
        node.visit_children_with(self);
        self.is_in_plural_arm = false;
    }
}
