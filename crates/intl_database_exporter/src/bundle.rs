use thiserror::Error;

use intl_database_core::{KeySymbol, Message, MessageValue, MessagesDatabase};
use intl_database_service::IntlDatabaseService;
use intl_markdown::{
    compile_to_format_js, raw_string_to_document, BlockNode, Document, InlineContent,
};

#[derive(Debug, Error)]
pub enum IntlMessageBundlerError {
    #[error("Source file {0} does not exist in the messages database")]
    SourceFileNotFound(KeySymbol),
    #[error("Message {0} does not exist in the messages database")]
    MessageNotFound(KeySymbol),
}

pub struct IntlMessageBundlerOptions {
    format: CompiledMessageFormat,
    bundle_secrets: bool,
}

impl IntlMessageBundlerOptions {
    pub fn with_format(mut self, format: CompiledMessageFormat) -> Self {
        self.format = format;
        self
    }
    pub fn with_bundle_secrets(mut self, bundle_secrets: bool) -> Self {
        self.bundle_secrets = bundle_secrets;
        self
    }
}

impl Default for IntlMessageBundlerOptions {
    fn default() -> Self {
        Self {
            format: CompiledMessageFormat::KeylessJson,
            bundle_secrets: false,
        }
    }
}

/// A struct for managing the pre-compilation of messages as a part of bundling to be compatible
/// with the `@discord/intl` runtime. The output of this service is a complete JSON object
/// representing all messages that are _intentionally_ included in bundled output, including
/// filtering out messages marked as secret, not ready for translation, or otherwise meant to be
/// left out of the bundle.
pub struct IntlMessageBundler<'a, W: std::io::Write> {
    database: &'a MessagesDatabase,
    output: &'a mut W,
    source_key: KeySymbol,
    locale_key: KeySymbol,
    options: IntlMessageBundlerOptions,
}

pub enum CompiledMessageFormat {
    Json,
    KeylessJson,
}

impl<'a, W: std::io::Write> IntlMessageBundler<'a, W> {
    pub fn new(
        database: &'a MessagesDatabase,
        output: &'a mut W,
        source_key: KeySymbol,
        locale_key: KeySymbol,
        options: IntlMessageBundlerOptions,
    ) -> Self {
        Self {
            database,
            output,
            source_key,
            locale_key,
            options,
        }
    }

    /// Returns true if the message should be bundled as part of the given locale, according to its
    /// meta information and other general semantics.
    fn should_bundle(&self, message: &Message, locale: KeySymbol) -> bool {
        // Never include messages that aren't defined for the source locale.
        // This catches cases where a message gets deleted from the source, but the translations
        // haven't yet been updated to remove them.
        if !message.is_defined() {
            return false;
        }

        let is_source = message
            .source_locale()
            .is_some_and(|source| source == locale);
        let should_translate = message.meta().translate;
        // If the message is marked as not ready for translation and this _isn't_ the source locale,
        // then don't include it.
        if !is_source && !should_translate {
            return false;
        }

        true
    }

    /// Returns true if the message _value_ should be obfuscated in the generated bundle.
    /// Obfuscated  messages are just given a non-empty placeholder value. Note that this only
    /// applies to the  _value_ of a message because the keys will _always_ be obfuscated as the
    /// hashed version.
    fn should_obfuscate(&self, message: &Message) -> bool {
        // Secret messages are obfuscated by default, but if the caller requests them to be bundled
        // then they are preserved as-is, i.e. for development builds testing out a new feature.
        message.meta().secret && !self.options.bundle_secrets
    }

    fn maybe_serialize_static_document(&mut self, document: &Document) -> anyhow::Result<bool> {
        if document.blocks().len() > 1 {
            return Ok(false);
        }

        let Some(BlockNode::InlineContent(items)) = document.blocks().get(0) else {
            return Ok(false);
        };

        let mut buffer = Vec::with_capacity(items.len() * 20);

        for item in items {
            match item {
                InlineContent::Text(text) => {
                    keyless_json::write_escaped_str_contents(&mut buffer, &text)?
                }
                _ => return Ok(false),
            }
        }

        self.output.write_all(b"\"")?;
        self.output.write_all(&buffer)?;
        self.output.write_all(b"\"")?;
        Ok(true)
    }

    fn serialize_document(&mut self, document: &Document) -> anyhow::Result<()> {
        // Serialize static documents as single strings, both for space savings and faster runtime
        // evaluation.
        if let Ok(true) = self.maybe_serialize_static_document(document) {
            return Ok(());
        }

        // For any other document, just serialize it as-is.
        match self.options.format {
            CompiledMessageFormat::Json => Ok(serde_json::to_writer(&mut self.output, &document)?),
            CompiledMessageFormat::KeylessJson => Ok(keyless_json::to_writer(
                &mut self.output,
                &compile_to_format_js(&document),
            )?),
        }
    }

    /// Serialize the given message using its hashed key as the value, rather than the actual
    /// content of the message, to obfuscate the value irreversibly and prevent leaking secrets.
    fn serialize_value(&mut self, message: &Message, value: &MessageValue) -> anyhow::Result<()> {
        let document = if self.should_obfuscate(message) {
            &raw_string_to_document(message.hashed_key())
        } else {
            &value.parsed
        };
        self.serialize_document(document)
    }
}

impl<W: std::io::Write> IntlDatabaseService for IntlMessageBundler<'_, W> {
    type Result = anyhow::Result<()>;

    fn run(&mut self) -> Self::Result {
        let message_keys = self
            .database
            .get_source_file(self.source_key)
            .map(|source| source.message_keys())
            .ok_or_else(|| IntlMessageBundlerError::SourceFileNotFound(self.source_key))?;

        let mut sorted_message_keys = Vec::with_capacity(message_keys.len());
        message_keys
            .iter()
            .collect_into(&mut sorted_message_keys)
            .sort();

        write!(self.output, "{{")?;
        let mut is_first = true;
        for key in sorted_message_keys {
            let message = self
                .database
                .messages
                .get(key)
                .ok_or_else(|| IntlMessageBundlerError::MessageNotFound(*key))?;

            if !self.should_bundle(message, self.locale_key) {
                continue;
            }

            if let Some(translation) = message.translations().get(&self.locale_key) {
                if !is_first {
                    write!(self.output, ",")?;
                } else {
                    is_first = false;
                }
                write!(self.output, "\"{}\":", message.hashed_key())?;
                self.serialize_value(message, translation)?;
            }
        }
        write!(self.output, "}}")?;
        Ok(())
    }
}
