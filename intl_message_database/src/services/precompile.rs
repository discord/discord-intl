use intl_markdown::compile_to_format_js;

use crate::{
    messages::{KeySymbol, MessagesDatabase},
    services::IntlService,
};

/// A struct for managing the pre-compilation of messages into a JSON format that the FormatJS
/// runtime understands. The output of this service is a complete JSON object representing all
/// known messages for the given locale, and is written as a stream for the caller to store it
/// wherever needed (an output file, in memory during bundling, etc.).
pub struct IntlMessagePreCompiler<'a, W: std::io::Write> {
    database: &'a MessagesDatabase,
    output: &'a mut W,
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
        locale_key: KeySymbol,
        format: CompiledMessageFormat,
    ) -> Self {
        Self {
            database,
            output,
            locale_key,
            format,
        }
    }
}
impl<W: std::io::Write> IntlService for IntlMessagePreCompiler<'_, W> {
    type Result = anyhow::Result<()>;

    fn run(&mut self) -> Self::Result {
        match self.format {
            CompiledMessageFormat::Json => self.run_json(),
            CompiledMessageFormat::KeylessJson => self.run_keyless_json(),
        }
    }
}

impl<W: std::io::Write> IntlMessagePreCompiler<'_, W> {
    fn run_json(&mut self) -> anyhow::Result<()> {
        let messages = self.database.messages.values();
        write!(self.output, "{{")?;
        let mut is_first = true;
        for message in messages {
            if let Some(translation) = message.translations().get(&self.locale_key) {
                if !is_first {
                    write!(self.output, ",")?;
                } else {
                    is_first = false;
                }
                write!(self.output, "\"{}\":", message.hashed_key())?;
                serde_json::to_writer(&mut self.output, &translation.parsed)?;
            }
        }
        write!(self.output, "}}")?;
        Ok(())
    }

    fn run_keyless_json(&mut self) -> anyhow::Result<()> {
        let messages = self.database.messages.values();
        write!(self.output, "{{")?;
        let mut is_first = true;
        for message in messages {
            if let Some(translation) = message.translations().get(&self.locale_key) {
                if !is_first {
                    write!(self.output, ",")?;
                } else {
                    is_first = false;
                }
                write!(self.output, "\"{}\":", message.hashed_key())?;
                keyless_json::to_writer(
                    &mut self.output,
                    &compile_to_format_js(&translation.parsed),
                )?;
            }
        }
        write!(self.output, "}}")?;
        Ok(())
    }
}
