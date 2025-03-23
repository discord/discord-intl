use intl_database_core::{
    key_symbol, FilePosition, KeySymbol, MessageSourceResult, MessageTranslationSource,
    RawMessageTranslation,
};
use intl_flat_json_parser::parse_flat_translation_json;

pub struct JsonMessageSource;

impl MessageTranslationSource for JsonMessageSource {
    fn get_locale_from_file_name(&self, file_name: &str) -> KeySymbol {
        file_name.split('.').next().unwrap_or("en-US").into()
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
