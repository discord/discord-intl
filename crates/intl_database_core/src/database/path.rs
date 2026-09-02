//! File paths are interned as `KeySymbol`s and compared byte-for-byte to decide whether two
//! definitions conflict, so a file needs exactly one spelling. On Windows it has several: a path
//! from `fs.realpathSync` (which Node applies to every resolved module, and pnpm's junctions
//! guarantee we hit) reports the drive letter upper case, while one built from `process.cwd()`
//! keeps whatever case the parent process used to `cd`.

use std::borrow::Cow;

use crate::database::symbol::{get_key_symbol, key_symbol, KeySymbol};

/// Intern `path` as a source file key. Message keys and locales are not paths - they keep using
/// [`key_symbol`].
pub fn file_key_symbol(path: &str) -> KeySymbol {
    key_symbol(&normalize_file_path(path))
}

/// Lookup counterpart to [`file_key_symbol`].
pub fn existing_file_key_symbol(path: &str) -> Option<KeySymbol> {
    get_key_symbol(&normalize_file_path(path))
}

/// Canonical spelling of `path` for use as a database key.
pub fn normalize_file_path(path: &str) -> Cow<'_, str> {
    if cfg!(windows) {
        normalize_windows_file_path(path)
    } else {
        Cow::Borrowed(path)
    }
}

/// Not `cfg`-gated, so it stays under test on platforms that are not Windows.
fn normalize_windows_file_path(path: &str) -> Cow<'_, str> {
    // `\\?\UNC\server\share` is `\\server\share`, which is not a substring of the input.
    if let Some(unc) = path.strip_prefix(r"\\?\UNC\") {
        let unc_path = format!(r"\\{unc}");
        return Cow::Owned(normalize_spelling(&unc_path).into_owned());
    }
    // Only drive-letter and UNC paths have a non-verbatim spelling. `\\?\Volume{...}\` has none, so
    // stripping its prefix would give a path that cannot resolve.
    let path = match path.strip_prefix(r"\\?\") {
        Some(rest) if drive_letter(rest).is_some() => rest,
        _ => path,
    };
    normalize_spelling(path)
}

/// Spelling only: resolving `.`, `..` or a relative path needs a working directory, which belongs
/// to the caller and not the database.
fn normalize_spelling(path: &str) -> Cow<'_, str> {
    let has_lower_drive = drive_letter(path).is_some_and(|drive| drive.is_ascii_lowercase());
    let has_forward_slash = path.contains('/');
    if !has_lower_drive && !has_forward_slash {
        return Cow::Borrowed(path);
    }

    let mut normalized = String::with_capacity(path.len());
    for (index, character) in path.char_indices() {
        normalized.push(match character {
            '/' => '\\',
            drive if index == 0 && has_lower_drive => drive.to_ascii_uppercase(),
            other => other,
        });
    }
    Cow::Owned(normalized)
}

fn drive_letter(path: &str) -> Option<char> {
    match path.as_bytes() {
        [drive, b':', ..] if drive.is_ascii_alphabetic() => Some(*drive as char),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_windows_file_path;

    #[test]
    fn upper_cases_the_drive_letter() {
        assert_eq!(
            normalize_windows_file_path(r"b:\b\app\en-US.messages.js"),
            r"B:\b\app\en-US.messages.js"
        );
        assert_eq!(
            normalize_windows_file_path(r"B:\b\app\en-US.messages.js"),
            r"B:\b\app\en-US.messages.js"
        );
    }

    #[test]
    fn rewrites_forward_slashes() {
        assert_eq!(
            normalize_windows_file_path("b:/b/app/en-US.messages.js"),
            r"B:\b\app\en-US.messages.js"
        );
        assert_eq!(
            normalize_windows_file_path("app/en-US.messages.js"),
            r"app\en-US.messages.js"
        );
    }

    #[test]
    fn strips_a_verbatim_prefix() {
        assert_eq!(
            normalize_windows_file_path(r"\\?\b:\b\app\en-US.messages.js"),
            r"B:\b\app\en-US.messages.js"
        );
    }

    #[test]
    fn rewrites_a_verbatim_unc_prefix() {
        assert_eq!(
            normalize_windows_file_path(r"\\?\UNC\build\share\en-US.messages.js"),
            r"\\build\share\en-US.messages.js"
        );
    }

    #[test]
    fn keeps_paths_it_cannot_spell_normally() {
        let path = r"\\?\Volume{9c4a4d5f-0000-0000-0000-100000000000}\app\en-US.messages.js";
        assert_eq!(normalize_windows_file_path(path), path);
    }

    #[test]
    fn leaves_a_unc_path_alone() {
        assert_eq!(
            normalize_windows_file_path(r"\\build\share\en-US.messages.js"),
            r"\\build\share\en-US.messages.js"
        );
    }

    #[test]
    fn does_not_case_fold_the_rest_of_the_path() {
        // NTFS is case-insensitive throughout, so this is not the whole story. Folding the rest
        // would mangle the path in every diagnostic we print, and nothing has needed it.
        assert_eq!(
            normalize_windows_file_path(r"b:\b\App\EN-us.Messages.js"),
            r"B:\b\App\EN-us.Messages.js"
        );
    }

    #[test]
    fn does_not_resolve_the_path() {
        assert_eq!(
            normalize_windows_file_path(r"b:\b\app\..\app\.\en-US.messages.js"),
            r"B:\b\app\..\app\.\en-US.messages.js"
        );
    }
}
