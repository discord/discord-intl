use intl_database_core::{
    key_symbol, KeySymbol, MessageDefinitionSource, MessageSourceError, MessageSourceResult,
    RawMessageDefinition, SourceFileKind, SourceFileMeta,
};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_core::ecma::ast::Module;

use crate::extractor::{extract_message_definitions, parse_message_definitions_file};

mod extractor;

pub struct JsMessageSource;

impl MessageDefinitionSource for JsMessageSource {
    fn get_default_locale(&self, _file_name: &str) -> KeySymbol {
        key_symbol("en-US")
    }

    fn extract_definitions(
        self,
        file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<(SourceFileMeta, impl Iterator<Item = RawMessageDefinition>)> {
        let (source, module) = parse_definitions_with_error_handling(file_name, content)?;
        let extractor = extract_message_definitions(&file_name, source, module);
        Ok((
            extractor.root_meta,
            extractor.message_definitions.into_iter(),
        ))
    }
}

fn parse_definitions_with_error_handling(
    file_name: KeySymbol,
    content: &str,
) -> MessageSourceResult<(Lrc<SourceMap>, Module)> {
    parse_message_definitions_file(&file_name, content).map_err(|error| {
        let kind = error.into_kind();
        let message = kind.msg();
        MessageSourceError::ParseError(SourceFileKind::Definition, message.to_string())
    })
}
