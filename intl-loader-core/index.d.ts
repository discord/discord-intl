export {
  hashMessageKey,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  IntlCompiledMessageFormat,
} from '@discord/intl-message-database';

export { database } from './src/database';
export {
  generateTypeDefinitions,
  processDefinitionsFile,
  processTranslationsFile,
  precompileFileForLocale,
} from './src/processing';
export { MessageDefinitionsTransformer } from './src/transformer';
export { findAllTranslationFiles, getLocaleFromTranslationsFileName } from './src/util';
export * as watcher from './src/watcher';
