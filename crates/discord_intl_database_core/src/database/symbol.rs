//! Small module for creating and working with Symbols (aka Atoms), which are
//! internal handles to commonly-shared values like message keys, file names
//! locale ids, or anything else that needs to be shared.
use ustr::{existing_ustr, ustr, Ustr, UstrMap, UstrSet};

/// A symbol representing a message key, file name, or any other frequently-
/// copied value. This is not the same as hashing a message key, which creates
/// a new, shorter representation of a message key for identification,
/// obfuscation, and minification.
pub type KeySymbol = Ustr;

pub type KeySymbolMap<Value> = UstrMap<Value>;
pub type KeySymbolSet = UstrSet;

/// Return the KeySymbol that represents the given value. If the requested
/// value is not currently known in the store, or if the store is poisoned,
/// a DatabaseError is returned instead.
pub fn get_key_symbol(value: &str) -> Option<KeySymbol> {
    existing_ustr(value)
}

/// Intern a new value into the global symbol store. This is thread-safe, but
/// will lock any reads from the store that are happening concurrently.
///
/// If the store can't be acquired or the write otherwise fails, this returns
/// a DatabaseError.
pub fn key_symbol(value: &str) -> KeySymbol {
    ustr(value)
}
