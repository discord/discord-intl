use std::cell::RefCell;
use std::fmt::Formatter;
use std::rc::Rc;

use serde::{Serialize, Serializer};

use intl_database_core::Message;
use intl_markdown::visitor::visit_with_mut;

use crate::message_diagnostic::DiagnosticBuilder;
pub use crate::message_diagnostic::MessageDiagnostic;
use crate::validators::ValidationVisitor;

mod message_diagnostic;
mod validators;

pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

impl DiagnosticSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl Serialize for DiagnosticSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_str())
    }
}

impl std::fmt::Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

fn validate_message_content(diagnostics: &mut Vec<MessageDiagnostic>, message: &Message) {
    for (locale, translation) in message.translations() {
        let builder = DiagnosticBuilder::new(
            message.key(),
            translation.file_position.unwrap().file,
            *locale,
        );
        let builder_ref = {
            let builder = Rc::new(RefCell::new(builder));
            let mut visitor = ValidationVisitor::new(vec![
                Box::new(validators::NoUnicodeVariableNames::new(Rc::clone(&builder))),
                Box::new(validators::NoRepeatedPluralNames::new(Rc::clone(&builder))),
                Box::new(validators::NoRepeatedPluralNames::new(Rc::clone(&builder))),
            ]);
            visit_with_mut(&mut visitor, &translation.parsed);
            builder
        };
        diagnostics.extend(builder_ref.take().diagnostics)
    }
}

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
    let mut diagnostics = vec![];

    validate_message_content(&mut diagnostics, message);

    let source_variables = &source.variables;
    let source_has_variables = source_variables
        .as_ref()
        .is_some_and(|variables| variables.count() > 0);

    for (locale, translation) in message.translations() {
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
                diagnostics.push(MessageDiagnostic {
                        key: message.key(),
                        file_key: translation.file_position.unwrap().file,
                        locale: locale.clone(),
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
                    diagnostics.push(MessageDiagnostic {
                        key: message.key(),
                        file_key: translation.file_position.unwrap().file,
                        locale: locale.clone(),
                        severity: DiagnosticSeverity::Warning,
                        description: "Source message includes variables, but this translation has none.".into(),
                        help: Some("This is okay, but likely unintentional. Check that the source message is defined as expected.".into())
                    });
                }

                continue;
            }
        };
    }

    diagnostics
}
