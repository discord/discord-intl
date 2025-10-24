use crate::harness;
use intl_database_core::key_symbol;
use intl_validator::validators::NoUnsafeVariableSyntax;

#[test]
fn js_newline_offset() {
    let source = harness::json_source_file(
        "test_file.json",
        r#"{"TEST_KEY":"\n\n\n!!{foo}!! !!{user1}!!"}"#,
    );
    let message = source.get(&key_symbol("TEST_KEY")).unwrap();
    println!("{:?}", message.source_offsets);
    println!("{:?}", message.source_offsets);
    let diagnostics =
        harness::validate_with(message, NoUnsafeVariableSyntax::new()).unwrap_or(vec![]);

    println!("{:#?}", diagnostics)
}
