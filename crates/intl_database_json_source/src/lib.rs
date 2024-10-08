use intl_database_core::{
    KeySymbol, MessageSourceError, MessageSourceResult, MessageTranslationSource,
    RawMessageTranslation, SourceFileKind,
};

use crate::deserialize::Translations;

mod deserialize;

pub struct JsonMessageSource;

impl MessageTranslationSource for JsonMessageSource {
    fn extract_translations(
        self,
        _file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<impl Iterator<Item = RawMessageTranslation>> {
        let translations: Translations = serde_json::from_str(content).map_err(|error| {
            MessageSourceError::ParseError(SourceFileKind::Translation, error.to_string())
        })?;
        Ok(translations.into_iter())
    }
}
