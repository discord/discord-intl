use swc_core::ecma::ast::Module;

pub use definitions_parser::{extract_message_definitions, parse_message_definitions_file};
pub use translations::{extract_message_translations, Translations};

use crate::messages::{
    FilePosition, global_intern_string, KeySymbol, Message, MessageMeta, MessagesDatabase,
    MessagesError, MessagesResult, MessageValue,
};
use crate::messages::symbols::KeySymbolSet;
use crate::sources::definitions_parser::ExtractedMessage;

mod definitions_parser;
mod translations;

pub fn parse_definitions_file(file_name: &str, content: &str) -> MessagesResult<Module> {
    parse_message_definitions_file(file_name, content).map_err(MessagesError::DefinitionParseError)
}

pub fn process_definitions_file(
    db: &mut MessagesDatabase,
    file_name: &str,
    content: &str,
    locale: &str,
) -> MessagesResult<KeySymbol> {
    let file_key = global_intern_string(file_name);
    let file_locale = global_intern_string(locale);
    let extracted = parse_definitions_file(file_name, content).map(extract_message_definitions)?;
    let file_meta = extracted.root_meta.unwrap_or(MessageMeta::default());

    let definitions = extracted.message_definitions;
    let mut inserted_keys = KeySymbolSet::default();

    // Check if this file has already been processed into the database before. If it has, this
    // becomes an Update operation, which allows definitions to be overridden. Otherwise, it is
    if let Some(existing_source_file) = db.get_source_file(file_key) {
        // Keys that used to exist in the file but are not found on this iteration are _removed_,
        // meaning they will have their entry in the database taken out, and won't be considered
        // part of the source file anymore.
        // To do that, this takes the existing list of keys and removes each found entry from it,
        // then uses those keys to delete values from the database.
        let mut to_remove = existing_source_file.message_keys().clone();
        for definition in definitions.into_iter() {
            let message = handle_definition(db, file_key, definition, file_locale)?;
            to_remove.remove(&message.key());
            inserted_keys.insert(message.key());
        }

        for key in to_remove {
            db.remove_definition(key);
        }
    } else {
        db.create_source_file(file_key, file_meta);
        // An insert operation doesn't need to track any existing behavior, so it can just insert
        // incrementally. The interior will track adding the keys to the set.
        for definition in definitions.into_iter() {
            let message = handle_definition(db, file_key, definition, file_locale)?;
            inserted_keys.insert(message.key());
        }
    }

    db.set_source_file_keys(file_key, inserted_keys)?;
    Ok(file_key)
}

/// Parse the message from the given `definition`, apply additional information from `file_key` and
/// `file_locale`, then insert it into the database.
fn handle_definition(
    db: &mut MessagesDatabase,
    file_key: KeySymbol,
    definition: ExtractedMessage,
    file_locale: KeySymbol,
) -> MessagesResult<&Message> {
    let value = MessageValue::from_raw(&definition.value).with_file_position(FilePosition {
        file: file_key,
        offset: definition.offset,
    });

    db.insert_definition(&definition.name, value, file_locale, definition.meta, true)
}
