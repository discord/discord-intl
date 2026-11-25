use discord_intl_database_core::{key_symbol, MessageTranslationSource};
use discord_intl_database_json_source::JsonMessageSource;
#[test]
fn translation_reports_preserve_values_positions_and_offsets() {
    for source in [
        r#"{"A":"\uFFFF", "B":"\uD83D\uDE00"}"#,
        r#"{"A":"\q", "A":"duplicate"}"#,
        "{/* comment */\n\"😀\":\"value\"}",
        r#"{"A":"kept", "B":"\uDC00"}"#,
    ] {
        let file = key_symbol("en.json");
        let legacy: Vec<_> = JsonMessageSource
            .extract_translations(file, source)
            .unwrap()
            .collect();
        let report = JsonMessageSource.extract_translations_with_diagnostics(file, source);
        assert_eq!(legacy.len(), report.messages.len());
        for (a, b) in legacy.iter().zip(report.messages) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.value.raw, b.value.raw);
            assert_eq!(a.value.file_position, b.value.file_position);
            assert_eq!(
                a.value.source_offsets.as_slice(),
                b.value.source_offsets.as_slice()
            );
        }
    }
}
