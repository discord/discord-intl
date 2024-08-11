use serde::{Deserialize, Serialize};

/// Meta information about how a message should be handled and processed.
/// Meta can be specified at a set level and individually per-message.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageMeta {
    /// Whether the message should be considered private and not suitable for
    /// inclusion in production builds. Along with `bundle_secrets`, this will
    /// control how the messages are bundled. `secret` messages also have
    /// additional rules and guardrails applied to them to help ensure secrecy
    /// while letting them be used freely in development and getting
    /// translations prepared for synchronized launches.
    pub secret: bool,
    /// Whether the message is suitable to be sent for translation, and whether
    /// existing translations should be included when building projects that
    /// include this message. When `false`, the default message value will be
    /// used in all locales, no matter if there is a translation present.
    pub translate: bool,
    /// Whether messages marked as `secret` should have their content preserved
    /// in builds that include the message. When `false`, secret messages will
    /// be fully obfuscated in built output. Setting to `true` allows for
    /// testing builds with secret content as they will appear once the message
    /// becomes non-secret.
    #[serde(rename = "bundleSecrets")]
    pub bundle_secrets: bool,
    /// A full path to a directory where translations for the message should be found.
    #[serde(rename = "translationsPath")]
    pub translations_path: String,
}

impl Default for MessageMeta {
    fn default() -> Self {
        Self {
            secret: false,
            translate: true,
            bundle_secrets: false,
            translations_path: "./messages".into(),
        }
    }
}
