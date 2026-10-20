use std::path::PathBuf;

use discord_intl_database_core::MessagesDatabase;
use discord_intl_database_service::IntlDatabaseService;

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
pub struct DumpMessagesCsv<'a> {
    database: &'a MessagesDatabase,
    output_directory: PathBuf,
}

impl<'a> DumpMessagesCsv<'a> {
    pub fn new(database: &'a MessagesDatabase, output_directory: PathBuf) -> Self {
        Self {
            database,
            output_directory,
        }
    }
}

pub struct CsvMessageFiles {
    pub definitions: PathBuf,
    pub translations: Vec<PathBuf>,
}

const DEFINITIONS_CSV_HEADERS: &[&str] = &[
    "key",
    "content",
    "locale",
    "filePath",
    "line",
    "col",
    "description",
    "is_secret",
    "is_translation",
];
const TRANSLATIONS_CSV_HEADERS: &[&str] = &["key", "content", "locale", "filePath", "line", "col"];

impl IntlDatabaseService for DumpMessagesCsv<'_> {
    type Result = anyhow::Result<CsvMessageFiles>;

    fn run(&mut self) -> Self::Result {
        std::fs::create_dir_all(&self.output_directory)?;

        let mut sorted_keys = self.database.messages.keys().collect::<Vec<_>>();
        sorted_keys.sort();

        let definitions_path = self.output_directory.join("definitions.csv");
        let mut definitions_writer = csv::Writer::from_path(&definitions_path)?;
        definitions_writer.write_record(DEFINITIONS_CSV_HEADERS)?;
        for key in sorted_keys.iter() {
            let message = self.database.messages.get(key).unwrap();
            let Some(value) = message.get_source_translation() else {
                continue;
            };
            let locale = message
                .source_locale()
                .expect("defined message to have source locale");
            let meta = message.meta();
            definitions_writer.write_record(&[
                key.as_str(),
                &value.raw,
                locale.as_str(),
                value.file_position.file.as_str(),
                &value.file_position.line.to_string(),
                &value.file_position.col.to_string(),
                &meta.description.as_ref().unwrap_or(&String::default()),
                &meta.secret.to_string(),
                &meta.translate.to_string(),
            ])?;
        }

        let mut translation_files = vec![];
        for locale in self.database.known_locales.iter() {
            let translations_path = self.output_directory.join(locale).with_extension("csv");
            let mut translations_writer = csv::Writer::from_path(&translations_path)?;
            translations_writer.write_record(TRANSLATIONS_CSV_HEADERS)?;
            for key in sorted_keys.iter() {
                let message = self.database.messages.get(key).unwrap();
                let Some(value) = message.translations().get(&locale) else {
                    continue;
                };
                translations_writer.write_record(&[
                    key.as_str(),
                    &value.raw,
                    locale.as_str(),
                    value.file_position.file.as_str(),
                    &value.file_position.line.to_string(),
                    &value.file_position.col.to_string(),
                ])?;
            }
            translation_files.push(translations_path);
        }

        Ok(CsvMessageFiles {
            definitions: definitions_path,
            translations: translation_files,
        })
    }
}
