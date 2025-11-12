use crate::fix::DiagnosticFix;
use crate::DiagnosticCategory;
use intl_database_core::{FilePosition, KeySymbol, MessageValue, SourceOffsetList};
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u8)]
pub enum DiagnosticName {
    NoAvoidableExactPlurals,
    NoExtraTranslationVariables,
    NoInvalidPluralSelector,
    NoMissingPluralOther,
    NoMissingSourceVariables,
    NoRepeatedPluralNames,
    NoRepeatedPluralOptions,
    NoTrimmableWhitespace,
    NoUnicodeVariableNames,
    NoUnsafeVariableSyntax,
    NoNonExhaustivePlurals,
    NoUnnecessaryPlural,
}

impl Display for DiagnosticName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl DiagnosticName {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiagnosticName::NoAvoidableExactPlurals => "NoAvoidableExactPlurals",
            DiagnosticName::NoExtraTranslationVariables => "NoExtraTranslationVariables",
            DiagnosticName::NoInvalidPluralSelector => "NoInvalidPluralSelector",
            DiagnosticName::NoMissingPluralOther => "NoMissingPluralOther",
            DiagnosticName::NoMissingSourceVariables => "NoMissingSourceVariables",
            DiagnosticName::NoRepeatedPluralNames => "NoRepeatedPluralNames",
            DiagnosticName::NoRepeatedPluralOptions => "NoRepeatedPluralOptions",
            DiagnosticName::NoTrimmableWhitespace => "NoTrimmableWhitespace",
            DiagnosticName::NoUnicodeVariableNames => "NoUnicodeVariableNames",
            DiagnosticName::NoUnsafeVariableSyntax => "NoUnsafeVariableSyntax",
            DiagnosticName::NoNonExhaustivePlurals => "NoNonExhaustivePlurals",
            DiagnosticName::NoUnnecessaryPlural => "NoUnnecessaryPlural",
        }
    }
}

pub type TextRange = (usize, usize);

#[derive(Debug, Clone)]
pub struct MessageDiagnostic {
    pub key: KeySymbol,
    /// Position of the message within the source file
    pub file_position: FilePosition,
    pub locale: KeySymbol,
    pub name: DiagnosticName,
    pub category: DiagnosticCategory,
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
    pub category: DiagnosticCategory,
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

        let converted_diagnostics = value_diagnostics.into_iter().map(|diagnostic| {
            // Ensure fixes have the correct span mapping as well.
            let fixes = diagnostic
                .fixes
                .into_iter()
                .map(|fix| DiagnosticFix {
                    source_span: convert_byte_span_to_character_span(
                        &message.raw,
                        fix.source_span,
                        &message.source_offsets,
                    ),
                    ..fix
                })
                .collect();

            MessageDiagnostic {
                key: self.key,
                file_position: message.file_position,
                locale,
                name: diagnostic.name,
                category: diagnostic.category,
                description: diagnostic.description,
                help: diagnostic.help,
                span: diagnostic.span.map(|span| {
                    convert_byte_span_to_character_span(&message.raw, span, &message.source_offsets)
                }),
                fixes,
            }
        });

        self.diagnostics.extend(converted_diagnostics);
    }
}

