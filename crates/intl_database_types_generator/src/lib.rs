mod comment;
mod type_def;
mod writer;

use rustc_hash::FxHashSet;
use std::fmt::Write;
use thiserror::Error;

use crate::comment::DocComment;
use crate::type_def::TypeDef;
use crate::writer::{
    source_map_entry, write_doc, AlphabeticSymbolMap, AlphabeticSymbolSet, TypeDocFormat,
    TypeDocWriter, WriteResult,
};
use intl_database_core::{KeySymbol, KeySymbolSet, Message, MessagesDatabase};
use intl_database_service::IntlDatabaseService;

pub struct IntlTypesGenerator<'a> {
    database: &'a MessagesDatabase,
    source_file_key: KeySymbol,
    output: TypeDocWriter,
    output_file_path: String,
}

impl<'a> IntlTypesGenerator<'a> {
    pub fn new(
        database: &'a MessagesDatabase,
        source_file_key: KeySymbol,
        output_file_path: String,
    ) -> Self {
        Self {
            database,
            source_file_key,
            output: TypeDocWriter::new(),
            output_file_path,
        }
    }

    pub fn take_buffer(&mut self) -> String {
        self.output.take_buffer()
    }

    fn make_doc_comment<'b>(
        &self,
        message: &'b Message,
        known_locales: &KeySymbolSet,
        spurious_variables: AlphabeticSymbolMap<AlphabeticSymbolSet>,
    ) -> DocComment<'b> {
        let found_locales: KeySymbolSet = message.translations().keys().map(Clone::clone).collect();
        let missing_locales = known_locales.difference(&found_locales).map(Clone::clone);

        DocComment {
            key: message.hashed_key(),
            value: message
                .get_source_translation()
                .map(|definition| definition.raw.as_str()),
            description: None,
            missing_translations: AlphabeticSymbolSet::from_iter(missing_locales),
            is_secret: message.meta().secret,
            ready_to_translate: message.meta().translate,
            spurious_variables,
        }
    }

    /// Return a list of variables that are only present in non-default translations of the given
    /// `message`. Each entry is a pre-formatted String containing the variable name and the list
    /// of locales that contain it.
    fn build_spurious_variables(
        &self,
        message: &Message,
    ) -> AlphabeticSymbolMap<AlphabeticSymbolSet> {
        let Some(source) = message.get_source_translation() else {
            return AlphabeticSymbolMap::default();
        };

        let source_variables = source
            .variables
            .as_ref()
            .map(|variables| variables.get_keys())
            .unwrap_or(FxHashSet::default());

        // Map of variable keys to locales that define them when the variable
        // is not present in the source message.
        // We only care about _added_ variables, since they affect the type but
        // aren't immediately apparent from the rest of the doc comment (which
        // only shows the source message value).
        let mut spurious_variables: AlphabeticSymbolMap<AlphabeticSymbolSet> =
            AlphabeticSymbolMap::default();

        for (locale_key, translation) in message.translations() {
            if translation == source {
                continue;
            }

            let Some(variables) = &translation.variables else {
                continue;
            };

            for variable in variables.get_keys() {
                if !source_variables.contains(variable) {
                    // For some reason, `entry().or_insert().and_modify()` is not available here.
                    spurious_variables
                        .entry(*variable)
                        .and_modify(|set| {
                            set.insert(*locale_key);
                        })
                        .or_insert(AlphabeticSymbolSet::from([*locale_key]));
                }
            }
        }

        spurious_variables
    }

    fn make_getter_type_def(&self, message: &Message) -> TypeDef {
        TypeDef {
            name: message.key(),
            variables: message.all_variables(),
        }
    }

    pub fn into_sourcemap(mut self) -> anyhow::Result<String> {
        let mut result = Vec::with_capacity(self.database.messages.len() * 10);
        self.output.source_map.set_file(Some(self.output_file_path));
        self.output
            .source_map
            .into_sourcemap()
            .to_writer(&mut result)?;
        Ok(String::from_utf8(result)?)
    }
}

/// Returns the set of message keys as a list, sorted alphabetically by the key's resolved value.
/// This is annoyingly inefficient with two vector allocations, but it works.
fn get_sorted_message_keys(keys: &KeySymbolSet) -> Vec<&KeySymbol> {
    let mut sorted_keys = Vec::from_iter(keys.into_iter());
    sorted_keys.sort();
    sorted_keys
}

#[derive(Debug, Error)]
pub enum IntlTypesGeneratorError {
    #[error("Requested source file '{0}' does not exist.")]
    SourceFileNotFound(KeySymbol),
    #[error("Message key '{0}' from source file '{1}' does not exist in the database.")]
    SourceFileMessageNotFound(KeySymbol, KeySymbol),
}

impl IntlDatabaseService for IntlTypesGenerator<'_> {
    type Result = WriteResult;

    fn run(&mut self) -> Self::Result {
        self.output.source_map.add_source(&self.source_file_key);
        self.output.write_prelude()?;
        self.output.indent();

        let known_locales = &self.database.known_locales;
        let Some(source_file) = self.database.sources.get(&self.source_file_key) else {
            return Ok(());
        };

        let source_message_keys = get_sorted_message_keys(source_file.message_keys());
        for message_key in source_message_keys {
            let message = self
                .database
                .messages
                .get(&message_key)
                .expect("Expected all source file message keys to have values in the database");

            let spurious_variables = self.build_spurious_variables(message);
            let type_def = self.make_getter_type_def(message);
            let doc_comment = self.make_doc_comment(message, known_locales, spurious_variables);
            let definition_position = message
                .get_source_translation()
                .map(|definition| definition.file_position);

            // Ordering is important here. `self.output` tracks the line and column position in the
            // written output, and we want to know that number precisely when adding the source map
            // entry here. So the doc comment is written first to get it out of the way, then we
            // know we're at the start of the name token for the message entry, so we can add that
            // current position to the map.
            write_doc!(
                self.output,
                [
                    "\n",
                    &doc_comment,
                    "\n",
                    &definition_position.map(source_map_entry),
                    &type_def,
                    ","
                ]
            )?;
        }
        self.output.dedent();

        write!(self.output, "\n}};\nexport default messages;")?;

        Ok(())
    }
}
