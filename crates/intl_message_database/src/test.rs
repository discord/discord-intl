use std::collections::HashMap;
use std::path::Path;

use crate::napi::IntlMessagesDatabase;

#[test]
pub fn test() {
    let input_root = Path::new("./data/input").canonicalize().expect("success");
    let output_root = Path::new("./data/output").canonicalize().expect("success");
    let definitions_path = input_root.join("en-US.js").to_string_lossy().to_string();
    let mut database = IntlMessagesDatabase::new();

    let locales = vec![
        "bg", "cs", "da", "de", "el", "en-GB", "es-419", "es-ES", "fi", "fr", "hi", "hr", "hu",
        "id", "it", "ja", "ko", "lt", "nl", "no", "pl", "pt-BR", "ro", "ru", "sv-SE", "th", "tr",
        "uk", "vi", "zh-CN", "zh-TW",
    ];

    let mut source_files = HashMap::new();
    for locale in locales {
        source_files.insert(
            locale,
            [
                input_root
                    .join(format!("{locale}.jsona"))
                    .to_string_lossy()
                    .to_string(),
                output_root
                    .join(format!("{locale}.json"))
                    .to_string_lossy()
                    .to_string(),
            ],
        );
    }

    database
        .process_definitions_file(definitions_path, Some("en-US".into()))
        .expect("failed to process definitions");

    // for (locale, [input, output]) in source_files {
    //     let source_file = database
    //         .process_translation_file(input, locale.into())
    //         .expect("failed to process translation");
    // }

    let source = input_root.join("en-US.js").to_string_lossy().to_string();
    let output = input_root.join("en-US.d.ts").to_string_lossy().to_string();
    database.generate_types(source, output, None);
}
