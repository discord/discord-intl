pub use database::MessagesDatabase;
pub use error::MessagesError;
pub use message_definition::{FilePosition, Message};
pub use message_variables_visitor::{
    MessageVariableInstance, MessageVariables, MessageVariablesVisitor, MessageVariableType,
};
pub use meta::{MessageMeta, SourceFileMeta};
pub use source_file::{DefinitionFile, SourceFile, TranslationFile};
pub use symbols::{
    global_get_symbol, global_get_symbol_or_error, global_intern_string, KeySymbol, KeySymbolMap,
};
pub use translation::{create_translation_map, Translation, TranslationsMap};
pub use value::MessageValue;

mod database;
mod error;
mod message_definition;
mod message_variables_visitor;
mod meta;
mod source_file;
pub(crate) mod symbols;
mod translation;
mod value;

pub type MessagesResult<T> = Result<T, MessagesError>;
