use intl_database_core::Message;

pub use crate::content::validate_message_value;
pub use crate::diagnostic::MessageDiagnostic;
use crate::diagnostic::{DiagnosticName, MessageDiagnosticsBuilder};
pub use crate::severity::DiagnosticSeverity;

mod content;
mod diagnostic;
mod severity;
mod validators;

/// Validate the content of a message across all of its translations, returning
/// a full set of diagnostics with information about each one.
///
/// Only normal messages can be validated, since validation implies some source
/// of truth (a definition) to check against. Undefined messages can still have
/// diagnostics presented from general errors, like invalid syntax or
/// unsupported syntax.
pub fn validate_message(message: &Message) -> Vec<MessageDiagnostic> {
    let Some(source) = message.get_source_translation() else {
        return vec![];
    };

    // SAFETY: If the message has a source translation, it must have a source locale.
    let source_locale = message.source_locale().unwrap();
    let mut diagnostics = MessageDiagnosticsBuilder::new(message.key());

    let source_variables = &source.variables;
    let source_has_variables = source_variables.count() > 0;

    for (locale, translation) in message.translations() {
        diagnostics.extend_from_value_diagnostics(
            validate_message_value(translation),
            translation.file_position,
            *locale,
        );
        if *locale == source_locale {
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
                severity: DiagnosticSeverity::Warning,
                description: "Translation includes variables, but the source message does not"
                    .into(),
                help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into())
            });
        // If the translation has no variables, but the source does, this
        // also likely not intentional, but still won't break things.
        } else if source_has_variables && translation.variables.is_empty() {
            diagnostics.add(MessageDiagnostic {
                key: message.key(),
                file_position: translation.file_position,
                locale: locale.clone(),
                name: DiagnosticName::NoMissingSourceVariables,
                severity: DiagnosticSeverity::Warning,
                description: "Source message includes variables, but this translation has none.".into(),
                help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into())
            });
        }
    }

    diagnostics.diagnostics
}
