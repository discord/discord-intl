const {
  hashMessageKey,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  IntlCompiledMessageFormat,
  IntlDatabaseInsertStrategy,
} = require('@discord/intl-message-database');

const { database } = require('./src/database');
const {
  findAllDefinitionsFilesForTranslations,
  findAllMessagesFiles,
  filterAllMessagesFiles,
  processAllMessagesFiles,
  generateTypeDefinitions,
  processDefinitionsFile,
  processTranslationsFile,
  precompileFileForLocale,
} = require('./src/processing');
const { MessageDefinitionsTransformer } = require('./src/transformer');
const { findAllTranslationFiles, getLocaleFromTranslationsFileName } = require('./src/util');
const watcher = require('./src/watcher');

module.exports = {
  // @ts-expect-error This is a const enum, which TypeScript doesn't like letting you export, even
  // though it's a tangible object that can be accessed just fine from normal JS.
  IntlCompiledMessageFormat,
  // @ts-expect-error This is a const enum, which TypeScript doesn't like letting you export, even
  // though it's a tangible object that can be accessed just fine from normal JS.
  IntlDatabaseInsertStrategy,
  MessageDefinitionsTransformer,
  database,
  findAllTranslationFiles,
  findAllDefinitionsFilesForTranslations,
  getLocaleFromTranslationsFileName,
  generateTypeDefinitions,
  hashMessageKey,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  processTranslationsFile,
  precompileFileForLocale,
  findAllMessagesFiles,
  filterAllMessagesFiles,
  processAllMessagesFiles,
  watcher,
};
