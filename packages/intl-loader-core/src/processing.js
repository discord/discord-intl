const path = require('node:path');

const debug = require('debug')('intl:loader-core');
const {
  IntlCompiledMessageFormat,
  IntlDatabaseInsertStrategy,
} = require('@discord/intl-message-database');

const { database } = require('./database');
const { findAllTranslationFiles, getLocaleFromTranslationsFileName } = require('./util');

/**
 * @import {
 *   IntlMessagesFileDescriptor,
 *   IntlSourceFileInsertionData,
 *   ProcessDefinitionsResult,
 *   ProcessTranslationsResult
 * } from '../types';
 *
 * @typedef {{
 *   format?: IntlCompiledMessageFormat,
 *   bundleSecrets?: boolean,
 * }} IntlPrecompileOptions
 */

/**
 * @param {string} sourcePath
 * @param {import('../types').IntlSourceFile} sourceFile
 */
function debugSourceFile(sourcePath, sourceFile) {
  debug(
    `[${sourcePath}] Parsed messages file: type=${sourceFile.type}, locale=${sourceFile.locale}, messageCount=${sourceFile.messageKeys.length}, meta=${JSON.stringify(sourceFile.meta)}`,
  );
}

/**
 *
 * @param {string} sourcePath Path of the source file being processed, for debug logging
 * @param {import('../types').IntlSourceFile} sourceFile SourceFile object from the database used to find translations.
 * @param {string} translationsPath Fully-resolved path to the directory for translations.
 * @returns {Record<string, string>}
 */
function buildTranslationsLocaleMap(sourcePath, sourceFile, translationsPath) {
  if (sourceFile.meta.translate === false) {
    debug(`[${sourcePath}] translate is set to false, no locale map is needed`);
    return {};
  }
  const map = findAllTranslationFiles(translationsPath);
  if (map instanceof Error) {
    debug(`[${sourcePath}] Failed to build locale map: [${map.name}] ${map.message}`);
    return {};
  }
  return map;
}

/**
 * Return file paths for all definitions files with a translations path meta value that would
 * include the given `translationPath`. This can be used to add reverse dependencies, and safely
 * cache translations files with appropriate change detection.
 *
 * Note that this process is _only_ accurate if the database has full knowledge of all messages
 * files that exist in the project, which can be guaranteed by running `findAllMessagesFiles` and
 * `processAllMessagesFiles` beforehand. In most cases, this is suitable to do only once when a
 * loader process is initializing.
 *
 * @param {string} filePath
 * @returns {string[]}
 */
function findAllDefinitionsFilesForTranslations(filePath) {
  const expectedTranslationsPath = path.dirname(filePath);
  const sourceFiles = database.getAllSourceFilePaths();
  const relevantPaths = [];
  for (const file of sourceFiles) {
    const source = database.getSourceFile(file);
    if (source.type === 'definition' && source.meta.translationsPath === expectedTranslationsPath) {
      relevantPaths.push(source.file);
    }
  }
  return relevantPaths;
}

/**
 * Scan the entire file system within the given `directories` to find all files that can be treated
 * as messages definitions _or_ translations.
 *
 * @param {string[]} directories
 * @param {string} defaultLocale
 * @returns {import('../types').IntlMessagesFileDescriptor[]}
 */
function findAllMessagesFiles(directories, defaultLocale = 'en-US') {
  return database.findAllMessagesFiles(directories, defaultLocale);
}

/**
 * Given an arbitrary list of `files`, keep only those that can be treated as messages files, either
 * definitions or translations. The returned list is a set of descriptors that can be processed by
 * `processAllMessagesFiles`.
 *
 * @param {string[]} files
 * @param {string} defaultLocale
 * @returns {IntlMessagesFileDescriptor[]}
 */
function filterAllMessagesFiles(files, defaultLocale = 'en-US') {
  return database.filterAllMessagesFiles(files, defaultLocale);
}

/**
 * Iterate the given `files`, processing each one's content into the database. Processing is done
 * in parallel using native acceleration, and returns an object describing all of the changes that
 * were processed.
 *
 * Note that this method _only_ operates by reading file contents from the system, it is not
 * possible to supply preloaded content through a Buffer in the same way as `processDefinitionsFile`
 * with a `sourceContent` argument.
 *
 * @param {IntlMessagesFileDescriptor[]} files
 * @param {IntlDatabaseInsertStrategy} [strategy]
 * @returns {IntlSourceFileInsertionData[]}
 */
function processAllMessagesFiles(files, strategy = IntlDatabaseInsertStrategy.Update) {
  return database.processAllMessagesFiles(files, strategy);
}

/**
 * @param {string} sourcePath
 * @param {string=} sourceContent
 * @param {{
 *   processTranslations?: boolean,
 *   locale?: string
 *   strategy?: IntlDatabaseInsertStrategy
 * }=} options
 * @returns {ProcessDefinitionsResult}
 */
