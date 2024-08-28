//! Small module for creating and working with Symbols (aka Atoms), which are
//! internal handles to commonly-shared values like message keys, file names
//! locale ids, or anything else that needs to be shared.
use ustr::{existing_ustr, ustr, Ustr, UstrMap, UstrSet};

use crate::messages::{MessagesError, MessagesResult};

/// A symbol representing a message key, file name, or any other frequently-
/// copied value. This is not the same as hashing a message key, which creates
/// a short _string_ representation, while this symbol is just a lightweight
/// integer handle for use as keys and references internally.
pub type KeySymbol = Ustr;

pub type KeySymbolMap<Value> = UstrMap<Value>;
pub type KeySymbolSet = UstrSet;

/// Return the KeySymbol that represents the given value. If the requested
/// value is not currently known in the store, or if the store is poisoned,
/// a MessagesError is returned instead.
pub fn global_get_symbol(value: &str) -> Option<KeySymbol> {
    existing_ustr(value)
}

pub fn global_get_symbol_or_error(value: &str) -> MessagesResult<KeySymbol> {
    global_get_symbol(value).ok_or_else(|| MessagesError::ValueNotInterned(value.into()))
}

/// Intern a new value into the global symbol store. This is thread-safe, but
/// will lock any reads from the store that are happening concurrently.
///
/// If the store can't be acquired or the write otherwise fails, this returns
/// a MessagesError.
pub fn global_intern_string(value: &str) -> KeySymbol {
    ustr(value)
}
