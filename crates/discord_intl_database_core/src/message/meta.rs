use path_absolutize::Absolutize;
use serde::{Deserialize, Serialize, Serializer};
use std::path::PathBuf;

/// Meta information about how a _set_ of messages should be handled and processed. SourceFileMeta
/// has the same attributes as [MessageMeta], and acts as the source of default values for it, but
/// also provides additional higher-level information like the name of the source file and the path
/// where translations for the messages can be found.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self", rename_all = "camelCase")]
pub struct SourceFileMeta {
    /// Optional additional context for the source file, giving more information about where its
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
    #[serde(getter = "Self::get_translations_path")]
    pub translations_path: PathBuf,
    /// The absolute path to the source file where this meta originated, acting as the base file
    /// for all messages contained in the set.
    pub source_file_path: PathBuf,
    /// Additional user-land information to apply to all messages in the source file. This list has
    /// no effect on the behavior of a message and is purely for other tools to consume.
    /// Tags defined on the source file are inherited by all messages in that file, even if the
    /// message defines new tags on itself.
    pub tags: Option<Vec<String>>,
}

impl Serialize for SourceFileMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Self::serialize(self, serializer)
    }
}

impl SourceFileMeta {
    pub fn new(source_file_path: &str) -> Self {
        Self {
            secret: false,
            translate: true,
            translations_path: "./messages".into(),
            source_file_path: source_file_path.into(),
            description: None,
            tags: None,
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
    pub fn with_tags(mut self, tags: &[String]) -> Self {
        self.tags = Some(tags.to_vec());
        self
    }

    /// Return an absolute, canonicalized path to the directory where translations for messages
    /// defined in this source file should live.
    ///
    /// ## Panic
    /// The `absolutize` method may panic if there is no current working directory in the process,
    /// but this is  exceptionally rare and consequential if true, and working with a Result type
    /// here is not  worth the edge case coverage. Instead, this method asserts that a working
    /// directory is present through `get_cwd!` (the same method used internally by `absolutize`),
    /// ensuring that the process will panic and not return unexpected results.
    pub fn get_translations_path(&self) -> PathBuf {
        assert!(self.source_file_path.is_file() && self.source_file_path.parent().is_some());
        assert!(
            std::env::current_dir().is_ok(),
            "Current Working Directory is not set"
        );
        let source_folder = self.source_file_path.parent().unwrap();
        let path = source_folder
            .join(&self.translations_path)
            .absolutize()
            .unwrap()
            .to_path_buf();

        path
    }

    /// Return an absolute, canonicalized path where translations for messages in this source file
    /// in the given `locale` should reside. If `extension` is given, it will be applied to the
    /// created path, otherwise the path will not have any extension.
    pub fn get_translations_file_path(
        &self,
        locale: &str,
        extension: Option<&str>,
    ) -> std::io::Result<PathBuf> {
        let mut path = self.get_translations_path().join(locale);
        if let Some(extension) = extension {
            path.set_extension(extension);
        }

        Ok(path)
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
    /// Additional user-land information to apply to this message. This list has no effect on the
    /// behavior of the message and is purely for other tools to consume.
    /// Tags defined on the source file are inherited by all messages in that file, even if the
    /// message defines new tags on itself.
    pub tags: Option<Vec<String>>,
}

impl Default for MessageMeta {
    fn default() -> Self {
        Self {
            secret: false,
            translate: true,
            description: None,
            tags: None,
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
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn extend_tags(&mut self, tags: Vec<String>) {
        match &mut self.tags {
            Some(existing_tags) => existing_tags.extend(tags),
            None => self.tags = Some(tags),
        }
    }
}

impl From<&SourceFileMeta> for MessageMeta {
    fn from(value: &SourceFileMeta) -> Self {
        MessageMeta {
            secret: value.secret,
            translate: value.translate,
            description: None,
            tags: value.tags.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_meta_extends_empty_tags() {
        let mut meta = MessageMeta::default().with_tags(vec![]);
        meta.extend_tags(vec!["foo".into()]);
        assert!(meta.tags.is_some_and(|tags| tags.len() == 1));
    }

    #[test]
    fn test_message_meta_extends_existing_tags() {
        let mut meta = MessageMeta::default().with_tags(vec!["foo".into(), "bar".into()]);
        meta.extend_tags(vec!["aaa".into()]);
        assert_eq!(
            vec![
                String::from("foo"),
                String::from("bar"),
                String::from("aaa")
            ],
            meta.tags.unwrap()
        );
    }
    #[test]
    fn test_message_meta_extends_no_tags() {
        let mut meta = MessageMeta::default();
        meta.extend_tags(vec!["foo".into()]);
        assert_eq!(vec![String::from("foo")], meta.tags.unwrap());
    }
}
