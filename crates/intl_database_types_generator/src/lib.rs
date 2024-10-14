use rustc_hash::FxHashSet;
use sora::{Mapping, Mappings, SourceMapBuilder};
use std::fmt::Write;
use std::io::{
    BufRead,
    // Needed because this file uses both fmt::Write and io::Write for the write! macro.
    Write as IoWrite,
};
use thiserror::Error;
use ustr::Ustr;

use intl_database_core::{
    KeySymbol, KeySymbolMap, KeySymbolSet, Message, MessageValue, MessageVariableType,
    MessagesDatabase,
};
use intl_database_service::IntlDatabaseService;

/// Struct for tracking the current position that has been written in a Write buffer.
struct PositionTrackingWriter<'a, W: std::io::Write> {
    output: &'a mut W,
    line: usize,
    col: usize,
}

impl<'a, W: std::io::Write> PositionTrackingWriter<'a, W> {
    fn new(output: &'a mut W) -> Self {
        Self {
            output,
            line: 1,
            col: 0,
        }
    }
}

impl<W: std::io::Write> std::io::Write for PositionTrackingWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut lines = BufRead::lines(buf).peekable();
        loop {
            let Some(Ok(line)) = lines.next() else {
                break;
            };

            match lines.peek() {
                // If there's another line, reset the column and increment the line count.
                Some(_) => {
                    self.col = 1;
                    self.line += 1;
                }
                // Count the column only on the last line of the given buffer.
                None => {
                    if buf.last().is_some_and(|last| b'\n' == *last) {
                        self.line += 1;
                        self.col = 1;
                    } else {
                        self.col += line.len();
                    }
                    break;
                }
            }
        }
        self.output.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

pub struct IntlTypesGenerator<'a, W: std::io::Write> {
    database: &'a MessagesDatabase,
    source_file_key: KeySymbol,
    output_file_path: String,
    output: PositionTrackingWriter<'a, W>,
    allow_nullability: bool,
    source_map: Vec<Mapping>,
}

impl<'a, W: std::io::Write> IntlTypesGenerator<'a, W> {
    pub fn new(
        database: &'a MessagesDatabase,
        source_file_key: KeySymbol,
        output: &'a mut W,
        output_file_path: String,
        allow_nullability: bool,
    ) -> Self {
        Self {
            database,
            source_file_key,
            output: PositionTrackingWriter::new(output),
            output_file_path,
            allow_nullability,
            source_map: Vec::with_capacity(database.messages.len()),
        }
    }

    fn make_doc_comment(
        &self,
        message: &Message,
        known_locales: &KeySymbolSet,
        spurious_variables: &Vec<(Ustr, String)>,
    ) -> anyhow::Result<String> {
        if message.is_defined() {
            self.make_normal_message_doc_comment(message, known_locales, spurious_variables)
        } else {
            self.make_undefined_message_doc_comment(message, known_locales)
        }
    }

    fn make_normal_message_doc_comment(
        &self,
        message: &Message,
        known_locales: &KeySymbolSet,
        spurious_variables: &Vec<(Ustr, String)>,
    ) -> anyhow::Result<String> {
        let key = message.hashed_key();
        // SAFETY: The caller asserts that this message is defined, so it must have a source.
        let default_translation = message.get_source_translation().unwrap();
        let translation_links =
            self.build_translation_links(message.translations(), Some(default_translation))?;

        let found_locales: KeySymbolSet = message.translations().keys().map(Clone::clone).collect();

        let missing_locales = {
            let diff = known_locales.difference(&found_locales);
            let mut missing_names = vec![];
            for key in diff {
                missing_names.push(format!("`{key}`"));
            }
            missing_names.sort();
            missing_names
        };

        let mut result = String::new();
        write!(result, "  /**\n   * Key: `{key}`.")?;

        if translation_links.is_empty() {
            write!(result, " **Untranslated**\n")?;
        } else if missing_locales.is_empty() {
            write!(result, " Translated in all languages\n")?;
        } else {
            write!(
                result,
                " Missing {} translations: {}\n",
                missing_locales.len(),
                missing_locales.join(", ")
            )?;
        }

        write!(
            result,
            "   * ```text\n   * {}\n   * ```\n",
            default_translation.raw,
        )?;

        if spurious_variables.len() > 0 {
            write!(result, "   * Spurious variables from translations:\n")?;
            for (_name, doc_text) in spurious_variables {
                write!(result, "   * - {doc_text}\n")?;
            }
        }

        write!(result, "   */")?;

        Ok(result)
    }

