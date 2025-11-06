use intl_database_core::{KeySymbol, MessageValue};

use crate::diagnostic::ValueDiagnostic;
use crate::validators;
use crate::validators::validator::Validator;

macro_rules! run_validator {
    ($name:ident, $locale:ident, $message:ident, $diagnostics:ident) => {{
        let mut validator = validators::$name::new($locale);
        if let Some(result) = validator.validate_raw($message) {
            $diagnostics.extend(result);
        }
        if let Some(result) = validator.validate_cst($message) {
            $diagnostics.extend(result);
        }
    }};
}

pub fn validate_message_value(message: &MessageValue, locale: KeySymbol) -> Vec<ValueDiagnostic> {
    let mut diagnostics: Vec<ValueDiagnostic> = vec![];
    run_validator!(NoUnicodeVariableNames, locale, message, diagnostics);
    run_validator!(NoRepeatedPluralNames, locale, message, diagnostics);
    run_validator!(NoRepeatedPluralOptions, locale, message, diagnostics);
    run_validator!(NoTrimmableWhitespace, locale, message, diagnostics);
    run_validator!(NoUnsafeVariableSyntax, locale, message, diagnostics);
    run_validator!(NoAvoidableExactPlurals, locale, message, diagnostics);

    diagnostics
}