/// TODO(faulty): This is very JS-specific and encodes JS's UTF-16 handling of surrogate pairs
/// into the offset calculation. This should really be done on the consuming side instead, but it's
/// massively slower.
fn convert_byte_span_to_character_span(
    source: &str,
    byte_span: (usize, usize),
    source_offsets: &SourceOffsetList,
) -> (usize, usize) {
    assert!(
        byte_span.0 <= byte_span.1,
        "convert_byte_span_to_character_span only accepts ordered spans (first <= second)"
    );
    let mut byte_count = 0usize;
    let mut char_count = 0usize;
    let mut char_span = (0usize, 0usize);
    let mut start_is_set = false;
    let mut end_is_set = false;
    let mut utf16_offset = 0usize;
    for c in source.chars() {
        // NOTE: This assumes byte-alignment in the given span. It should
        // always be `==`, but a manually-constructed span could end up inside
        // a multibyte character.
        if byte_span.0 <= byte_count && !start_is_set {
            char_span.0 = char_count + utf16_offset;
            start_is_set = true;
        }
        // Also assumes that span.0 <= span.1
        if byte_span.1 <= byte_count {
            char_span.1 = char_count + utf16_offset;
            end_is_set = true;
            break;
        }
        byte_count += c.len_utf8();
        char_count += 1;
        // Generally UTF-8 and UTF-16 align on bytes, but some things like
        // emoji take up 2 UTF-16 codepoints, and in JS will be `.length === 2`
        // rather than 1. This offset tracks that additional position to allow
        utf16_offset += c.len_utf16() - 1;
    }
    if !start_is_set {
        char_span.0 = char_count + utf16_offset;
    }
    if !end_is_set {
        char_span.1 = char_count + utf16_offset;
    }

    (
        char_span.0 + source_offsets.total_offset_at(byte_span.0) as usize,
        char_span.1 + source_offsets.total_offset_at(byte_span.1) as usize,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_positions() {
        // ƒ is a 2-byte character, so the _character_ position of `baz` should
        // be offset 3 less than the _byte_ position to align visually.
        let source = "baz";
        let baz_byte_position = (0, 3);
        let expected_char_position = (0, 3);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &Default::default());
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn out_of_range_positions() {
        // Maybe this should panic? idk.
        let source = "baz";
        let baz_byte_position = (0, 5);
        let expected_char_position = (0, 3);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &Default::default());
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn multibyte_character_span_conversion() {
        // ƒ is a 2-byte character, so the _character_ position of `baz` should
        // be offset 3 less than the _byte_ position to align visually.
        let source = "ƒƒƒ baz";
        let baz_byte_position = (7, 10);
        let expected_char_position = (4, 7);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &Default::default());
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn three_byte_span_conversion() {
        // ƒ is a 2-byte character, so the _character_ position of `baz` should
        // be offset 3 less than the _byte_ position to align visually.
        let source = "you’ll be included";
        let baz_byte_position = (7, 10);
        let expected_char_position = (5, 8);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &Default::default());
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn multibyte_character_span_surrounded() {
        let source = "bar ƒƒƒ baz";
        let baz_byte_position = (1, 12);
        let expected_char_position = (1, 9);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &Default::default());
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn escaped_character_span_conversion() {
        let source = "\nbar \n baz";
        let offset_list = SourceOffsetList::new(vec![(0, 1), (5, 2)]);
        let baz_byte_position = (1, 10);
        let expected_char_position = (2, 12);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &offset_list);
        assert_eq!(char_span, expected_char_position);
    }
    #[test]
    fn escaped_character_at_end_span_conversion() {
        let source = "\n\n\n!!{foo}!! !!{user1}!!";
        let offset_list = SourceOffsetList::new(vec![(0, 1), (1, 2), (2, 3)]);
        let baz_byte_position = (13, 24);
        let expected_char_position = (16, 27);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &offset_list);
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn mixed_characters_after_position() {
        let source = "bar baz\n\n";
        let offset_list = SourceOffsetList::new(vec![(7, 1), (8, 2)]);
        let baz_byte_position = (1, 4);
        let expected_char_position = (1, 4);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &offset_list);
        assert_eq!(char_span, expected_char_position);
    }

    #[test]
    fn mixed_character_span_conversion() {
        let source = "\n\nƒƒƒbarbaz";
        let offset_list = SourceOffsetList::new(vec![(0, 1), (1, 2)]);
        let baz_byte_position = (4, 13);
        let expected_char_position = (5, 12);
        let char_span =
            convert_byte_span_to_character_span(source, baz_byte_position, &offset_list);
        assert_eq!(char_span, expected_char_position);
    }
}
