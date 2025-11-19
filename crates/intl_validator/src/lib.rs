extern crate core;

use intl_database_core::Message;

pub use crate::category::DiagnosticCategory;
pub use crate::content::validate_message_value;
pub use crate::diagnostic::{
    DiagnosticName, MessageDiagnostic, MessageDiagnosticsBuilder, TextRange, ValueDiagnostic,
};
pub use crate::fix::{apply_fixes, DiagnosticFix};

mod category;
mod content;
mod diagnostic;
mod fix;
mod macros;
mod util;
pub mod validators;

/// Validate the content of a message across all of its translations, returning
/// a full set of diagnostics with information about each one.
///
/// Only normal messages can be validated, since validation implies some source
/// of truth (a definition) to check against. Undefined messages can still have
/// diagnostics presented from general errors, like invalid syntax or
/// unsupported syntax.
pub fn validate_message(message: &Message) -> Vec<MessageDiagnostic> {
    let (source_has_variables, source_locale) =
        if let Some(source) = message.get_source_translation() {
            (
                !source.variables.is_empty(),
                // SAFETY: If the source exists, then the locale for it must also exist.
                Some(message.source_locale().unwrap()),
            )
        } else {
            (false, None)
        };

    let mut diagnostics = MessageDiagnosticsBuilder::new(message.key());

    for (locale, translation) in message.translations() {
        diagnostics.extend_from_value_diagnostics(
            validate_message_value(translation, *locale),
            translation,
            *locale,
        );
        if source_locale.is_some_and(|source_locale| *locale == source_locale) {
            continue;
        }

        // If the translation contains variables but the source does not,
        // it's likely unintended (the only time this should reasonably
        // happen is when translations are out-of-date, which should be
        // fixed automatically once the translations are imported again).
        if !source_has_variables && !translation.variables.is_empty() {
            diagnostics.add(MessageDiagnostic {
                key: message.key(),
                file_position: translation.file_position,
                locale: locale.clone(),
                name: DiagnosticName::NoExtraTranslationVariables,
                category: DiagnosticCategory::Suspicious,
                description: "Translation includes variables, but the source message does not"
                    .into(),
                help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into()),
                span: None,
                fixes: vec![],
            });
        // If the translation has no variables, but the source does, this
        // also likely not intentional, but still won't break things.
        } else if source_has_variables && translation.variables.is_empty() {
            diagnostics.add(MessageDiagnostic {
                key: message.key(),
                file_position: translation.file_position,
                locale: locale.clone(),
                name: DiagnosticName::NoMissingSourceVariables,
                category: DiagnosticCategory::Suspicious,
                description: "Source message includes variables, but this translation has none.".into(),
                help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into()),
                span: None,
                fixes: vec![],
            });
        }
    }

    diagnostics.diagnostics
}
