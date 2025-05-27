use intl_database_core::{DatabaseInsertStrategy, MessagesDatabase};
use intl_message_database::public::{process_definitions_file, validate_messages};

pub fn main() {
    let input_root = "./data/temp";
    let output_root = "./data/output";
    let mut database = MessagesDatabase::new();
    //
    // let files = find_all_messages_files([&input_root].into_iter(), "en-US");
    // process_all_messages_files(&mut database, files.into_iter()).expect("all files are processed");
    // process_translation_file(&mut database, "./data/temp/es-ES.messages.jsona", "es-ES")
    //     .expect("processed");
    process_definitions_file(
        &mut database,
        "./data/temp/en-US.messages.js",
        None,
        DatabaseInsertStrategy::NewSourceFile,
    )
    .expect("processed");

    validate_messages(&database).expect("validated messages");

    // let source = format!("{input_root}/en-US.messages.js");
    // let output = format!("{output_root}/en-US.messages.d.ts");
    // generate_types(&mut database, &source, &output).expect("types should be generated");
}
