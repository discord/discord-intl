use crate::DiagnosticSeverity;
use intl_database_core::{FilePosition, KeySymbol, MessageValue};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum DiagnosticName {
    NoExtraTranslationVariables,
    NoMissingSourceVariables,
    NoRepeatedPluralNames,
    NoRepeatedPluralOptions,
    NoTrimmableWhitespace,
    NoUnicodeVariableNames,
}

impl Display for DiagnosticName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DiagnosticName {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticName::NoExtraTranslationVariables => "NoExtraTranslationVariables",
            DiagnosticName::NoMissingSourceVariables => "NoMissingSourceVariables",
            DiagnosticName::NoRepeatedPluralNames => "NoRepeatedPluralNames",
            DiagnosticName::NoRepeatedPluralOptions => "NoRepeatedPluralOptions",
            DiagnosticName::NoTrimmableWhitespace => "NoTrimmableWhitespace",
            DiagnosticName::NoUnicodeVariableNames => "NoUnicodeVariableNames",
        }
    }
}

type TextRange = (usize, usize);

#[derive(Debug, Clone)]
pub struct DiagnosticFix {
    pub message: Option<String>,
    pub source_span: TextRange,
    pub replacement: String,
}

impl DiagnosticFix {
    pub fn remove_text(source_span: TextRange) -> Self {
        DiagnosticFix {
            message: None,
            source_span,
            replacement: "".into(),
        }
    }

    pub fn replace_text(source_span: TextRange, replacement: &str) -> Self {
        DiagnosticFix {
            message: None,
            source_span,
            replacement: replacement.into(),
        }
    }

    pub fn insert_text(start: usize, new_text: &str) -> Self {
        DiagnosticFix {
            message: None,
            source_span: (start, start),
            replacement: new_text.into(),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct MessageDiagnostic {
    pub key: KeySymbol,
    /// Position of the message within the source file
    pub file_position: FilePosition,
    pub locale: KeySymbol,
    pub name: DiagnosticName,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
    /// Position _within the message_ of the diagnostic
    pub span: Option<(usize, usize)>,
    pub fixes: Vec<DiagnosticFix>,
}

#[derive(Debug, Clone)]
pub struct ValueDiagnostic {
    pub name: DiagnosticName,
    pub span: Option<(usize, usize)>,
    pub severity: DiagnosticSeverity,
    pub description: String,
    pub help: Option<String>,
    pub fixes: Vec<DiagnosticFix>,
}

pub struct MessageDiagnosticsBuilder {
    pub diagnostics: Vec<MessageDiagnostic>,
    pub key: KeySymbol,
}

impl MessageDiagnosticsBuilder {
    pub fn new(key: KeySymbol) -> Self {
        Self {
            diagnostics: vec![],
            key,
        }
    }

    pub fn add(&mut self, diagnostic: MessageDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend_from_value_diagnostics(
        &mut self,
        value_diagnostics: Vec<ValueDiagnostic>,
        message: &MessageValue,
        locale: KeySymbol,
    ) {
        // JS linters tend to work on string indices rather than by byte ranges when reporting lint
        // positions. For anything containing Unicode, this ends up causing a misalignment when a
        // character occupies more than one byte, since we're representing all text offsets as byte
        // positions, but the linter will see that as "x visible characters". So, this converts
        // between the two on the assumption that JS will want the character index, not the byte.

        let converted_diagnostics =
            value_diagnostics
                .into_iter()
                .map(|diagnostic| MessageDiagnostic {
                    key: self.key,
                    file_position: message.file_position,
                    locale,
                    name: diagnostic.name,
                    severity: diagnostic.severity,
                    description: diagnostic.description,
                    help: diagnostic.help,
                    span: diagnostic
                        .span
                        .map(|span| convert_byte_span_to_character_span(&message.raw, span)),
                    fixes: diagnostic.fixes,
                });

        self.diagnostics.extend(converted_diagnostics);
    }
}

fn convert_byte_span_to_character_span(source: &str, byte_span: (usize, usize)) -> (usize, usize) {
    assert!(
        byte_span.0 <= byte_span.1,
        "convert_byte_span_to_character_span only accepts ordered spans (first <= second)"
    );
    let mut byte_count = 0usize;
    let mut char_count = 0usize;
    let mut char_span = (0usize, 0usize);
    for c in source.chars() {
        // NOTE: This assumes byte-alignment in the given span. It should
        // always be `==`, but a manually-constructed span could end up inside
        // a multibyte character.
        if byte_span.0 <= byte_count {
            char_span.0 = char_count;
        }
        // Also assumes that span.0 <= span.1
        if byte_span.1 <= byte_count {
            char_span.1 = char_count;
            break;
        }
        byte_count += c.len_utf8();
        char_count += 1;
    }
    char_span
}
