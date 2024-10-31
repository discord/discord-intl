const {
  hashMessageKey,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  IntlCompiledMessageFormat,
} = require('@discord/intl-message-database');

const { database } = require('./src/database');
const {
  findAllMessagesFiles,
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
  MessageDefinitionsTransformer,
  database,
  findAllTranslationFiles,
  getLocaleFromTranslationsFileName,
  generateTypeDefinitions,
  hashMessageKey,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  processTranslationsFile,
  precompileFileForLocale,
  findAllMessagesFiles,
  processAllMessagesFiles,
  watcher,
};
