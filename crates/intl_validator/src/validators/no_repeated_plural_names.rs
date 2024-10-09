use intl_database_core::{key_symbol, KeySymbol};
use intl_markdown::{IcuPlural, IcuPluralArm, IcuSelect, IcuVariable, Visitor};

use crate::DiagnosticSeverity;
use crate::message_diagnostic::Diagnostics;

pub struct NoRepeatedPluralNames {
    diagnostics: Diagnostics,
    current_plural_name_stack: Vec<Option<KeySymbol>>,
    is_in_plural_arm: bool,
}

impl NoRepeatedPluralNames {
    pub fn new(diagnostics: Diagnostics) -> Self {
        Self {
            diagnostics,
            current_plural_name_stack: Vec::with_capacity(2),
            is_in_plural_arm: false,
        }
    }
}

impl Visitor for NoRepeatedPluralNames {
    fn visit_icu_plural(&mut self, node: &IcuPlural) {
        self.current_plural_name_stack
            .push(Some(key_symbol(node.name())));
    }

    fn exit_icu_plural(&mut self, _node: &IcuPlural) {
        self.current_plural_name_stack.pop();
    }

    /// The name node of the ICU segments will always be included in the traversal, but we don't
    /// want to trigger repetition validations for those, since they are the original. We only care
    /// about instances of the same name within the plural _arms_.
    fn visit_icu_plural_arm(&mut self, _node: &IcuPluralArm) {
        self.is_in_plural_arm = true;
    }

    fn exit_icu_plural_arm(&mut self, _node: &IcuPluralArm) {
        self.is_in_plural_arm = false;
    }

    // `select` does not count as a plural, but contains IcuPluralArms, so we need a temporary
    // block on the stack to prevent it from getting read.
    fn visit_icu_select(&mut self, _node: &IcuSelect) {
        self.current_plural_name_stack.push(None);
    }
    fn exit_icu_select(&mut self, _node: &IcuSelect) {
        self.current_plural_name_stack.pop();
    }

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

        if plural_name.eq(node.name()) {
            self.diagnostics.borrow_mut().create(
                DiagnosticSeverity::Warning,
                "Plural variable names should use # instead of repeating the name of the variable",
                Some(String::from("Replace this variable name with #")),
            )
        }
    }
}
