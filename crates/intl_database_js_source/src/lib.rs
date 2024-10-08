use swc_common::errors::HANDLER;

use intl_database_core::{
    KeySymbol, MessageDefinitionSource, MessageSourceError, MessageSourceResult,
    RawMessageDefinition, SourceFileKind, SourceFileMeta,
};

use crate::extractor::{extract_message_definitions, parse_message_definitions_file};

mod extractor;

pub struct JsMessageSource;

impl MessageDefinitionSource for JsMessageSource {
    fn extract_definitions(
        self,
        file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<(SourceFileMeta, impl Iterator<Item = RawMessageDefinition>)> {
        let module = parse_message_definitions_file(&file_name, content).map_err(|error| {
            let diagnostic = HANDLER.with(|handler| error.into_diagnostic(&handler).message());
            MessageSourceError::ParseError(SourceFileKind::Definition, diagnostic)
        })?;
        let extractor = extract_message_definitions(&file_name, module);
        Ok((
            extractor.root_meta,
            extractor.message_definitions.into_iter(),
        ))
    }
}
