use std::fmt::Write;

use rustc_hash::FxHashSet;
use thiserror::Error;

use crate::{
    messages::{
        KeySymbol, KeySymbolMap, Message, MessagesDatabase, MessagesError,
        MessageVariableType, read_global_symbol_store,
    },
    services::IntlService,
};
use crate::messages::MessageValue;

pub struct IntlTypesGenerator<'a, W: std::io::Write> {
    database: &'a MessagesDatabase,
    source_file_key: KeySymbol,
    output: &'a mut W,
}

impl<'a, W: std::io::Write> IntlTypesGenerator<'a, W> {
    pub fn new(
        database: &'a MessagesDatabase,
        source_file_key: KeySymbol,
        output: &'a mut W,
    ) -> Self {
        Self {
            database,
            source_file_key,
            output,
        }
    }

    fn make_doc_comment(
        &self,
        message: &Message,
        known_locales: &FxHashSet<KeySymbol>,
    ) -> anyhow::Result<String> {
        if message.is_defined() {
            self.make_normal_message_doc_comment(message, known_locales)
        } else {
            self.make_undefined_message_doc_comment(message, known_locales)
        }
    }

    fn make_normal_message_doc_comment(
        &self,
        message: &Message,
        known_locales: &FxHashSet<KeySymbol>,
    ) -> anyhow::Result<String> {
        let key = message.hashed_key();
        // SAFETY: The caller asserts that this message is defined, so it must have a source.
        let default_translation = message.get_source_translation().unwrap();
        let spurious_variables = self.build_spurious_variables_info(message)?;
        let translation_links =
            self.build_translation_links(message.translations(), Some(default_translation))?;

        let found_locales: FxHashSet<KeySymbol> =
            message.translations().keys().map(Clone::clone).collect();

        let missing_locales = {
            let diff = known_locales.difference(&found_locales);
            let symbol_store = read_global_symbol_store()?;
            let mut missing_names = vec![];
            for key in diff {
                missing_names.push(
                    symbol_store
                        .resolve(*key)
                        .map(|locale| format!("`{locale}`"))
                        .ok_or(MessagesError::SymbolNotFound(*key))?,
                );
            }
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
            "   * ```\n   * {}\n   * ```\n",
            default_translation.raw,
        )?;

        if spurious_variables.len() > 0 {
            write!(
                result,
                "   * Spurious variables from translations:\n   * - {}\n",
                spurious_variables.join("\n   * - ")
            )?;
        }

        write!(result, "   */")?;

        Ok(result)
    }

    fn build_translation_links(
        &self,
        translations: &KeySymbolMap<MessageValue>,
        default_translation: Option<&MessageValue>,
    ) -> anyhow::Result<Vec<String>> {
        let symbol_store = read_global_symbol_store()?;

        let mut links = Vec::with_capacity(translations.len() - 1);

        for (key, translation) in translations {
            if default_translation.is_some_and(|t| t == translation) {
                continue;
            }
            let locale = symbol_store
                .resolve(*key)
                .ok_or(MessagesError::SymbolNotFound(*key))?;
            // let file = symbol_store
            //     .resolve(translation.file())
            //     .ok_or(MessagesError::SymbolNotFound(*key))?;
            // let link = format!("[{}]({})", locale, file);
            links.push(locale.into())
        }

        Ok(links)
    }

