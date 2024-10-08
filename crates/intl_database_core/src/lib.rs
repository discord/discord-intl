pub use database::message::Message;
pub use database::MessagesDatabase;
pub use database::source::{
    MessageDefinitionSource, MessageSourceError, MessageSourceResult, MessageTranslationSource,
    RawMessage, RawMessageDefinition, RawMessageTranslation,
};
pub use database::symbol::{get_key_symbol, key_symbol, KeySymbol, KeySymbolMap, KeySymbolSet};
pub use error::{DatabaseError, DatabaseResult};
pub use message::meta::{MessageMeta, SourceFileMeta};
pub use message::source_file::{
    DefinitionFile, FilePosition, SourceFile, SourceFileKind, TranslationFile,
};
pub use message::value::MessageValue;
pub use message::variables::{
    collect_message_variables, MessageVariableInstance, MessageVariables, MessageVariableType,
};

mod database;
mod error;
mod message;

// TODO: Allow this to be configurable, or determined by source files themselves through `meta`.
pub static DEFAULT_LOCALE: &str = "en-US";
