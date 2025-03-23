use thiserror::Error;

use crate::database::symbol::KeySymbol;
use crate::message::source_file::SourceFileKind;
use crate::MessageSourceError;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error(transparent)]
    SourceError(MessageSourceError),
    #[error("Processing {0} yielded neither message definitions nor translations")]
    NoExtractableValues(String),
    #[error("{0} has no matching source implementation")]
    NoSourceImplementation(String),
    #[error("{0} has already been defined in this source file and cannot be defined again")]
    AlreadyDefined(KeySymbol),
    #[error("{0} already has a translation in the locale {1} and cannot be set again")]
    TranslationAlreadySet(KeySymbol, KeySymbol),
    #[error("Ambiguous operation occurred for key {0}: {1}")]
    AmbiguousOperation(KeySymbol, String),

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

impl DatabaseError {
    /// Returns the message key this error applies to, if relevant.
    pub fn key(&self) -> Option<KeySymbol> {
        match self {
            DatabaseError::AlreadyDefined(key) => Some(*key),
            DatabaseError::TranslationAlreadySet(key, _) => Some(*key),
            DatabaseError::AmbiguousOperation(key, _) => Some(*key),
            DatabaseError::SymbolNotFound(key) => Some(*key),
            _ => None,
        }
    }

    /// Returns the locale that this error applies to, if relevant.
    pub fn locale(&self) -> Option<KeySymbol> {
        match self {
            DatabaseError::AlreadyDefined(_) => None,
            DatabaseError::TranslationAlreadySet(_, locale) => Some(*locale),
            _ => None,
        }
    }
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
