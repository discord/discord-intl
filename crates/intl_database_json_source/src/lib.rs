use intl_database_core::{
    KeySymbol, MessageSourceResult, MessageTranslationSource,
    RawMessageTranslation,
};

mod parser;

pub struct JsonMessageSource;

impl MessageTranslationSource for JsonMessageSource {
    fn get_locale_from_file_name(&self, file_name: &str) -> KeySymbol {
        file_name.split('.').next().unwrap_or("en-US").into()
    }

    fn extract_translations(
        self,
        _file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<impl Iterator<Item=RawMessageTranslation>> {
        Ok(parser::parse_flat_translation_json(&content))
    }
}