    fn build_translation_links(
        &self,
        translations: &KeySymbolMap<MessageValue>,
        default_translation: Option<&MessageValue>,
    ) -> anyhow::Result<Vec<String>> {
        let mut links = Vec::with_capacity(translations.len() - 1);

        for (key, translation) in translations {
            if default_translation.is_some_and(|t| t == translation) {
                continue;
            }
            // let file = symbol_store
            //     .resolve(translation.file())
            //     .ok_or(MessagesError::SymbolNotFound(*key))?;
            // let link = format!("[{}]({})", locale, file);
            links.push(key.to_string())
        }

        Ok(links)
    }

    /// Return a list of variables that are only present in non-default translations of the given
    /// `message`. Each entry is a pre-formatted String containing the variable name and the list
    /// of locales that contain it.
    fn build_spurious_variables_info(
        &self,
        message: &Message,
    ) -> anyhow::Result<Vec<(Ustr, String)>> {
        let Some(source) = message.get_source_translation() else {
            return Ok(vec![]);
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
        let mut spurious_variables: KeySymbolMap<KeySymbolSet> = KeySymbolMap::default();

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
                    if !spurious_variables.contains_key(variable) {
                        spurious_variables.insert(*variable, KeySymbolSet::default());
                    }
                    spurious_variables
                        .get_mut(variable)
                        .unwrap()
                        .insert(*locale_key);
                }
            }
        }

        let mut lines = vec![];
        for (variable, locales) in spurious_variables {
            lines.push((
                variable,
                format!(
                    "`{variable}` -- {}",
                    Vec::from_iter(locales.into_iter().map(|locale| locale.as_str())).join(", ")
                ),
            ));
        }
        lines.sort();
        Ok(lines)
    }

    fn make_undefined_message_doc_comment(
        &self,
        message: &Message,
        _known_locales: &KeySymbolSet,
    ) -> anyhow::Result<String> {
        let mut result = String::new();
        let translation_links = self.build_translation_links(message.translations(), None)?;

        write!(
            result,
            "  /**\n   * Translated in: {}\n   * @deprecated - Not defined in default locale\n   */",
            translation_links.join(", "),
        )?;
        Ok(result)
    }

    fn make_getter_type_def(
        &self,
        message: &Message,
        spurious_variables: &Vec<(Ustr, String)>,
    ) -> anyhow::Result<String> {
        let name = message.key();
        let variables = message.all_variables();
        let mut entries = vec![];
        for (name, instances) in variables.iter() {
            let type_names = instances
                .iter()
                .map(|instance| {
                    if self.allow_nullability {
                        get_loose_type_name(&instance.kind)
                    } else {
                        get_variable_type_name(&instance.kind)
                    }
                })
                .collect::<FxHashSet<&str>>();
            // TODO: Do this once per variable rather than having to check every instance, since
            // builtin-ness is determined by the name, not the instance.
            let is_builtin = instances.iter().any(|instance| instance.is_builtin);
            let type_str = Vec::from_iter(type_names).join(" | ");
            // TODO: These types shouldn't actually be optional, as they'll crash at runtime.
            // Optionality is just a migration step.
            let is_optional = spurious_variables
                .iter()
                .any(|(spurious_name, _)| spurious_name == name);
            entries.push(format!(
                "{}{}: {}",
                name,
                if is_optional || is_builtin { "?" } else { "" },
                type_str
            ));
        }

        entries.sort();
        Ok(format!(
            "'{name}': TypedIntlMessageGetter<{{{}}}>,",
            entries.join(", ")
        ))
    }

    pub fn into_sourcemap(mut self) -> std::io::Result<String> {
        self.source_map
            .sort_by_key(|mapping| mapping.generated().line);
        let builder = SourceMapBuilder::default()
            .with_mappings(Mappings::new(self.source_map))
            .with_file(self.output_file_path.into())
            .with_sources(vec![Some(self.source_file_key.as_str().into())]);
        // SAFETY: We don't have `source_content`, because it's unnecessary but required by
        // basically every library out there. That's the only thing missing from this map.
        let map = unsafe { builder.build_unchecked() };
        map.to_string()
    }
}

