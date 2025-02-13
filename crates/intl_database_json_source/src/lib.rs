use intl_database_core::{
    key_symbol, KeySymbol, MessageSourceResult, MessageTranslationSource, RawMessageTranslation,
    RawPosition,
};
use intl_flat_json_parser::parse_flat_translation_json;

pub struct JsonMessageSource;

impl MessageTranslationSource for JsonMessageSource {
    fn get_locale_from_file_name(&self, file_name: &str) -> KeySymbol {
        file_name.split('.').next().unwrap_or("en-US").into()
    }

    fn extract_translations(
        self,
        _file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<impl Iterator<Item = RawMessageTranslation>> {
        let iter = parse_flat_translation_json(&content);
        Ok(iter.map(|item| {
            RawMessageTranslation::new(
                key_symbol(&item.key),
                RawPosition::new(item.position.line, item.position.col),
                item.value,
            )
        }))
    }
}
