use intl_database_core::{
    key_symbol, DatabaseInsertStrategy, FilePosition, MessageValue, MessagesDatabase,
    SourceOffsetList,
};

#[test]
fn full_stack_test() {
    let mut db = MessagesDatabase::new();
    let value = MessageValue::from_raw(
        "Some $[settings](openSettingsHook) are managed by your guardian. Check in with them to make changes.",
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

    println!("{:#?}", message.source_variables());
}
