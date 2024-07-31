//! Small module for creating and working with Symbols (aka Atoms), which are
//! internal handles to commonly-shared values like message keys, file names
//! locale ids, or anything else that needs to be shared.
use rustc_hash::FxHasher;
use serde::ser::Error;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::BuildHasherDefault;
use std::sync::{OnceLock, RwLock, RwLockReadGuard};
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner, Symbol};

use super::{MessagesError, MessagesResult};

// TODO: Propagate these errors rather than just using `.ok()?`.

/// Returns a global, mutexed symbol store
pub fn global_symbol_store() -> &'static RwLock<SymbolStore> {
    static SYMBOL_STORE: OnceLock<RwLock<SymbolStore>> = OnceLock::new();
    SYMBOL_STORE.get_or_init(|| RwLock::new(SymbolStore::new()))
}

/// Acquire and return a read handle on the global symbol store,
/// returning a MessagesError if it can't be acquired.
#[inline]
pub fn read_global_symbol_store<'a>() -> MessagesResult<RwLockReadGuard<'a, SymbolStore>> {
    global_symbol_store()
        .read()
        .map_err(|_| MessagesError::SymbolStorePoisonedError)
}

/// Return the KeySymbol that represents the given value. If the requested
/// value is not currently known in the store, or if the store is poisoned,
/// a MessagesError is returned instead.
pub fn global_get_symbol(value: &str) -> MessagesResult<KeySymbol> {
    read_global_symbol_store()?
        .get(value)
        .ok_or_else(|| MessagesError::ValueNotInterned(value.into()))
}

/// Intern a new value into the global symbol store. This is thread-safe, but
/// will lock any reads from the store that are happening concurrently.
///
/// If the store can't be acquired or the write otherwise fails, this returns
/// a MessagesError.
pub fn global_intern_string(value: &str) -> MessagesResult<KeySymbol> {
    let mut store = global_symbol_store()
        .write()
        .map_err(|_| MessagesError::SymbolStorePoisonedError)?;
    Ok(store.intern(value))
}

/// Resolve a symbol from the value of the given variable name to a `&str`. If the symbol can't be
/// found, or if the symbol store can't be read, this macro returns early with a MessagesError.
#[macro_export]
macro_rules! resolve_symbol {
    ($var:ident) => {{
        let store = read_global_symbol_store()?;
        store
            .resolve($var)
            .ok_or_else(|| MessagesError::SymbolNotFound($var))
    }};
}

/// A symbol representing a message key, file name, or any other frequently-
/// copied value. This is not the same as hashing a message key, which creates
/// a short _string_ representation, while this symbol is just a lightweight
/// integer handle for use as keys and references internally.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct KeySymbol(DefaultSymbol);

impl KeySymbol {
    pub fn from_usize(value: usize) -> Option<Self> {
        let symbol = DefaultSymbol::try_from_usize(value)?;
        Some(KeySymbol(symbol))
    }

    pub fn value(&self) -> usize {
        self.0.to_usize()
    }
}

impl Serialize for KeySymbol {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let symbol_store = global_symbol_store().read().map_err(Error::custom)?;
        let value = symbol_store.resolve(*self);
        match value {
            Some(value) => serializer.serialize_str(value),
            None => serializer.serialize_none(),
        }
    }
}

impl std::fmt::Display for KeySymbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let symbol_store = global_symbol_store()
            .read()
            .map_err(|_| Error::custom("Could not read global symbol store"))?;
        if let Some(value) = symbol_store.resolve(*self) {
            f.write_str(value)?;
        }
        Ok(())
    }
}

pub struct SymbolStore {
    interner: StringInterner<DefaultBackend, BuildHasherDefault<FxHasher>>,
}

impl SymbolStore {
    pub fn new() -> Self {
        Self {
            interner: StringInterner::new(),
        }
    }

    pub fn intern(&mut self, file_name: &str) -> KeySymbol {
        KeySymbol(self.interner.get_or_intern(file_name))
    }

    pub fn get(&self, value: &str) -> Option<KeySymbol> {
        self.interner.get(value).map(KeySymbol)
    }

    pub fn resolve(&self, symbol: KeySymbol) -> Option<&str> {
        self.interner.resolve(symbol.0)
    }
}

/// A specialized hasher for using KeySymbols as hash keys. Since symbols are
/// already u64 by default (usize, but we're always 64-bit), the values don't
/// actually need to be hashed at all, and the value is just used as the hash
/// result directly.
#[derive(Clone, Debug, Default)]
pub struct KeySymbolHasher {
    value: u64,
}

impl std::hash::Hasher for KeySymbolHasher {
    fn finish(&self) -> u64 {
        self.value
    }

    fn write(&mut self, _: &[u8]) {
        unreachable!("KeySymbolHasher is only valid for single unsigned integer values. No other types may be written.")
    }

    fn write_u32(&mut self, value: u32) {
        self.value = value as u64
    }

    fn write_u64(&mut self, value: u64) {
        self.value = value;
    }

    fn write_usize(&mut self, value: usize) {
        self.value = value as u64;
    }
}

pub type KeySymbolMap<Value> = HashMap<KeySymbol, Value, BuildHasherDefault<KeySymbolHasher>>;
