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
    let source_has_variables = source_variables
        .as_ref()
        .is_some_and(|variables| variables.count() > 0);

    for (locale, translation) in message.translations() {
        diagnostics.extend_from_value_diagnostics(
            validate_message_value(translation),
            translation.file_position.unwrap(),
            *locale,
        );
        if *locale == source_locale {
            continue;
        }

        let _translation_variables = match &translation.variables {
            // If the translation contains variables but the source does not,
            // it's likely unintended (the only time this should reasonably
            // happen is when translations are out-of-date, which should be
            // fixed automatically once the translations are imported again).
            Some(translation_variables)
                if !source_has_variables && translation_variables.count() > 0 =>
            {
                diagnostics.add(MessageDiagnostic {
                        key: message.key(),
                        file_position: translation.file_position.unwrap(),
                        locale: locale.clone(),
                        name: DiagnosticName::NoExtraTranslationVariables,
                        severity: DiagnosticSeverity::Warning,
                        description: "Translation includes variables, but the source message does not"
                            .into(),
                        help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into())
                    });
                continue;
            }

            Some(translated_variables) => translated_variables,
            // If the translation has no variables, but the source does, this
            // also likely not intentional, but still won't break things.
            _ => {
                if source_has_variables {
                    diagnostics.add(MessageDiagnostic {
                        key: message.key(),
                        file_position: translation.file_position.unwrap(),
                        locale: locale.clone(),
                        name: DiagnosticName::NoMissingSourceVariables,
                        severity: DiagnosticSeverity::Warning,
                        description: "Source message includes variables, but this translation has none.".into(),
                        help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into())
                    });
                }

                continue;
            }
        };
    }

    diagnostics.diagnostics
}
