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

/// Returns true if the given `file_name` is considered a message definitions file.
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
    file_name.ends_with(".messages.json") || file_name.ends_with(".messages.jsona")
}

pub fn is_compiled_messages_artifact(file_name: &str) -> bool {
    file_name.contains(".compiled.messages.")
}

pub fn is_any_messages_file(file_name: &str) -> bool {
    // Split into <prefix> <second_extension> <last_extension>. A file is a messages file
    // if `last_extension` or `second_extension` is `messages`, meaning anything like `.messages.js`
    // or `.messages.py` or anything else counts, as well as implicit final extensions, like
    // `Feature.messages` in languages where extensions aren't used in imports.
    let mut parts = file_name.rsplitn(3, '.');
    let last_extension = parts.next();
    let second_extension = parts.next();
    let stem = parts.next();

    let is_messages_extesnsion = last_extension.is_some_and(|ext| ext == "messages")
        || second_extension.is_some_and(|ext| ext == "messages");

    // Disallow any file with more prefix parts, such as `.compiled.messages.json`.
    is_messages_extesnsion && !stem.is_some_and(|stem| stem.contains('.'))
}

static DOUBLE_NEWLINE_FINDER: Lazy<memmem::Finder> = Lazy::new(|| memmem::Finder::new(b"\n\n"));

/// Returns true if the given `message` contains block-like content and should
/// be parsed with blocks included. For now, this requires that the message
/// contains a double newline anywhere inside it.
pub fn message_may_have_blocks(message: &str) -> bool {
    DOUBLE_NEWLINE_FINDER.find(message.as_bytes()).is_some()
}
