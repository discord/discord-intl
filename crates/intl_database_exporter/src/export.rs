use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use intl_database_core::{DEFAULT_LOCALE, KeySymbol, MessagesDatabase, SourceFile};
use intl_database_service::IntlDatabaseService;
use rustc_hash::FxHashMap;

/// A service for persisting the current contents of a [MessagesDatabase] into a set of translation
/// files, organized according to the configuration of each message's meta information for where
/// each of its translations should be stored. Because messages can share translation files across
/// multiple definition files, this process operates on the entire database at once, ensuring every
/// message is included in each file when it is returned, without clobbering messages from other
/// sources.
///
/// The result of this service is the list of file paths that were written. After running, that list
/// can be used by the consumer to prune any extra files in the project, check for empty values to
/// save on total file counts, or do any other operation with full confidence that all translations
/// for messages in the database are covered by those files.
///
/// Importantly, _only_ translations are processed by this export, source messages and definition
/// files are _not_ included, since those always come from a different format.
pub struct ExportTranslations<'a> {
    database: &'a MessagesDatabase,
    file_extension: String,
}

impl<'a> ExportTranslations<'a> {
    pub fn new(database: &'a MessagesDatabase, file_extension: Option<String>) -> Self {
        Self {
            database,
            file_extension: file_extension.unwrap_or("messages.json".into()),
        }
    }
}

impl IntlDatabaseService for ExportTranslations<'_> {
    type Result = anyhow::Result<Vec<String>>;

    fn run(&mut self) -> Self::Result {
        let definition_files = self
            .database
            .sources
            .values()
            .filter_map(|source| match source {
                SourceFile::Definition(definition) => Some(definition),
                _ => None,
            });

        let mut result: FxHashMap<PathBuf, BTreeMap<KeySymbol, &String>> = FxHashMap::default();
        for file in definition_files {
            for locale in &self.database.known_locales {
                // TODO: Make DEFAULT_LOCALE configurable. This assumes all definitions are
                // in the default locale, but it's possible for definitions to use a different
                // locale as the source, and this is arguably something that can be set per-message.
                if *locale == DEFAULT_LOCALE {
                    continue;
                }

                let path = file.meta().get_translations_path(&locale, None);
                let values = result.entry(path).or_default();
                for key in file.message_keys() {
                    let Some(message) = self.database.get_message(&key) else {
                        continue;
                    };
                    if message
                        .source_locale()
                        .is_some_and(|source| source == *locale)
                    {
                        continue;
                    }
                    let Some(value) = self
                        .database
                        .get_message(&key)
                        .and_then(|message| message.translations().get(&locale))
                    else {
                        continue;
                    };

                    values.insert(*key, &value.raw);
                }
            }
        }

        let mut affected_files = vec![];

        for (file, values) in result {
            let path = file.with_extension(&self.file_extension);
            affected_files.push(path.display().to_string());

            if let Some(directory) = path.parent() {
                std::fs::create_dir_all(directory)?;
            }

            let content = serde_json::to_string_pretty(&values)?;
            let mut output = std::fs::File::create(path)?;
            output.write_all(content.as_bytes())?;
        }

        Ok(affected_files)
    }
}
