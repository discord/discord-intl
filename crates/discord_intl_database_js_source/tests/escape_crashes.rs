use discord_intl_database_core::{key_symbol, MessageDefinitionSource};
use discord_intl_database_js_source::JsMessageSource;
#[test]
fn short_unicode_escapes_no_longer_panic() {
    let source = r"import {defineMessages} from '@discord/intl'; export default defineMessages({A:'\u{41}',B:'\uFFFF',C:'x\u9999'});";
    let (_, definitions) = JsMessageSource
        .extract_definitions(key_symbol("test.js"), source)
        .unwrap();
    let values: Vec<_> = definitions.map(|d| d.value.raw).collect();
    assert_eq!(values, ["A", "\u{FFFF}", "x香"]);
}
#[test]
fn offsets_for_successful_inputs_keep_their_existing_values() {
    let source = r"import {defineMessages} from '@discord/intl'; export default defineMessages({A:'\u0041',B:'\uFFFF padding',C:`\nfoo`});";
    let (_, definitions) = JsMessageSource
        .extract_definitions(key_symbol("test.js"), source)
        .unwrap();
    let offsets: Vec<_> = definitions
        .map(|d| d.value.source_offsets.as_slice().to_vec())
        .collect();
    assert_eq!(offsets, [vec![(0, 5)], vec![(0, 3)], vec![]]);
}
