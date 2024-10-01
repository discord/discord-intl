use std::fmt::Formatter;

use serde::{Serialize, Serializer};

use crate::messages::{KeySymbol, Message};

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

pub struct MessageDiagnostic {
    pub key: KeySymbol,
    pub file_key: KeySymbol,
    pub locale: KeySymbol,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
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

    // If the source message couldn't be parsed, then it can't be used as a comparison.
    // TODO: Re-add when parsing diagnostics are added.
    // if let Err(err) = &source.value.parsed {
    //     let diagnostic = Diagnostic::error().with_message("Source message could not be parsed");
    //     return vec![diagnostic
    //         .with_labels(vec![Label::primary(source.file(), source.full_span())])
    //         .with_notes(vec![err.to_string()])];
    // }

    let mut diagnostics = vec![];

    let source_variables = &source.variables;
    let source_has_variables = source_variables
        .as_ref()
        .is_some_and(|variables| variables.count() > 0);

    for (locale, translation) in message.translations() {
        if *locale == source_locale {
            continue;
        }

        // TODO: Re-add when parsing diagnostics are added.
        // if let Err(err) = &translation.value.parsed {
        //     diagnostics.push(
        //         Diagnostic::error()
        //             .with_message("Translation could not be parsed")
        //             .with_labels(vec![Label::primary(
        //                 translation.file(),
        //                 translation.full_span(),
        //             )])
        //             .with_notes(vec![err.to_string()]),
        //     );
        //     continue;
        // }

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
