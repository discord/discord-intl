use discord_intl_flat_json_parser::{
    parse_flat_translation_json, JsonDiagnosticKind, TranslationsJsonParser,
};
fn parity(source: &str) {
    let legacy: Vec<_> = parse_flat_translation_json(source).collect();
    let mut parser = TranslationsJsonParser::with_diagnostics(source);
    let report: Vec<_> = parser.by_ref().collect();
    assert_eq!(legacy.len(), report.len(), "{source}");
    for (a, b) in legacy.iter().zip(&report) {
        assert_eq!(a.key, b.key);
        assert_eq!(a.raw, b.raw);
        assert_eq!(a.value, b.value);
        assert_eq!(
            (a.position.line, a.position.col),
            (b.position.line, b.position.col)
        );
    }
}
#[test]
fn checked_mode_matches_all_legacy_interpretation_including_permissive_forms() {
    for source in [
        "{}",
        "{ }",
        r#"{"A":"text",}"#,
        "{/* comment */\n\"😀\":\"value\",\"A\":\"other\",\"A\":\"last\"}",
        r#"{"A":"\q"}"#,
        r#"{"A":"\uFFFF"}"#,
        r#"{"A":"\uD83D\uDE00"}"#,
        r#"{"A":"\uD800xxDC00"}"#,
        r#"{"A":"\uD800\uE000"}"#,
        r#"{"A":"ok", "B":false}"#,
        "prefix {\"A\":\"value\"} trailing",
        "{\"A\":\"line\nline\"}",
        "",
        "{",
        "{\"A\":",
        "{} trailing",
        r#"{"A":"\uDC00"}"#,
    ] {
        parity(source);
    }
}
#[test]
fn diagnostics_observe_decode_errors_and_ignored_trailing_input() {
    let mut p = TranslationsJsonParser::with_diagnostics(r#"{"A":"ok", "B":"\uDC00"}"#);
    assert_eq!(p.by_ref().count(), 1);
    assert!(!p.is_complete());
    assert_eq!(p.diagnostics()[0].kind, JsonDiagnosticKind::InvalidUnicode);
    let mut p = TranslationsJsonParser::with_diagnostics("{} trailing");
    assert_eq!(p.by_ref().count(), 0);
    assert!(!p.is_complete());
    assert_eq!(p.diagnostics()[0].kind, JsonDiagnosticKind::UnsupportedForm);
}
#[test]
fn report_mode_observes_decoder_panics_without_changing_legacy_decoder() {
    for source in [r#"{"A":"\u12"}"#, r#"{"A":"\uD800"}"#] {
        let mut parser = TranslationsJsonParser::with_diagnostics(source);
        assert!(parser.by_ref().next().is_none());
        assert!(!parser.diagnostics().is_empty());
    }
}
