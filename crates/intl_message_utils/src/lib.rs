use memchr::memmem;
use once_cell::sync::Lazy;

/// Name of the JS runtime package that should be used for all generated code or parsing for imports
/// that read from the package.
pub static RUNTIME_PACKAGE_NAME: &str = "@discord/intl";

/// The seed used when computing hash keys for message names and other hashed identifiers.
///
/// Ensure this hash seed matches the seed used in `intl/hash.ts`.
pub static KEY_HASH_SEED: u64 = 0;

/// Lookup table used for quickly creating a base64 representation of a hashed key.
static BASE64_TABLE: &[u8] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/".as_bytes();

/// Returns a consistent, short hash of the given key by first processing it
/// through a sha256 digest, then encoding the first few bytes to base64.
///
/// Note that while this function is _generally_ the only place responsible for
/// hashing a key, there is a mirrored, client-side hash for use at runtime
/// that _must_ match this identically: `packages/intl/hash.ts`.
pub fn hash_message_key(content: &str) -> String {
    let hash = xxhash_rust::xxh64::xxh64(content.as_bytes(), KEY_HASH_SEED);
    let input: [u8; 8] = hash.to_ne_bytes();
    // Since we know that we only want 6 characters out of the hash, we can
    // shortcut the base64 encoding to just directly read the bits out into an
    // encoded byte array and directly create a str from that.
    let output: Vec<u8> = vec![
        BASE64_TABLE[(input[0] >> 2) as usize],
        BASE64_TABLE[((input[0] & 0x03) << 4 | input[1] >> 4) as usize],
        BASE64_TABLE[((input[1] & 0x0f) << 2 | input[2] >> 6) as usize],
        BASE64_TABLE[(input[2] & 0x3f) as usize],
        BASE64_TABLE[(input[3] >> 2) as usize],
        BASE64_TABLE[((input[3] & 0x03) << 4 | input[4] >> 4) as usize],
    ];

    // SAFETY: We built this string out of ASCII characters, it doesn't need to
    // be checked for utf-8 validity.
    unsafe { String::from_utf8_unchecked(output) }
}

/// Returns true if the given `file_name` is considered a message definitions file. Note that this
/// allows for either _just_ the `.messages` extension (as might be written for an `import`
/// statement), or the entire `.messages.<ext>` for supported languages.
pub fn is_message_definitions_file(file_name: &str) -> bool {
    // `.messages` is the path used when importing, like:
    //     import {messages} from 'Somewhere.messages';
    file_name.ends_with(".messages")
    // All others are used when referencing the actual file path.
        || file_name.ends_with(".messages.tsx")
        || file_name.ends_with(".messages.jsx")
        || file_name.ends_with(".messages.ts")
        || file_name.ends_with(".messages.js")
}

pub fn is_message_translations_file(file_name: &str) -> bool {
    (file_name.ends_with(".messages.json") || file_name.ends_with(".messages.jsona"))
        && !(file_name.ends_with("compiled.messages.json")
            || file_name.ends_with("compiled.messages.jsona"))
}

pub fn is_compiled_messages_artifact(file_name: &str) -> bool {
    file_name.ends_with(".compiled.messages.json")
        || file_name.ends_with(".compiled.messages.jsona")
}

/// A file is a messages file if it is any of a definitions file, translations file, or compiled
/// messages file.
pub fn is_any_messages_file(file_name: &str) -> bool {
    is_message_definitions_file(file_name)
        || is_compiled_messages_artifact(file_name)
        || is_message_translations_file(file_name)
}

static DOUBLE_NEWLINE_FINDER: Lazy<memmem::Finder> = Lazy::new(|| memmem::Finder::new(b"\n\n"));

