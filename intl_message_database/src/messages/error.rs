use crate::messages::source_file::SourceFileKind;
use thiserror::Error;

use super::symbols::KeySymbol;

#[derive(Debug, Error)]
pub enum MessagesError {
    // Definition file errors
    #[error("Parsing error from SWC: {0:?}")]
    DefinitionParseError(swc_core::ecma::parser::error::Error),
    #[error("Message definition semantics were violated: {0}")]
    DefinitionRestrictionViolated(String),
    #[error("Definitions file did not contain any actual message definitions.")]
    NoDefinitionsFound,
    #[error("A meta object was defined in this file, but was not given an initializer")]
    NoMetaInitializer,
    #[error("Message definition for {0} did not contain a default message value")]
    NoMessageValue(String),
    #[error("{0} has already been defined in this source file and cannot be defined again")]
    AlreadyDefined(KeySymbol),
    #[error("{0} already has a translation in the locale {1} and cannot be set again")]
    TranslationAlreadySet(String, KeySymbol),

    // Translation file errors
    #[error(transparent)]
    TranslationDeserializationError(#[from] serde_json::Error),

    // Database errors
    #[error("Expected source file {file_name} to be a {expected} but found {found}")]
    MismatchedSourceFileKind {
        file_name: String,
        expected: SourceFileKind,
        found: SourceFileKind,
    },
    #[error("Global symbol store was poisoned and could not be read")]
    SymbolStorePoisonedError,
    #[error("Symbol {0:?} was not found in the symbol store")]
    SymbolNotFound(KeySymbol),
    #[error("Tried to look up symbol for the given value, but it has not yet been interned. The value was: {0}")]
    ValueNotInterned(String),
    #[error("Source file {0} is not a known source file in the database")]
    UnknownSourceFile(KeySymbol),
}
