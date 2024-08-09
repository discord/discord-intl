use thiserror::Error;

use intl_markdown::compile_to_format_js;

use crate::{
    messages::{KeySymbol, MessagesDatabase},
    services::IntlService,
};
use crate::messages::MessageValue;

#[derive(Debug, Error)]
pub enum IntlMessagePreCompilerError {
    #[error("Source file {0} does not exist in the messages database")]
    SourceFileNotFound(KeySymbol),
    #[error("Message {0} does not exist in the messages database")]
    MessageNotFound(KeySymbol),
}

/// A struct for managing the pre-compilation of messages into a JSON format that the FormatJS
/// runtime understands. The output of this service is a complete JSON object representing all
/// known messages for the given locale, and is written as a stream for the caller to store it
/// wherever needed (an output file, in memory during bundling, etc.).
pub struct IntlMessagePreCompiler<'a, W: std::io::Write> {
    database: &'a MessagesDatabase,
    output: &'a mut W,
    source_key: KeySymbol,
    locale_key: KeySymbol,
    format: CompiledMessageFormat,
}

pub enum CompiledMessageFormat {
    Json,
    KeylessJson,
}

impl<'a, W: std::io::Write> IntlMessagePreCompiler<'a, W> {
    pub fn new(
        database: &'a MessagesDatabase,
        output: &'a mut W,
        source_key: KeySymbol,
        locale_key: KeySymbol,
        format: CompiledMessageFormat,
    ) -> Self {
        Self {
            database,
            output,
            source_key,
            locale_key,
            format,
        }
    }
}

impl<W: std::io::Write> IntlService for IntlMessagePreCompiler<'_, W> {
    type Result = anyhow::Result<()>;

    fn run(&mut self) -> Self::Result {
        let message_keys = self
            .database
            .get_source_file(self.source_key)
            .map(|source| source.message_keys())
            .ok_or_else(|| IntlMessagePreCompilerError::SourceFileNotFound(self.source_key))?;

        write!(self.output, "{{")?;
        let mut is_first = true;
        for key in message_keys {
            let message = self
                .database
                .messages
                .get(key)
                .ok_or_else(|| IntlMessagePreCompilerError::MessageNotFound(*key))?;

            if let Some(translation) = message.translations().get(&self.locale_key) {
                if !is_first {
                    write!(self.output, ",")?;
                } else {
                    is_first = false;
                }
                write!(self.output, "\"{}\":", message.hashed_key())?;
                self.serialize_translation(translation)?;
            }
        }
        write!(self.output, "}}")?;
        Ok(())
    }
}

impl<W: std::io::Write> IntlMessagePreCompiler<'_, W> {
    fn serialize_translation(&mut self, translation: &MessageValue) -> anyhow::Result<()> {
        match self.format {
            CompiledMessageFormat::Json => Ok(serde_json::to_writer(
                &mut self.output,
                &translation.parsed,
            )?),
            CompiledMessageFormat::KeylessJson => Ok(keyless_json::to_writer(
                &mut self.output,
                &compile_to_format_js(&translation.parsed),
            )?),
        }
    }
}