    /// Return a list of variables that are only present in non-default translations of the given
    /// `message`. Each entry is a pre-formatted String containing the variable name and the list
    /// of locales that contain it.
    fn build_spurious_variables_info(&self, message: &Message) -> anyhow::Result<Vec<String>> {
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
        let mut spurious_variables: KeySymbolMap<FxHashSet<KeySymbol>> = KeySymbolMap::default();

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
                        spurious_variables.insert(*variable, FxHashSet::default());
                    }
                    spurious_variables
                        .get_mut(variable)
                        .unwrap()
                        .insert(*locale_key);
                }
            }
        }

        let symbol_store = read_global_symbol_store()?;
        let mut lines = vec![];
        for (variable_key, locale_keys) in spurious_variables {
            let mut locales = vec![];
            for key in locale_keys {
                locales.push(
                    symbol_store
                        .resolve(key)
                        .ok_or(MessagesError::SymbolNotFound(key))?,
                );
            }

            let variable_name = symbol_store
                .resolve(variable_key)
                .ok_or(MessagesError::SymbolNotFound(variable_key))?;

            lines.push(format!("`{variable_name}` -- {}", locales.join(", ")));
        }
        Ok(lines)
    }

    fn make_undefined_message_doc_comment(
        &self,
        message: &Message,
        _known_locales: &FxHashSet<KeySymbol>,
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

    fn make_getter_type_def(&self, message: &Message) -> anyhow::Result<String> {
        let symbol_store = read_global_symbol_store()?;

        let name = message.key();
        let variables = message.all_variables();
        let mut entries = vec![];
        for (name, instances) in variables.iter() {
            let name = symbol_store
                .resolve(*name)
                .map(String::from)
                .ok_or(MessagesError::SymbolNotFound(*name))?;

            let type_names = instances
                .iter()
                .map(|instance| get_variable_type_name(&instance.kind))
                .collect::<FxHashSet<&str>>();
            let type_str = Vec::from_iter(type_names).join(" | ");
            entries.push(format!("{}: {}", name, type_str));
        }

        if entries.is_empty() {
            Ok(format!("  '{name}': TypedIntlMessageGetter<undefined>,"))
        } else {
            Ok(format!(
                "  '{name}': TypedIntlMessageGetter<{{{}}}>,",
                entries.join(", ")
            ))
        }
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
    }
}

/// Returns the set of message keys as a list, sorted alphabetically by the key's resolved value.
/// This is annoyingly inefficient with two vector allocations, but it works.
fn get_sorted_message_keys(keys: &FxHashSet<KeySymbol>) -> anyhow::Result<Vec<&KeySymbol>> {
    let symbol_store = read_global_symbol_store()?;

    let mut sorted_keys = Vec::with_capacity(keys.len());
    for key in keys {
        sorted_keys.push((
            key,
            symbol_store
                .resolve(*key)
                .ok_or(MessagesError::SymbolNotFound(*key))?,
        ));
    }
    sorted_keys.sort_by_key(|(_, name)| name.to_owned());
    Ok(sorted_keys.into_iter().map(|(key, _)| key).collect())
}

#[derive(Debug, Error)]
pub enum IntlTypesGeneratorError {
    #[error("Requested source file '{0}' does not exist.")]
    SourceFileNotFound(KeySymbol),
    #[error("Message key '{0}' from source file '{1}' does not exist in the database.")]
    SourceFileMessageNotFound(KeySymbol, KeySymbol),
}

impl<W: std::io::Write> IntlService for IntlTypesGenerator<'_, W> {
    type Result = anyhow::Result<()>;

    fn run(&mut self) -> Self::Result {
        write!(
            self.output,
            "/* THIS FILE IS AUTOGENERATED. DO NOT EDIT MANUALLY. */

import {{TypedIntlMessageGetter}} from '{}';

type LinkFunction = (content: any[]) => React.ReactNode;
type HookFunction = (content: any[]) => React.ReactNode;

declare const messages: {{
",
            intl_message_utils::RUNTIME_PACKAGE_NAME
        )?;

        let known_locales = &self.database.known_locales;
        let source_file = self.database.sources.get(&self.source_file_key).ok_or(
            IntlTypesGeneratorError::SourceFileNotFound(self.source_file_key),
        )?;

        let source_message_keys = get_sorted_message_keys(source_file.message_keys())?;
        for message_key in source_message_keys {
            let Some(message) = self.database.messages.get(message_key) else {
                return Err(IntlTypesGeneratorError::SourceFileMessageNotFound(
                    *message_key,
                    self.source_file_key,
                )
                .into());
            };

            let doc_comment = self.make_doc_comment(message, known_locales)?;
            let type_def = self.make_getter_type_def(message)?;
            write!(self.output, "{doc_comment}\n{type_def}\n")?;
        }

        write!(self.output, "}};\nexport default messages;")?;

        Ok(())
    }
}
