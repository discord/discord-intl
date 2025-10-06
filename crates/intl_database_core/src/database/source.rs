use thiserror::Error;

use crate::{FilePosition, KeySymbol, MessageMeta, MessageValue, SourceFileKind, SourceFileMeta};

#[derive(Debug, Error, PartialEq)]
pub enum MessageSourceError {
    #[error("Failed to parse message {0} source: {1}")]
    ParseError(SourceFileKind, String),
    #[error("Semantic restriction for definitions was violated: {0}")]
    DefinitionRestrictionViolated(String),
    #[error("Semantic restriction for translations was violated: {0}")]
    TranslationRestrictionViolated(String),
    #[error("Message {0} did not contain a message value")]
    NoMessageValue(KeySymbol),
    #[error("Source file meta descriptor is invalid")]
    InvalidSourceFileMeta,
    #[error("Message meta descriptor for message {0} is invalid")]
    InvalidMessageMeta(KeySymbol),
    #[error("Expected to encounter at least 1 definition in the source file, but none were found")]
    NoMessagesFound,
}

pub type MessageSourceResult<T> = Result<T, MessageSourceError>;
/// List of offsets to add to any ranges representing differences between escaped and "processed"
/// text. For example a JS string may encode newlines into a message as escaped characters:
///     MESSAGE_ONE: "Foo\n\nBar"
/// When reading from the original text, a source is responsible for processing these escaped
/// characters to ensure consistent data gets passed into the Markdown parser (rather than forcing
/// the parser to understand different escape syntaxes from potentially multiple programming
/// languages), but this means any positions that the parser references would not perfectly line up
/// with the original source text because a literal newline character is 1 character, but the
/// encoded "\n" is 2 characters in the source text.
///
/// This offset list allows any position to be mapped to a true source position by adding the
/// offset at the nearest lower "index" value (the first of the pair for each element). This is
/// unfortunately not constant-time, but since lookups only need to be done in a relatively small
/// number of cases and are not part of the initial processing path, this cost is acceptable.
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
pub struct SourceOffsetList(Vec<(u32, u32)>);

impl SourceOffsetList {
    pub fn new(list: Vec<(u32, u32)>) -> Self {
        Self(list)
    }

    /// Return the true source position that `pos` maps to based on this
    /// offset list for a source string.
    pub fn adjust_byte_position(&self, pos: u32) -> u32 {
        for (bound, offset) in self.0.iter().rev() {
            if *bound <= pos {
                return pos + offset;
            }
        }
        pos
    }
}

pub trait RawMessage {
    fn name(&self) -> KeySymbol;
    fn position(&self) -> &FilePosition;
    fn take_value(self) -> MessageValue;
}

#[derive(Debug)]
pub struct RawMessageDefinition {
    pub name: KeySymbol,
    pub value: MessageValue,
    pub raw: String,
    pub meta: MessageMeta,
}

impl RawMessageDefinition {
    pub fn new<V: AsRef<str>>(
        name: KeySymbol,
        position: FilePosition,
        value: V,
        raw: &str,
        meta: MessageMeta,
        source_offsets: SourceOffsetList,
    ) -> Self {
        let value = MessageValue::from_raw(value.as_ref(), position, source_offsets);
        Self {
            name,
            value,
            raw: raw.into(),
            meta,
        }
    }
}

impl RawMessage for RawMessageDefinition {
    fn name(&self) -> KeySymbol {
        self.name
    }

    fn position(&self) -> &FilePosition {
        &self.value.file_position
    }

    fn take_value(self) -> MessageValue {
        self.value
    }
}

#[derive(Debug)]
pub struct RawMessageTranslation {
    pub name: KeySymbol,
    pub value: MessageValue,
}

impl RawMessageTranslation {
    pub fn new<V: AsRef<str>>(
        name: KeySymbol,
        position: FilePosition,
        value: V,
        source_offsets: SourceOffsetList,
    ) -> Self {
        let value = MessageValue::from_raw(value.as_ref(), position, source_offsets);
        Self { name, value }
    }
}

impl RawMessage for RawMessageTranslation {
    fn name(&self) -> KeySymbol {
        self.name
    }
    fn position(&self) -> &FilePosition {
        &self.value.file_position
    }

    fn take_value(self) -> MessageValue {
        self.value
    }
}

pub trait MessageDefinitionSource {
    /// Return the default locale for which definitions in the given `file_name` should be applied.
    fn get_default_locale(&self, file_name: &str) -> KeySymbol;

    /// Return an [`Iterator`] over all the message definitions contained in the source file.
    /// Any kind of iterator is valid, so long as it yields complete [`RawMessageDefinition`]
    /// structs for the database to handle inserting and updating as needed.
    fn extract_definitions(
        self,
        file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<(
        SourceFileMeta,
        impl Iterator<Item = RawMessageDefinition> + '_,
    )>;
}

pub trait MessageTranslationSource {
    /// Return the locale for which translations within the given `file_name` should be applied.
    fn get_locale_from_file_name(&self, file_name: &str) -> KeySymbol;

    /// Return an [`Iterator`] over all of the message translations contained in the source file.
    /// Any kind of iterator is valid, so long as it yields complete [`RawMessageTranslation`]
    /// structs for the database to handle inserting and updating as needed.
    fn extract_translations(
        self,
        file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<impl Iterator<Item = RawMessageTranslation> + '_>;
}
