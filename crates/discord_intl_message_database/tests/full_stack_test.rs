use discord_intl_database_core::{
    key_symbol, DatabaseInsertStrategy, FilePosition, MessageValue, MessagesDatabase,
    SourceOffsetList,
};
use discord_intl_message_database::public::process_all_messages_files;
use discord_intl_message_database::sources::MessagesFileDescriptor;
use discord_intl_validator::validate_message;

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

#[test]
fn process_all_messages_files_skips_non_messages_files() {
    let file_path = std::env::temp_dir().join("intl_skip_non_messages_file.zip");
    std::fs::write(&file_path, [0xff, 0xfe, 0x00]).unwrap();

    let mut db = MessagesDatabase::new();
    let files = vec![MessagesFileDescriptor {
        file_path,
        locale: key_symbol("en-US"),
    }];
    let results = process_all_messages_files(
        &mut db,
        files.into_iter(),
        DatabaseInsertStrategy::NewSourceFile,
    )
    .unwrap();
    assert!(results.is_empty());
}
