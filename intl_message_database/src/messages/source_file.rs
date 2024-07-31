use std::fmt::Formatter;

use rustc_hash::FxHashSet;
use serde::Serialize;

use super::{KeySymbol, LocaleId, MessageMeta};

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum SourceFileKind {
    Definition,
    Translation,
}

impl std::fmt::Display for SourceFileKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceFileKind::Definition => f.write_str("Definition"),
            SourceFileKind::Translation => f.write_str("Translation"),
        }
    }
}

/// Representation of a file that either defines or provides translations for
/// some set of messages. The file name is mapped to all of the definitions it
/// affects, along with extra information useful for processing that file.
///
/// SourceFiles allow interactive editing of files to automatically update all
/// of the affected messages safely and efficiently.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum SourceFile {
    #[serde(rename = "definition")]
    Definition {
        file: String,
        meta: MessageMeta,
        #[serde(rename = "messageKeys")]
        message_keys: FxHashSet<KeySymbol>,
    },
    #[serde(rename = "translation")]
    Translation {
        file: String,
        locale: LocaleId,
        #[serde(rename = "messageKeys")]
        message_keys: FxHashSet<KeySymbol>,
    },
}

impl SourceFile {
    pub fn file(&self) -> &String {
        match self {
            SourceFile::Definition { file, .. } => file,
            SourceFile::Translation { file, .. } => file,
        }
    }

    pub fn message_keys(&self) -> &FxHashSet<KeySymbol> {
        match self {
            SourceFile::Definition { message_keys, .. } => message_keys,
            SourceFile::Translation { message_keys, .. } => message_keys,
        }
    }

    pub fn set_message_keys(&mut self, new_keys: FxHashSet<KeySymbol>) {
        match self {
            SourceFile::Definition {
                ref mut message_keys,
                ..
            } => *message_keys = new_keys,
            SourceFile::Translation {
                ref mut message_keys,
                ..
            } => *message_keys = new_keys,
        }
    }
}
