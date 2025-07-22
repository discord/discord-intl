//! This module contains functions for profiling parser and database performance using IDE tools.
//! In RustRover, for example, the `main` function can be run using the `Profile` tool to get a
//! flamegraph and call trace for the run to identify hotspots and areas for performance
//! improvement. The goal of this file is to be a broad test that flexes most parts of the system
//! at once with realistic data to ensure performance gains are real and not just abstract numbers.

use intl_database_core::{DatabaseInsertStrategy, MessagesDatabase};
use intl_message_database::public::{
    find_all_messages_files, is_message_definitions_file, process_definitions_file,
    process_translation_file,
};

fn find_and_process_files(database: &mut MessagesDatabase) -> anyhow::Result<()> {
    let input_path = std::env::current_dir()?.join("crates/intl_message_database/data/input");
    let input_root = input_path.to_str().expect("Data dir did not exist");
    let files = find_all_messages_files([&input_root].into_iter(), "en-US");
    for file in files {
        if is_message_definitions_file(file.file_path.to_str().unwrap()) {
            process_definitions_file(
                database,
                file.file_path.to_str().unwrap(),
                Some("en-US"),
                DatabaseInsertStrategy::UpdateSourceFile,
            )?;
        } else {
            process_translation_file(
                database,
                file.file_path.to_str().unwrap(),
                &file.locale,
                DatabaseInsertStrategy::UpdateSourceFile,
            )?;
        }
    }

    Ok(())
}

pub fn main() {
    let mut databases = vec![];
    for _ in 0..10 {
        let mut database = MessagesDatabase::new();
        find_and_process_files(&mut database).expect("Failed to process message files");
        databases.push(database);
    }

    println!("Processed {} unique messages", databases[0].messages.len());

    // validate_messages(&database).expect("validated messages");

    // let source = format!("{input_root}/en-US.messages.js");
    // let output = format!("{output_root}/en-US.messages.d.ts");
    // generate_types(&mut database, &source, &output).expect("types should be generated");
}
