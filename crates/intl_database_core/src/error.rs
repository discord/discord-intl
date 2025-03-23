use thiserror::Error;

use crate::database::symbol::KeySymbol;
use crate::message::source_file::SourceFileKind;
use crate::{MessageSourceError, MessageValue};

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error(transparent)]
    SourceError(MessageSourceError),
    #[error("Processing {0} yielded neither message definitions nor translations")]
    NoExtractableValues(String),
    #[error("{0} has no matching source implementation")]
    NoSourceImplementation(String),
    #[error("{name} has already been defined at {} and cannot be defined again", .existing.file_position)]
    AlreadyDefined {
        name: KeySymbol,
        existing: MessageValue,
        replacement: MessageValue,
    },
    #[error("{name} already has a translation from {} and cannot be set again", .existing.file_position)]
    TranslationAlreadySet {
        name: KeySymbol,
        locale: KeySymbol,
        existing: MessageValue,
        replacement: MessageValue,
    },
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
    /// Returns the type name of this error
    pub fn name(&self) -> String {
        match self {
            DatabaseError::AlreadyDefined { .. } => "AlreadyDefined".to_string(),
            DatabaseError::TranslationAlreadySet { .. } => "TranslationAlreadySet".to_string(),
            DatabaseError::SourceError(_) => "SourceError".to_string(),
            DatabaseError::NoSourceImplementation(_) => "NoSourceImplementation".to_string(),
            DatabaseError::ValueNotInterned(_) => "ValueNotInterned".to_string(),
            DatabaseError::UnknownSourceFile(_) => "UnknownSourceFile".to_string(),
            DatabaseError::NoExtractableValues(_) => "NoExtractableValues".to_string(),
            DatabaseError::AmbiguousOperation(_, _) => "AmbiguousOperation".to_string(),
            DatabaseError::MismatchedSourceFileKind { .. } => {
                "MismatchedSourceFileKind".to_string()
            }
            DatabaseError::SymbolStorePoisonedError => "SymbolStorePoisonedError".to_string(),
            DatabaseError::SymbolNotFound(_) => "SymbolNotFound".to_string(),
        }
    }

    /// Returns the message key this error applies to, if relevant.
    pub fn key(&self) -> Option<KeySymbol> {
        match self {
            DatabaseError::AlreadyDefined { name, .. } => Some(*name),
            DatabaseError::TranslationAlreadySet { name, .. } => Some(*name),
            DatabaseError::AmbiguousOperation(key, _) => Some(*key),
            DatabaseError::SymbolNotFound(key) => Some(*key),
            _ => None,
        }
    }

    /// Returns the locale that this error applies to, if relevant.
    pub fn locale(&self) -> Option<KeySymbol> {
        match self {
            DatabaseError::AlreadyDefined { .. } => None,
            DatabaseError::TranslationAlreadySet { locale, .. } => Some(*locale),
            _ => None,
        }
    }

    pub fn existing_message(&self) -> Option<&MessageValue> {
        match self {
            DatabaseError::AlreadyDefined { existing, .. } => Some(existing),
            DatabaseError::TranslationAlreadySet { existing, .. } => Some(existing),
            _ => None,
        }
    }

    pub fn replacement_message(&self) -> Option<&MessageValue> {
        match self {
            DatabaseError::AlreadyDefined { replacement, .. } => Some(replacement),
            DatabaseError::TranslationAlreadySet { replacement, .. } => Some(replacement),
            _ => None,
        }
    }

    pub fn file(&self) -> Option<String> {
        match self {
            DatabaseError::AlreadyDefined { replacement, .. }
            | DatabaseError::TranslationAlreadySet { replacement, .. } => {
                Some(replacement.file_position.file.to_string())
            }
            DatabaseError::MismatchedSourceFileKind { file_name, .. } => Some(file_name.clone()),
            DatabaseError::NoSourceImplementation(file_name) => Some(file_name.clone()),
            DatabaseError::NoExtractableValues(file_name) => Some(file_name.clone()),
            DatabaseError::UnknownSourceFile(file_name) => Some(file_name.to_string()),
            _ => None,
        }
    }

    pub fn line(&self) -> Option<u32> {
        match self {
            DatabaseError::AlreadyDefined { replacement, .. }
            | DatabaseError::TranslationAlreadySet { replacement, .. } => {
                Some(replacement.file_position.line)
            }
            _ => None,
        }
    }

    pub fn col(&self) -> Option<u32> {
        match self {
            DatabaseError::AlreadyDefined { replacement, .. }
            | DatabaseError::TranslationAlreadySet { replacement, .. } => {
                Some(replacement.file_position.col)
            }
            _ => None,
        }
    }
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;
