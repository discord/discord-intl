use intl_database_core::MessageValue;

use crate::diagnostic::ValueDiagnostic;
use crate::validators;
use crate::validators::validator::Validator;

pub fn validate_message_value(message: &MessageValue) -> Vec<ValueDiagnostic> {
    let mut diagnostics: Vec<ValueDiagnostic> = vec![];
    let mut validators: Vec<Box<dyn Validator>> = vec![
        Box::new(validators::NoUnicodeVariableNames::new()),
        Box::new(validators::NoRepeatedPluralNames::new()),
        Box::new(validators::NoRepeatedPluralOptions::new()),
        Box::new(validators::NoTrimmableWhitespace::new()),
        Box::new(validators::NoUnsafeVariableSyntax::new()),
        Box::new(validators::NoAvoidableExactPlurals::new()),
    ];
    for validator in validators.iter_mut() {
        if let Some(result) = validator.validate_raw(message) {
            diagnostics.extend(result);
        }
        if let Some(result) = validator.validate_cst(message) {
            diagnostics.extend(result);
        }
    }

    diagnostics
}
