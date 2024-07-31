use std::fs::read_to_string;

use crate::messages::{global_get_symbol, MessagesDatabase};
use crate::services;
use crate::services::IntlService;
use crate::services::precompile::CompiledMessageFormat;

#[test]
pub fn test() {
    let translations = read_to_string("./data/input/fr.jsona").unwrap();
    let mut database = MessagesDatabase::new();

    let locales = vec![
        ("bg.json", "bg"),
        ("cs.json", "cs"),
        // ("da.json", "da"),
        // ("de.json", "de"),
        // ("el.json", "el"),
        // ("en-GB.json", "en-GB"),
        // ("es-419.json", "es-419"),
        // ("es-ES.json", "es-ES"),
        // ("fi.json", "fi"),
        ("fr.json", "fr"),
        // ("hi.json", "hi"),
        // ("hr.json", "hr"),
        // ("hu.json", "hu"),
        // ("id.json", "id"),
        // ("it.json", "it"),
        // ("ja.json", "ja"),
        // ("ko.json", "ko"),
        // ("lt.json", "lt"),
        // ("nl.json", "nl"),
        // ("no.json", "no"),
        // ("pl.json", "pl"),
        // ("pt-BR.json", "pt-BR"),
        // ("ro.json", "ro"),
        // ("ru.json", "ru"),
        // ("sv-SE.json", "sv-SE"),
        // ("th.json", "th"),
        // ("tr.json", "tr"),
        // ("uk.json", "uk"),
        // ("vi.json", "vi"),
        // ("zh-CN.json", "zh-CN"),
        // ("zh-TW.json", "zh-TW"),
    ];

    // for (file, locale) in locales {
    //     database
    //         .process_translations_file(file, &locale.into(), &translations)
    //         .expect("failed to process file");
    // }

    let locale_key = global_get_symbol("fr").unwrap();

    let mut buffer = Vec::new();
    for _ in 0..10 {
        buffer.clear();
        services::precompile::IntlMessagePreCompiler::new(
            &database,
            &mut buffer,
            locale_key,
            CompiledMessageFormat::Json,
        )
        .run()
        .ok();
    }
}