/// Returns true if the given `message` contains block-like content and should
/// be parsed with blocks included. For now, this requires that the message
/// contains a double newline anywhere inside it.
pub fn message_may_have_blocks(message: &str) -> bool {
    DOUBLE_NEWLINE_FINDER.find(message.as_bytes()).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    // NOTE(faulty): These tests are only relevant so long as discord-intl is supporting
    // JavaScript only. If other source languages end up being supported, like Python,
    // then the definition of a messages file changes and these should be revisited.

    /////
    // Definitions
    /////

    #[test]
    fn test_is_message_definitions_file() {
        assert!(is_message_definitions_file("abc.messages.js"));
        assert!(is_message_definitions_file("en-US.messages.js"));
        assert!(is_message_definitions_file("Feature.messages.js"));
        assert!(is_message_definitions_file("Multiple.prefixes.messages.js"));
        assert!(is_message_definitions_file("messages.messages.js"));
        assert!(is_message_definitions_file("message.messages.js"));
        assert!(is_message_definitions_file("messages.messages.messages.js"));
        assert!(is_message_definitions_file("Foo.messages.messages.js"));
        assert!(is_message_definitions_file("Even with spaces.messages.js"));
        assert!(is_message_definitions_file("foo.js.messages.js"));
        assert!(is_message_definitions_file("js.messages.js"));
        assert!(is_message_definitions_file("messages.js.messages.js"));
    }

    /// All JS-like languages are technically supported, since SWC can
    /// transpile any of them and the relevant syntax will be the same across
    /// all of them.
    #[test]
    fn test_any_js_language_is_definitions_file() {
        assert!(is_message_definitions_file("en-US.messages.js"));
        assert!(is_message_definitions_file("Feature.messages.jsx"));
        assert!(is_message_definitions_file("fr-FR.messages.ts"));
        assert!(is_message_definitions_file("foo_bar.messages.tsx"));
    }

    /// This is needed because bundled apps will often omit the final extension
    /// for the sake of more flexible file lookups.
    #[test]
    fn test_no_final_extension_is_definitions_file() {
        assert!(is_message_definitions_file("en-US.messages"));
        assert!(is_message_definitions_file("Feature.messages"));
        assert!(is_message_definitions_file("fr-FR.messages"));
        assert!(is_message_definitions_file("fr.messages"));
    }

    #[test]
    fn test_other_languages_not_message_definitions_files() {
        assert!(!is_message_definitions_file("foo.messages.rs"));
        assert!(!is_message_definitions_file("foo.messages.py"));
        assert!(!is_message_definitions_file("foo.messages.jsonc"));

        assert!(!is_message_definitions_file("foo.messages.py.js"));
        assert!(!is_message_definitions_file("foo.messages.js.py"));
    }

    #[test]
    fn test_compound_suffix_not_definitions_file() {
        assert!(!is_message_definitions_file("feature.messages-foo.js"));
        assert!(!is_message_definitions_file("feature.foo-messages.js"));
        assert!(!is_message_definitions_file("feature.messages-messages.js"));
        assert!(!is_message_definitions_file("feature.en-US-messages.js"));
        assert!(!is_message_definitions_file("feature.messages-en_US.js"));
    }

    #[test]
    fn test_no_prefix_not_message_definitions_files() {
        assert!(!is_message_definitions_file("messages.js"));
        assert!(!is_message_definitions_file("messages.ts"));
    }

    #[test]
    fn test_intermediate_suffixes_not_definitions_files() {
        assert!(!is_message_definitions_file("messages.foo.js"));
        assert!(!is_message_definitions_file("messages.foo-bar.js"));
        assert!(!is_message_definitions_file("messages.foo-messages.js"));
        assert!(!is_message_definitions_file("foo.messages.js.js"));
    }

    /////
    // Translations
    /////

    #[test]
    fn test_is_message_translations_file_json() {
        assert!(is_message_translations_file("abc.messages.json"));
        assert!(is_message_translations_file("en-US.messages.json"));
        assert!(is_message_translations_file("Feature.messages.json"));
        assert!(is_message_translations_file(
            "Multiple.prefixes.messages.json"
        ));
        assert!(is_message_translations_file("messages.messages.json"));
        assert!(is_message_translations_file("message.messages.json"));
        assert!(is_message_translations_file(
            "messages.messages.messages.json"
        ));
        assert!(is_message_translations_file("Foo.messages.messages.json"));
        assert!(is_message_translations_file(
            "Even with spaces.messages.json"
        ));

        assert!(is_message_translations_file("abc.json.messages.json"));
        assert!(is_message_translations_file("json.messages.json"));
        assert!(is_message_translations_file("messages.json.messages.json"));
    }

    #[test]
    fn test_is_message_translations_file_jsona() {
        assert!(is_message_translations_file("abc.messages.jsona"));
        assert!(is_message_translations_file("en-US.messages.jsona"));
        assert!(is_message_translations_file("Feature.messages.jsona"));
        assert!(is_message_translations_file(
            "Multiple.prefixes.messages.jsona"
        ));
        assert!(is_message_translations_file("messages.messages.jsona"));
        assert!(is_message_translations_file("message.messages.jsona"));
        assert!(is_message_translations_file(
            "messages.messages.messages.jsona"
        ));
        assert!(is_message_translations_file("Foo.messages.messages.jsona"));
        assert!(is_message_translations_file(
            "Even with spaces.messages.jsona"
        ));

        assert!(is_message_translations_file("abc.jsona.messages.jsona"));
        assert!(is_message_translations_file("jsona.messages.jsona"));
        assert!(is_message_translations_file(
            "messages.jsona.messages.jsona"
        ));
    }

    #[test]
    fn test_other_languages_not_message_translations_files() {
        assert!(!is_message_translations_file("foo.messages.js"));
        assert!(!is_message_translations_file("foo.messages.jsx"));
        assert!(!is_message_translations_file("foo.messages.ts"));
        assert!(!is_message_translations_file("foo.messages.tsx"));
        assert!(!is_message_translations_file("foo.messages.rs"));
        assert!(!is_message_translations_file("foo.messages.py"));
        assert!(!is_message_translations_file("foo.messages.jsonc"));
    }

    #[test]
    fn test_compound_suffix_not_translations_file() {
        assert!(!is_message_translations_file("feature.messages-foo.json"));
        assert!(!is_message_translations_file("feature.messages-foo.jsona"));
        assert!(!is_message_translations_file("feature.foo-messages.json"));
        assert!(!is_message_translations_file("feature.foo-messages.jsona"));
        assert!(!is_message_translations_file(
            "feature.messages-messages.json"
        ));
        assert!(!is_message_translations_file(
            "feature.messages-messages.jsona"
        ));
        assert!(!is_message_translations_file("feature.en-US-messages.json"));
        assert!(!is_message_translations_file(
            "feature.en-US-messages.jsona"
        ));
        assert!(!is_message_translations_file("feature.messages-en_US.json"));
        assert!(!is_message_translations_file(
            "feature.messages-en_US.jsona"
        ));
    }

    #[test]
    fn test_no_prefix_not_message_translations_files() {
        assert!(!is_message_translations_file("messages.json"));
        assert!(!is_message_translations_file("messages.jsona"));
    }

    #[test]
    fn test_intermediate_suffixes_not_translations_files() {
        assert!(!is_message_translations_file("messages.foo.json"));
        assert!(!is_message_translations_file("messages.foo-bar.json"));
        assert!(!is_message_translations_file("messages.foo-messages.json"));
        assert!(!is_message_translations_file("messages.foo.jsona"));
        assert!(!is_message_translations_file("messages.foo-bar.jsona"));
        assert!(!is_message_translations_file("messages.foo-messages.jsona"));
    }

    #[test]
    fn test_compiled_artifact_is_not_translations_file() {
        assert!(!is_message_translations_file("foo.compiled.messages"));
        assert!(!is_message_translations_file("foo.compiled.messages.json"));
        assert!(!is_message_translations_file("foo.compiled.messages.jsona"));
    }

    /////
    // Compiled artifacts
    /////

    #[test]
    fn test_is_compiled_messages_artifact() {
        assert!(is_compiled_messages_artifact(
            "en-US.compiled.messages.json"
        ));
        assert!(is_compiled_messages_artifact(
            "en-US.compiled.messages.jsona"
        ));
        assert!(is_compiled_messages_artifact("fr.compiled.messages.json"));
        assert!(is_compiled_messages_artifact("fr.compiled.messages.jsona"));
        assert!(is_compiled_messages_artifact(
            "zh-CN.compiled.messages.json"
        ));
        assert!(is_compiled_messages_artifact(
            "zh-CN.compiled.messages.jsona"
        ));

        // While non-standard, this is still considered an artifact with a random locale.
        assert!(is_compiled_messages_artifact(
            "Some-Feature.compiled.messages.json"
        ));
        assert!(is_compiled_messages_artifact(
            "Some-Feature.compiled.messages.jsona"
        ));
    }

    #[test]
    fn test_is_not_compiled_messages_artifact() {
        assert!(!is_compiled_messages_artifact("en-US.compiled.json"));
        assert!(!is_compiled_messages_artifact("en-US.compiled.jsona"));
        assert!(!is_compiled_messages_artifact("fr.messages.json"));
        assert!(!is_compiled_messages_artifact("fr.messages.jsona"));
        assert!(!is_compiled_messages_artifact(
            "Some-Feature.compiled.messages"
        ));
        assert!(!is_compiled_messages_artifact("en-US.compiled.messages.js"));
        assert!(!is_compiled_messages_artifact("fr.compiled.messages.jsx"));
        assert!(!is_compiled_messages_artifact("fr.compiled.messages.ts"));
        assert!(!is_compiled_messages_artifact(
            "en-US.compiled.messages.tsx"
        ));
    }

    #[test]
    fn test_no_prefix_is_not_compiled_messages_artifact() {
        assert!(!is_compiled_messages_artifact("compiled.messages.json"));
        assert!(!is_compiled_messages_artifact("compiled.messages.jsona"));
    }

    #[test]
    fn test_other_order_is_not_compiled_messages_artifacts() {
        assert!(!is_compiled_messages_artifact(
            "en-US.json.compiled.messages"
        ));
        assert!(!is_compiled_messages_artifact(
            "en-US.compiled.messages.json.json"
        ));
        assert!(!is_compiled_messages_artifact(
            "en-US.compiled.messages.compiled.json"
        ));
        assert!(!is_compiled_messages_artifact(
            "en-US.messages.compiled.json"
        ));
        assert!(!is_compiled_messages_artifact(
            "compiled.en-US.messages.json"
        ));
        assert!(!is_compiled_messages_artifact(
            "compiled.messages.en-US.json"
        ));
    }

    /////
    // Any messages file
    ////

    #[test]
    fn test_supported_languages_are_messages_files() {
        assert!(is_any_messages_file("foo.messages.js"));
        assert!(is_any_messages_file("foo.messages.jsx"));
        assert!(is_any_messages_file("foo.messages.ts"));
        assert!(is_any_messages_file("foo.messages.tsx"));
        assert!(is_any_messages_file("foo.messages.json"));
        assert!(is_any_messages_file("foo.messages.jsona"));
        assert!(is_any_messages_file("foo.compiled.messages.json"));
        assert!(is_any_messages_file("foo.compiled.messages.jsona"));
    }
    #[test]
    fn test_no_final_extension_is_messages_file() {
        assert!(is_any_messages_file("foo.messages"));
        assert!(is_any_messages_file("en-US.messages"));
        assert!(is_any_messages_file("Even with spaces.messages"));
    }

    #[test]
    fn test_other_languages_are_not_messages_files() {
        assert!(!is_any_messages_file("foo.messages.py"));
        assert!(!is_any_messages_file("foo.messages.rs"));
        assert!(!is_any_messages_file("foo.messages.jsonc"));
        assert!(!is_any_messages_file("foo.messages.md"));
        assert!(!is_any_messages_file("foo.messages.txt"));
        assert!(!is_any_messages_file("foo.messages.yaml"));
    }

    #[test]
    fn no_prefix_is_not_messages_file() {
        assert!(!is_any_messages_file("messages.py"));
        assert!(!is_any_messages_file("messages.rs"));
        assert!(!is_any_messages_file("messages.js"));
        assert!(!is_any_messages_file("messages.jsx"));
        assert!(!is_any_messages_file("messages.ts"));
        assert!(!is_any_messages_file("messages.tsx"));
        assert!(!is_any_messages_file("messages.json"));
        assert!(!is_any_messages_file("messages.jsona"));
        assert!(!is_any_messages_file("compiled.messages.json"));
        assert!(!is_any_messages_file("compiled.messages.jsona"));
    }
}
