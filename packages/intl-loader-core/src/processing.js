const path = require('node:path');

const debug = require('debug')('intl:loader-core');
const { IntlCompiledMessageFormat } = require('@discord/intl-message-database');

const { database } = require('./database');
const { findAllTranslationFiles, getLocaleFromTranslationsFileName } = require('./util');

/**
 * @param {string} sourcePath
 * @param {import('@discord/intl-message-database').IntlSourceFile} sourceFile
 */
function debugSourceFile(sourcePath, sourceFile) {
  debug(
    `[${sourcePath}] Parsed messages file: type=${sourceFile.type}, locale=${sourceFile.locale}, messageCount=${sourceFile.messageKeys.length}, meta=${JSON.stringify(sourceFile.meta)}`,
  );
}

/**
 * @param {string} sourcePath
 * @param {string=} sourceContent
 * @param {{
 *   processTranslations?: boolean,
 *   locale?: string
 * }=} options
 * @returns {import('./types').ProcessDefinitionsResult}
 */
function processDefinitionsFile(sourcePath, sourceContent, options = {}) {
  const {
    processTranslations = false,
    // TODO: Make this more configurable/automatically determined.
    locale = 'en-US',
  } = options;
  debug(`[${sourcePath}] Processing definitions with locale "${locale}"`);

  if (sourceContent != null) {
    database.processDefinitionsFileContent(sourcePath, sourceContent);
  } else {
    database.processDefinitionsFile(sourcePath);
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
  let translationsLocaleMap = findAllTranslationFiles(translationsPath);
  if (translationsLocaleMap instanceof Error) {
    debug(
      `[${sourcePath}] Failed to build translations locale map: [${translationsLocaleMap.name}] ${translationsLocaleMap.message}`,
    );
    translationsLocaleMap = {};
  }

  if (processTranslations) {
    database.processAllTranslationFiles(translationsLocaleMap);
  }

  return {
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
 * }=} options
 * @returns {import('./types').ProcessTranslationsResult}
 */
function processTranslationsFile(sourcePath, sourceContent, options = {}) {
  const { locale = getLocaleFromTranslationsFileName(sourcePath) } = options;
  if (sourceContent) {
    database.processTranslationFileContent(sourcePath, locale, sourceContent);
  } else {
    database.processTranslationFile(sourcePath, locale);
  }

  const sourceFile = database.getSourceFile(sourcePath);
  debugSourceFile(sourcePath, sourceFile);
  if (sourceFile.type !== 'translation') {
    throw new Error(
      `Expected ${sourcePath} to be a message translations file, but it resulted in ${sourceFile.type} instead.`,
    );
  }

  return {
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
 * @param {string} sourcePath
 * @param {string} locale
 * @param {{
 *   format?: IntlCompiledMessageFormat,
 *   outputFile?: string
 * }=} options
 *
 * @returns {Buffer | void}
 */
function precompileFileForLocale(sourcePath, locale, options = {}) {
  const { format = IntlCompiledMessageFormat.KeylessJson, outputFile } = options;
  return outputFile != null
    ? database.precompile(sourcePath, locale, outputFile, format)
    : database.precompileToBuffer(sourcePath, locale, format);
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
 * If `allowNullability` is set, the generated types for variables within messages will allow
 * `null` and `undefined` for most value types, as well as looser restrictions on typing, such as
 * allowing `string | number` for number variables.
 *
 * Returns `true` if the types were successfully generated, or `false` otherwise, such as if the
 * source file is not already in the database.
 *
 * @param {string} sourcePath
 * @param {string=} outputFile
 * @param {boolean=} allowNullability
 * @returns {boolean}
 */
function generateTypeDefinitions(sourcePath, outputFile, allowNullability = false) {
  const paths = database.getAllSourceFilePaths();
  if (!paths.includes(sourcePath)) return false;

  database.generateTypes(
    sourcePath,
    outputFile ?? sourcePath.replace(/\.[^.]+$/, '.d.ts'),
    allowNullability,
  );
  return true;
}

module.exports = {
  generateTypeDefinitions,
  precompileFileForLocale,
  processDefinitionsFile,
  processTranslationsFile,
};
