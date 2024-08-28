use std::fmt::Formatter;

use serde::Serialize;

use crate::messages::symbols::KeySymbolSet;

use super::{LocaleId, MessageMeta};

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
        message_keys: KeySymbolSet,
    },
    #[serde(rename = "translation")]
    Translation {
        file: String,
        locale: LocaleId,
        #[serde(rename = "messageKeys")]
        message_keys: KeySymbolSet,
    },
}

impl SourceFile {
    pub fn file(&self) -> &String {
        match self {
            SourceFile::Definition { file, .. } => file,
            SourceFile::Translation { file, .. } => file,
        }
    }

    pub fn message_keys(&self) -> &KeySymbolSet {
        match self {
            SourceFile::Definition { message_keys, .. } => message_keys,
            SourceFile::Translation { message_keys, .. } => message_keys,
        }
    }

    #[inline(always)]
    pub fn message_keys_mut(&mut self) -> &mut KeySymbolSet {
        match self {
            SourceFile::Definition { message_keys, .. } => message_keys,
            SourceFile::Translation { message_keys, .. } => message_keys,
        }
    }

    pub fn set_message_keys(&mut self, new_keys: KeySymbolSet) {
        *self.message_keys_mut() = new_keys;
    }
}
