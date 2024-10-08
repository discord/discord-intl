use std::fmt::Write;

use rustc_hash::FxHashSet;
use thiserror::Error;
use ustr::Ustr;

use intl_database_core::{
    KeySymbol, KeySymbolMap, KeySymbolSet, Message, MessagesDatabase, MessageValue,
    MessageVariableType,
};
use intl_database_service::IntlDatabaseService;

pub struct IntlTypesGenerator<'a, W: std::io::Write> {
    database: &'a MessagesDatabase,
    source_file_key: KeySymbol,
    output: &'a mut W,
    allow_nullability: bool,
}

impl<'a, W: std::io::Write> IntlTypesGenerator<'a, W> {
    pub fn new(
        database: &'a MessagesDatabase,
        source_file_key: KeySymbol,
        output: &'a mut W,
        allow_nullability: bool,
    ) -> Self {
        Self {
            database,
            source_file_key,
            output,
            allow_nullability,
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
            "   * ```\n   * {}\n   * ```\n",
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

        if entries.is_empty() {
            Ok(format!("  '{name}': TypedIntlMessageGetter<undefined>,"))
        } else {
            entries.sort();
            Ok(format!(
                "  '{name}': TypedIntlMessageGetter<{{{}}}>,",
                entries.join(", ")
            ))
        }
    }
}

fn get_variable_type_name(kind: &MessageVariableType) -> &str {
    // TODO: All of these undefined unions are technically incorrect and should
    // be handled on the consuming side somehow.
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
            write!(self.output, "{doc_comment}\n{type_def}\n")?;
        }

        write!(self.output, "}};\nexport default messages;")?;

        Ok(())
    }
}
