use discord_intl_database_core::{
    key_symbol, KeySymbol, MessageDefinitionSource, MessageSourceError, MessageSourceResult,
    RawMessageDefinition, SourceFileKind, SourceFileMeta, DEFAULT_LOCALE,
};
use swc_common::sync::Lrc;
use swc_common::SourceMap;
use swc_core::ecma::ast::Module;

use crate::extractor::{extract_message_definitions, parse_message_definitions_file};

mod extractor;

pub struct JsMessageSource;

/// Legacy extraction results with observations about dropped or lossy input.
#[derive(Debug)]
pub struct ExtractionReport {
    pub meta: SourceFileMeta,
    pub definitions: Vec<RawMessageDefinition>,
    pub diagnostics: Vec<discord_intl_database_core::SourceDiagnostic>,
    pub complete: bool,
}

impl JsMessageSource {
    pub fn extract_definitions_with_diagnostics(
        self,
        file: KeySymbol,
        content: &str,
    ) -> Result<ExtractionReport, discord_intl_database_core::SourceParseError> {
        let (map, module, mut diagnostics) = extractor::parse_with_diagnostics(&file, content)
            .map_err(
                |(diagnostics, _)| discord_intl_database_core::SourceParseError { diagnostics },
            )?;
        let span = swc_common::Spanned::span(&module);
        let extracted = extract_message_definitions(&file, map, module);
        if !extracted.found_definitions {
            diagnostics.push(discord_intl_database_core::SourceDiagnostic::new(
                discord_intl_database_core::DiagnosticCode::UnsupportedForm,
                file,
                content,
                (span.lo.0.saturating_sub(1), span.hi.0.saturating_sub(1)),
                "Legacy extraction found no recognized definition object",
            ));
        }
        diagnostics.extend(extracted.diagnostics.into_inner());
        Ok(ExtractionReport {
            meta: extracted.root_meta,
            definitions: extracted.message_definitions,
            complete: diagnostics.is_empty(),
            diagnostics,
        })
    }
}

impl MessageDefinitionSource for JsMessageSource {
    fn get_default_locale(&self, _file_name: &str) -> KeySymbol {
        key_symbol(DEFAULT_LOCALE)
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
