use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Meta information about how a _set_ of messages should be handled and processed. SourceFileMeta
/// has the same attributes as [MessageMeta], and acts as the source of default values for it, but
/// also provides additional higher-level information like the name of the source file and the path
/// where translations for the messages can be found.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFileMeta {
    /// Optional additional context for the source file, giving more information  about where its
    /// messages may be used or how the messages are intended to be grouped.
    pub description: Option<String>,
    /// Whether the message should be considered private and not suitable for
    /// inclusion in production builds. Message consumers can use this
    /// information to control how messages are bundled. `secret` messages also
    /// have  additional rules and guardrails applied to them to help ensure
    /// secrecy while letting them be used freely in development and getting
    /// translations prepared for synchronized launches.
    pub secret: bool,
    /// Whether the message is suitable to be sent for translation, and whether
    /// existing translations should be included when building projects that
    /// include this message. When `false`, the default message value will be
    /// used in all locales, no matter if there is a translation present.
    pub translate: bool,
    /// A (normally relative) path to a directory where translations for the messages in this source
    /// file should be found.
    #[serde(rename = "translationsPath")]
    pub translations_path: PathBuf,
    /// The absolute path to the source file where this meta originated, acting as the base file
    /// for all messages contained in the set.
    #[serde(rename = "sourceFilePath")]
    pub source_file_path: PathBuf,
}

impl SourceFileMeta {
    pub fn new(source_file_path: &str) -> Self {
        Self {
            secret: false,
            translate: true,
            translations_path: "./messages".into(),
            source_file_path: source_file_path.into(),
            description: None,
        }
    }

    pub fn with_secret(mut self, secret: bool) -> Self {
        self.secret = secret;
        self
    }
    pub fn with_translate(mut self, translate: bool) -> Self {
        self.translate = translate;
        self
    }
    pub fn with_translations_path(mut self, translations_path: &str) -> Self {
        self.translations_path = PathBuf::from(translations_path);
        self
    }
    pub fn with_source_file_path(mut self, source_file_path: &str) -> Self {
        self.source_file_path = PathBuf::from(source_file_path);
        self
    }
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(String::from(description));
        self
    }

    /// Return an absolute, canonicalized path where translations for messages in this source file
    /// in the given `locale` should reside. If `extension` is given, it will be applied to the
    /// created path, otherwise the path will not have any extension.
    pub fn get_translations_path(
        &self,
        locale: &str,
        extension: Option<&str>,
    ) -> std::io::Result<PathBuf> {
        assert!(self.source_file_path.is_file() && self.source_file_path.parent().is_some());
        let source_folder = self.source_file_path.parent().unwrap();
        let path = source_folder
            .join(self.translations_path.as_path())
            .join(locale)
            .absolutize()?
            .to_path_buf();

        match extension {
            Some(ext) => Ok(path.with_extension(ext)),
            None => Ok(path),
        }
    }
}

/// Meta information about how a message should be handled and processed. MessageMeta
#[derive(Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageMeta {
    /// Optional additional context for the source file, giving more information about where its
    /// messages may be used or how the messages are intended to be grouped.
    pub description: Option<String>,
    /// Whether the message should be considered private and not suitable for  inclusion in
    /// production builds. Message consumers can use this  information to control how messages are
    /// bundled. `secret` messages also have additional rules and guardrails applied to them to help
    /// ensure secrecy while letting them be used freely in development and getting translations
    /// prepared for synchronized launches.
    pub secret: bool,
    /// Whether the message is suitable to be sent for translation, and whether existing
    /// translations should be included when building projects that include this message. When
    /// `false`, the default message value will be used in all locales, no matter if there is a
    /// translation present.
    pub translate: bool,
}

impl Default for MessageMeta {
    fn default() -> Self {
        Self {
            secret: false,
            translate: true,
            description: None,
        }
    }
}

impl MessageMeta {
    pub fn with_secret(mut self, secret: bool) -> Self {
        self.secret = secret;
        self
    }
    pub fn with_translate(mut self, translate: bool) -> Self {
        self.translate = translate;
        self
    }
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(String::from(description));
        self
    }
}

impl From<&SourceFileMeta> for MessageMeta {
    fn from(value: &SourceFileMeta) -> Self {
        MessageMeta {
            secret: value.secret,
            translate: value.translate,
            description: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_translations_path_returns_canonical_path() {}
}
