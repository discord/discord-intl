pub use database::MessagesDatabase;
pub use error::MessagesError;
pub use message_definition::{FilePosition, Message, MessageKey};
pub use message_variables_visitor::{
    MessageVariableInstance, MessageVariables, MessageVariablesVisitor, MessageVariableType,
};
pub use meta::MessageMeta;
pub use source_file::SourceFile;
pub use symbols::{
    global_get_symbol, global_intern_string, global_symbol_store, KeySymbol,
    KeySymbolMap, read_global_symbol_store,
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
pub type LocaleId = String;