fn get_variable_type_name(kind: &MessageVariableType) -> &str {
    match kind {
        MessageVariableType::Any => "any",
        MessageVariableType::Number => "number",
        MessageVariableType::Plural => "number",
        MessageVariableType::Enum(_) => todo!(),
        MessageVariableType::Date => "number | string | Date",
        MessageVariableType::Time => "number | string | Date",
        MessageVariableType::HookFunction => "HookFunction",
        MessageVariableType::LinkFunction => "LinkFunction",
        MessageVariableType::HandlerFunction => "HandlerFunction",
    }
}

/// When `allow_nullability` is true, use this method in place of `get_variable_type_name` to get
/// a type that allows nulls and other looser types for the variable.
fn get_loose_type_name(kind: &MessageVariableType) -> &str {
    // TODO: All of these undefined unions are technically incorrect and should
    // be handled on the consuming side somehow.
    match kind {
        MessageVariableType::Any => "any",
        MessageVariableType::Number => "number | string | null | undefined",
        MessageVariableType::Plural => "number | string | null | undefined",
        MessageVariableType::Enum(_) => todo!(),
        MessageVariableType::Date => "number | string | Date | null | undefined",
        MessageVariableType::Time => "number | string | Date | null | undefined",
        MessageVariableType::HookFunction => "HookFunction",
        MessageVariableType::LinkFunction => "LinkFunction",
        MessageVariableType::HandlerFunction => "HandlerFunction",
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

impl<W: std::io::Write> IntlDatabaseService for IntlTypesGenerator<'_, W> {
    type Result = anyhow::Result<()>;

    fn run(&mut self) -> Self::Result {
        write!(
            self.output,
            "/* THIS FILE IS AUTOGENERATED. DO NOT EDIT MANUALLY. */
/* eslint-disable */
/* prettier-ignore */

import {{MessageLoader, TypedIntlMessageGetter, HandlerFunction, HookFunction, LinkFunction}} from '{}';

export declare const messagesLoader: MessageLoader;

declare const messages: {{
",
            intl_message_utils::RUNTIME_PACKAGE_NAME
        )?;

        let known_locales = &self.database.known_locales;
        let source_file = self.database.sources.get(&self.source_file_key).ok_or(
            IntlTypesGeneratorError::SourceFileNotFound(self.source_file_key),
        )?;

        let source_message_keys = get_sorted_message_keys(source_file.message_keys());
        for message_key in source_message_keys {
            let Some(message) = self.database.messages.get(message_key) else {
                return Err(IntlTypesGeneratorError::SourceFileMessageNotFound(
                    *message_key,
                    self.source_file_key,
                )
                .into());
            };

            let spurious_variables = self.build_spurious_variables_info(message)?;
            let doc_comment = self.make_doc_comment(message, known_locales, &spurious_variables)?;
            let type_def = self.make_getter_type_def(message, &spurious_variables)?;
            write!(self.output, "{doc_comment}\n  ")?;
            // Ordering is important here. `self.output` tracks the line and column position in the
            // written output, and we want to know that number precisely when adding the source map
            // entry here. So the doc comment is written first to get it out of the way, then we
            // know we're at the start of the name token for the message entry, so we can add that
            // current position to the map.
            if let Some(definition_position) = message
                .get_source_translation()
                .and_then(|definition| definition.file_position)
            {
                self.source_map.push(
                    Mapping::new(self.output.line as u32, self.output.col as u32).with_source(
                        0,
                        // I couldn't possibly tell you whether 0- or 1-based indexing is correct.
                        // The spec doesn't say which.
                        definition_position.line - 1,
                        definition_position.col,
                    ),
                )
            }
            write!(self.output, "{type_def}\n")?;
        }

        write!(self.output, "}};\nexport default messages;")?;

        Ok(())
    }
}
