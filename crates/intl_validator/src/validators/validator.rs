use intl_database_core::{KeySymbol, MessageValue};

use crate::diagnostic::ValueDiagnostic;

pub trait Validator {

    fn validate_raw(&mut self, _message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        None
    }

    fn validate_cst(&mut self, _message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        None
    }
}

pub struct ValidatorContext {
    pub locale: KeySymbol,
    pub diagnostics: Vec<ValueDiagnostic>,
}

impl ValidatorContext {
    pub fn new(locale: KeySymbol) -> Self {
        Self {
            locale,
            diagnostics: vec![],
        }
    }

    pub fn report(&mut self, diagnostic: ValueDiagnostic) {
        self.diagnostics.push(diagnostic);
    }
}
