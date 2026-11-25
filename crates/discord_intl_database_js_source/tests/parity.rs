use discord_intl_database_core::{key_symbol, DiagnosticCode, MessageDefinitionSource};
use discord_intl_database_js_source::JsMessageSource;

fn parity(source: &str) -> discord_intl_database_js_source::ExtractionReport {
    let file = key_symbol("fixture.messages.js");
    let legacy = JsMessageSource.extract_definitions(file, source);
    let report = JsMessageSource.extract_definitions_with_diagnostics(file, source);
    match (legacy, report) {
        (Ok((meta, values)), Ok(report)) => {
            let legacy: Vec<_> = values.collect();
            assert_eq!(meta, report.meta);
            assert_eq!(legacy.len(), report.definitions.len());
            for (old, new) in legacy.iter().zip(&report.definitions) {
                assert_eq!(old.name, new.name);
                assert_eq!(old.raw, new.raw);
                assert_eq!(old.meta, new.meta);
                assert_eq!(
                    serde_json::to_value(&old.value).unwrap(),
                    serde_json::to_value(&new.value).unwrap()
                );
                assert_eq!(
                    old.value.source_offsets.as_slice(),
                    new.value.source_offsets.as_slice()
                );
            }
            report
        }
        (Err(_), Err(error)) => panic!("fixture should parse: {error}"),
        _ => panic!("legacy/report outcome differs"),
    }
}

#[test]
fn reports_match_legacy_for_supported_ignored_and_lossy_forms() {
    for body in [
        "export default defineMessages({A:'text'});",
        "export default defineMessages({A:`static template`});",
        "export default defineMessages({A:{message:'text',description:`ignored`,secret:!0,translate:'false',tags:tagList}});",
        "export default defineMessages({A:'first', A:'second'}, undefined);",
        "export default defineMessages({A:{message:'text',tags:[...'abc']}});",
        "export default defineMessages({A:'text'}); export const meta={secret:true};",
        "const values=defineMessages({A:'text'}); export default values;",
        "export default defineMessages({A:variable,B:`dynamic ${value}`,C:{description:'missing'},D:'kept',...other,[key]:'ignored'});",
        r"export default defineMessages({A:'\uD800','\uDC00':'key',B:{message:'ok',description:'\uD800'}});",
        "const recoverable=0123; export default defineMessages({A:'kept'});",
        "export const meta={secret:true,tags:['shared']}; export default defineMessages({A:{message:'hello {name}',tags:['local']}});",
    ] {
        parity(&format!("import {{defineMessages}} from '@discord/intl'; {body}"));
    }
    parity("import {defineMessages as messages} from '@discord/intl'; export default messages({A:`text`});");
    parity("import {defineMessages,t} from '@discord/intl'; export default defineMessages({A:'ignored'});");
    parity("");
}

#[test]
fn marker_import_and_static_templates_remain_complete() {
    let r=parity("import {defineMessages} from '@discord/intl'; export default defineMessages({A:`static`});");
    assert!(r.complete, "{:?}", r.diagnostics);
}

#[test]
fn dropped_and_lossy_values_are_observed_without_removing_definitions() {
    let r = parity(
        r"import {defineMessages} from '@discord/intl'; export default defineMessages({A:'\uD800',B:{message:'ok',description:`ignored`},C:expression});",
    );
    assert!(!r.complete);
    assert_eq!(r.definitions.len(), 2);
    assert_eq!(r.definitions[0].value.raw, "�");
    for code in [
        DiagnosticCode::InvalidUnicode,
        DiagnosticCode::IgnoredMetadata,
        DiagnosticCode::UnsupportedForm,
    ] {
        assert!(r.diagnostics.iter().any(|d| d.code == code), "{code:?}");
    }
}

#[test]
fn catalog_samples_keep_templates_and_metadata_order() {
    for source in [
        include_str!("fixtures/checkout.messages.js"),
        include_str!("fixtures/checkpoint.messages.js"),
        include_str!("fixtures/design.messages.js"),
    ] {
        parity(source);
    }
}

#[test]
fn fatal_and_recoverable_parse_locations_are_retained() {
    let error = JsMessageSource
        .extract_definitions_with_diagnostics(
            key_symbol("test.js"),
            "const octal=0123;\nexport default {",
        )
        .unwrap_err();
    assert!(error.diagnostics.len() >= 2);
    assert_eq!(error.diagnostics.last().unwrap().line, 2);
}