function processDefinitionsFile(sourcePath, sourceContent, options = {}) {
  const {
    processTranslations = false,
    // TODO: Make this more configurable/automatically determined.
    locale = 'en-US',
    strategy = IntlDatabaseInsertStrategy.Update,
  } = options;
  debug(`[${sourcePath}] Processing definitions with locale "${locale}"`);

  /** @type {IntlSourceFileInsertionData} */
  let result;
  if (sourceContent != null) {
    result = database.processDefinitionsFileContent(sourcePath, sourceContent, locale, strategy);
  } else {
    result = database.processDefinitionsFile(sourcePath, locale, strategy);
  }
  if (result.errors.length > 0) {
    return { succeeded: false, errors: result.errors };
  }

  const sourceFile = database.getSourceFile(sourcePath);
  debugSourceFile(sourcePath, sourceFile);
  if (sourceFile.type !== 'definition') {
    throw new Error(
      `Expected ${sourcePath} to be a message definitions file, but it resulted in ${sourceFile.type} instead.`,
    );
  }

  const messageKeys = database.getSourceFileKeyMap(sourcePath);
  const translationsPath = path.resolve(path.dirname(sourcePath), sourceFile.meta.translationsPath);
  const translationsLocaleMap = buildTranslationsLocaleMap(
    sourcePath,
    sourceFile,
    translationsPath,
  );

  if (processTranslations) {
    const translationsResult = database.processAllTranslationFiles(translationsLocaleMap, strategy);
    const allErrors = translationsResult.flatMap((result) => result.errors);
    if (allErrors.length > 0) {
      return { succeeded: false, errors: allErrors };
    }
    debug('[${sourcePath}] Finished processing all translations: %O', translationsResult);
  }

  return {
    succeeded: true,
    sourceFile,
    locale,
    messageKeys,
    translationsPath,
    translationsLocaleMap,
  };
}

/**
 *
 * @param {string} sourcePath
 * @param {string=} sourceContent
 * @param {{
 *   locale?: string,
 *   strategy?: IntlDatabaseInsertStrategy
 * }=} options
 * @returns {ProcessTranslationsResult}
 */
function processTranslationsFile(sourcePath, sourceContent, options = {}) {
  const {
    locale = getLocaleFromTranslationsFileName(sourcePath),
    strategy = IntlDatabaseInsertStrategy.Update,
  } = options;

  /** @type {IntlSourceFileInsertionData} */
  let result;
  if (sourceContent) {
    result = database.processTranslationFileContent(sourcePath, locale, sourceContent, strategy);
  } else {
    result = database.processTranslationFile(sourcePath, locale, strategy);
  }
  if (result.errors.length > 0) {
    return { succeeded: false, errors: result.errors };
  }

  const sourceFile = database.getSourceFile(sourcePath);
  debugSourceFile(sourcePath, sourceFile);
  if (sourceFile.type !== 'translation') {
    throw new Error(
      `Expected ${sourcePath} to be a message translations file, but it resulted in ${sourceFile.type} instead.`,
    );
  }

  return {
    succeeded: true,
    sourceFile,
    locale,
    messageKeys: database.getSourceFileKeyMap(sourcePath),
  };
}

/**
 * Precompile the messages defined in the given `sourcePath` using the value of the translation for
 * that message in the given `locale`. `format` specifies which serialization format the result
 * will be written in.
 *
 * By default, the compiled content will be returned as a Buffer containing the serialized string,
 * but if `outputFile` is given then the content will be written directly to the file and the
 * function becomes `void`.
 *
 * Compiling automatically handles filtering out messages based on the meta information like
 * `translate`, `secret`, and `bundleSecrets`, to ensure that all consumers apply these values
 * accurately and consistently.
 *
 * @param {string} sourcePath
 * @param {string} locale
 * @param {string=} outputFile
 * @param {IntlPrecompileOptions} [options]
 *
 * @returns {Buffer | void}
 */
function precompileFileForLocale(sourcePath, locale, outputFile, options = {}) {
  return outputFile != null
    ? database.precompile(sourcePath, locale, outputFile, options)
    : database.precompileToBuffer(sourcePath, locale, options);
}

/**
 * Generate a `.d.ts` file containing TypeScript type definitions for all of the messages defined in
 * `sourcePath`. This method does not process `sourcePath` at all, meaning it expects the database
 * to already know about the source, as well as all of the related translations to create an
 * accurate typescript definition for each message.
 *
 * If not given, `outputFile` will default to the same path as `sourcePath`, with the last extension
 * replaced by `.d.ts`. For example, a file like `SomeMessages.Other.messages.js` would become
 * `SomeMessages.Other.messages.d.ts`.
 *
 * Returns `true` if the types were successfully generated, or `false` otherwise, such as if the
 * source file is not already in the database.
 *
 * @param {string} sourcePath
 * @param {string=} outputFile
 * @returns {boolean}
 */
function generateTypeDefinitions(sourcePath, outputFile) {
  const paths = database.getAllSourceFilePaths();
  if (!paths.includes(sourcePath)) return false;

  database.generateTypes(sourcePath, outputFile ?? sourcePath.replace(/\.[^.]+$/, '.d.ts'));
  return true;
}

module.exports = {
  findAllDefinitionsFilesForTranslations,
  findAllMessagesFiles,
  filterAllMessagesFiles,
  processAllMessagesFiles,
  generateTypeDefinitions,
  precompileFileForLocale,
  processDefinitionsFile,
  processTranslationsFile,
};
