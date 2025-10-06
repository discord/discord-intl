use intl_database_core::MessageValue;

use crate::diagnostic::ValueDiagnostic;

pub trait Validator {
    fn validate_raw(&mut self, _message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        None
    }

    fn validate_cst(&mut self, _message: &MessageValue) -> Option<Vec<ValueDiagnostic>> {
        None
    }
}
