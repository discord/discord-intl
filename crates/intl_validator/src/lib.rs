extern crate core;

use intl_database_core::{KeySymbolSet, Message};

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

fn variable_name_list(names: &KeySymbolSet) -> String {
    let names = names.iter().map(|name| name.as_ref()).collect::<Vec<_>>();
    names.join(", ")
}

/// Validate the content of a message across all of its translations, returning
/// a full set of diagnostics with information about each one.
///
/// Only normal messages can be validated, since validation implies some source
/// of truth (a definition) to check against. Undefined messages can still have
/// diagnostics presented from general errors, like invalid syntax or
/// unsupported syntax.
pub fn validate_message(message: &Message) -> Vec<MessageDiagnostic> {
    let has_source = message.source_locale().is_some();
    let (source_visible_variables, source_locale) =
        if let Some(source) = message.get_source_translation() {
            (
                source.variables.visible_variable_names(),
                // SAFETY: If the source exists, then the locale for it must also exist.
                Some(message.source_locale().unwrap()),
            )
        } else {
            (Default::default(), None)
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

        let translation_visible_variables = translation.variables.visible_variable_names();

        // If the translation contains visible variables but the source does not, it's likely
        // unintended (the only time this should reasonably happen is when translations are
        // out-of-date, which should be fixed automatically once the translations are imported
        // again). This only applies to "visible" variables because in languages without distinct
        // plural forms (e.g., Japanese), messages that contain plural expressions in English should
        // _not_ have the same plural expressions and instead just render the number if necessary.
        // If the English form does not actually render the variable (i.e., it's invisible), then
        // the translation can generally omit the variable entirely, since it has no effect on the
        // rendered content.
        if has_source
            && translation_visible_variables.len() > 0
            && source_visible_variables.is_empty()
        {
            diagnostics.add(MessageDiagnostic {
                key: message.key(),
                file_position: translation.file_position,
                locale: locale.clone(),
                name: DiagnosticName::NoExtraTranslationVariables,
                category: DiagnosticCategory::Suspicious,
                description: format!(
                    "Translation includes visible variables that the source message does not: {variables}",
                    variables = variable_name_list(&translation_visible_variables)
                ),
                help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into()),
                span: None,
                fixes: vec![],
            });
        }

        if source_visible_variables.len() > 0 && translation_visible_variables.is_empty() {
            diagnostics.add(MessageDiagnostic {
                key: message.key(),
                file_position: translation.file_position,
                locale: locale.clone(),
                name: DiagnosticName::NoMissingSourceVariables,
                category: DiagnosticCategory::Suspicious,
                description: format!(
                    "Source message includes visible variables missing in this translation: {variables}",
                    variables = variable_name_list(&source_visible_variables)
                ),
                help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into()),
                span: None,
                fixes: vec![],
            });
        }
    }

    diagnostics.diagnostics
}
