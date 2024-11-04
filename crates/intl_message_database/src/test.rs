use crate::public::{find_all_messages_files, generate_types, process_all_messages_files};
use intl_database_core::MessagesDatabase;

#[test]
pub fn test() {
    let input_root = "./data/input";
    let output_root = "./data/output";
    let mut database = MessagesDatabase::new();

    let files = find_all_messages_files([&input_root].into_iter(), "en-US");
    process_all_messages_files(&mut database, files.into_iter()).expect("all files are processed");

    let source = format!("{input_root}/en-US.messages.js");
    let output = format!("{output_root}/en-US.messages.d.ts");
    generate_types(&mut database, &source, &output, None).expect("types should be generated");
}
