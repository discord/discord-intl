mod database;
mod error;
mod message_definition;
mod message_variables_visitor;
mod meta;
mod source_file;
mod symbols;
mod translation;
mod value;

pub use database::MessagesDatabase;
pub use error::MessagesError;
pub use message_definition::{FilePosition, Message, MessageKey};
pub use message_variables_visitor::{
    MessageVariableInstance, MessageVariableType, MessageVariables, MessageVariablesVisitor,
};
pub use meta::MessageMeta;
pub use source_file::SourceFile;
pub use symbols::{
    global_get_symbol, global_intern_string, global_symbol_store, read_global_symbol_store,
    KeySymbol, KeySymbolMap,
};
pub use translation::{create_translation_map, Translation, TranslationsMap};
pub use value::MessageValue;

pub type MessagesResult<T> = Result<T, MessagesError>;
pub type LocaleId = String;
