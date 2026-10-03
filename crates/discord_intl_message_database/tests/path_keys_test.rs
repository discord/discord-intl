//! Tests that a source file has one identity in the database no matter how the caller spells its
//! path.
//!
//! The bug these protect against: file paths are interned by exact bytes, so two spellings of one
//! path used to be two source files. On Windows a caller that derived one path from a realpath and
//! another from the working directory got `B:\...` and `b:\...`, and every message in the file
//! collided with itself under `AlreadyDefined`.

use discord_intl_database_core::{DatabaseInsertStrategy, MessagesDatabase};
use discord_intl_message_database::public::{get_source_file, process_definitions_file_content};

fn definitions_source(messages: &str) -> String {
    format!(
        r#"
        import {{defineMessages}} from '{}';

        export default defineMessages({{{messages}}})
        "#,
        discord_intl_message_utils::RUNTIME_PACKAGE_NAME
    )
}

/// Insert and lookup have to normalize identically. Normalizing only the insert side turns the
/// duplicate-key bug into a `ValueNotInterned` error from every lookup instead of fixing it.
#[test]
fn insert_and_lookup_agree_on_spelling() {
    let mut db = MessagesDatabase::new();
    let path = "insert_and_lookup/en-US.messages.js";

    let data = process_definitions_file_content(
        &mut db,
        path,
        &definitions_source("ONLY_MESSAGE: 'hello'"),
        Some("en-US"),
        DatabaseInsertStrategy::NewSourceFile,
    );
    assert!(data.errors.is_empty(), "{:?}", data.errors);

    get_source_file(&db, path).expect("the file the database just processed is not findable");
}

#[cfg(windows)]
#[test]
fn drive_letter_case_does_not_collide() {
    let mut db = MessagesDatabase::new();
    let content = definitions_source("FIRST_MESSAGE: 'hello', SECOND_MESSAGE: 'world'");

    let created = process_definitions_file_content(
        &mut db,
        r"B:\drive_letter_case\en-US.messages.js",
        &content,
        Some("en-US"),
        DatabaseInsertStrategy::NewSourceFile,
    );
    assert!(created.errors.is_empty(), "{:?}", created.errors);

    // The same file, spelled the way `process.cwd()` would give it to us.
    let updated = process_definitions_file_content(
        &mut db,
        r"b:\drive_letter_case\en-US.messages.js",
        &content,
        Some("en-US"),
        DatabaseInsertStrategy::UpdateSourceFile,
    );
    assert!(updated.errors.is_empty(), "{:?}", updated.errors);

    assert_eq!(created.file_key, updated.file_key);
    assert_eq!(db.sources.len(), 1);
    assert_eq!(updated.inserted_keys.len(), 2);
    assert!(updated.removed_keys.is_empty());
}

#[cfg(windows)]
#[test]
fn lookup_tolerates_a_different_spelling() {
    let mut db = MessagesDatabase::new();

    process_definitions_file_content(
        &mut db,
        r"B:\different_spelling\en-US.messages.js",
        &definitions_source("ONLY_MESSAGE: 'hello'"),
        Some("en-US"),
        DatabaseInsertStrategy::NewSourceFile,
    );

    for spelling in [
        r"b:\different_spelling\en-US.messages.js",
        "b:/different_spelling/en-US.messages.js",
        r"\\?\b:\different_spelling\en-US.messages.js",
    ] {
        get_source_file(&db, spelling).unwrap_or_else(|_| panic!("{spelling} is not findable"));
    }
}
