use discord_intl_message_database::public::extract_file_content_with_diagnostics;
#[test]
fn report_exports_legacy_values_even_when_metadata_is_ignored() {
    let report=extract_file_content_with_diagnostics("virtual.js", "import {defineMessages} from '@discord/intl'; export default defineMessages({A:{message:`text`,description:`ignored`,secret:!0}}); export const meta={secret:true};");
    assert!(!report.complete);
    assert!(report.meta.unwrap().secret);
    assert_eq!(report.messages.len(), 1);
    assert_eq!(report.messages[0].text, "text");
    let meta = report.messages[0].meta.as_ref().unwrap();
    assert!(!meta.secret);
    assert!(meta.description.is_none());
    assert!(!report.diagnostics.is_empty());
}
#[test]
fn marker_imports_and_templates_remain_accepted() {
    let report = extract_file_content_with_diagnostics(
        "virtual.js",
        "import {defineMessages} from '@discord/intl'; export default defineMessages({A:`text`});",
    );
    assert!(report.complete, "{:?}", report.diagnostics);
    assert_eq!(report.messages[0].text, "text");
    let report = extract_file_content_with_diagnostics("virtual.js", "\nexport default {");
    assert!(!report.complete);
    assert_eq!(report.diagnostics[0].line, 2);
}
#[test]
fn json_reports_keep_legacy_escape_interpretation_and_offsets() {
    let report = extract_file_content_with_diagnostics("en.json", r#"{"A":"\q", "B":"\uFFFF"}"#);
    assert!(report.complete, "{:?}", report.diagnostics);
    assert_eq!(report.messages[0].text, "q");
    assert_eq!(report.messages[1].source_offsets, &[(0, 4)]);
}
