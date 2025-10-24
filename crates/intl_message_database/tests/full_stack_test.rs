use intl_database_core::{
    key_symbol, DatabaseInsertStrategy, FilePosition, MessageValue, MessagesDatabase,
    SourceOffsetList,
};
use intl_validator::validate_message;

#[test]
fn full_stack_test() {
    let mut db = MessagesDatabase::new();
    let value = MessageValue::from_raw(
        r#"!!{username}!!: @everyone \"!!{topic}!!\" beginnt. Komm vorbei!"#,
        FilePosition::new(key_symbol("test-file.messages.js"), 1, 1),
        SourceOffsetList::default(),
    );

    let message = db
        .insert_definition(
            "test",
            value,
            key_symbol("en-US"),
            Default::default(),
            DatabaseInsertStrategy::NewSourceFile,
        )
        .expect("Failed to insert definition");

    println!(
        "{:?}",
        message.get_source_translation().unwrap().source_offsets
    );

    let diagnostics = validate_message(message);
    for diagnostic in diagnostics {
        println!("{:?}\n-----------", &diagnostic,);
    }
}
