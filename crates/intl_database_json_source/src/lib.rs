use intl_database_core::{
    key_symbol, FilePosition, KeySymbol, MessageSourceResult, MessageTranslationSource,
    RawMessageTranslation,
};
use intl_flat_json_parser::parse_flat_translation_json;
use std::path::Path;

pub struct JsonMessageSource;

impl MessageTranslationSource for JsonMessageSource {
    fn get_locale_from_file_name(&self, file_name: &str) -> KeySymbol {
        Path::new(file_name)
            .file_name()
            .and_then(|p| p.to_str())
            .map(|p| p.split_once(".").map_or(p, |(name, _ext)| name))
            .unwrap_or("en-US")
            .into()
    }

    fn extract_translations(
        self,
        file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<impl Iterator<Item = RawMessageTranslation>> {
        let iter = parse_flat_translation_json(&content);
        Ok(iter.map(move |item| {
            RawMessageTranslation::new(
                key_symbol(&item.key),
                FilePosition::new(file_name, item.position.line, item.position.col),
                item.value,
            )
        }))
    }
}

#[test]
fn test_locale_from_file_name() {
    assert_eq!(
        key_symbol("en-US"),
        JsonMessageSource.get_locale_from_file_name("foo/bar/baz/en-US.messages.jsona")
    );
    assert_eq!(
        key_symbol("fr-FR"),
        JsonMessageSource.get_locale_from_file_name("foo/bar/baz/fr-FR.messages.jsona")
    );
    assert_eq!(
        key_symbol("notareal__locale#1"),
        JsonMessageSource.get_locale_from_file_name("notareal__locale#1.messages.jsona")
    );

    assert_eq!(
        key_symbol("da"),
        JsonMessageSource.get_locale_from_file_name("da.messages.jsona")
    );
    assert_eq!(
        key_symbol("cz"),
        JsonMessageSource.get_locale_from_file_name("foo/bar/cz")
    );
}
