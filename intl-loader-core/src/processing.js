const path = require('node:path');

const { IntlCompiledMessageFormat } = require('@discord/intl-message-database');

const { database } = require('./database');
const { findAllTranslationFiles, getLocaleFromTranslationsFileName } = require('./util');

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
  if (sourceContent != null) {
    database.processDefinitionsFileContent(sourcePath, sourceContent);
  } else {
    database.processDefinitionsFile(sourcePath);
  }

  const sourceFile = database.getSourceFile(sourcePath);
  if (sourceFile.type !== 'definition') {
    throw new Error(
      `Expected ${sourcePath} to be a message definitions file, but it resulted in ${sourceFile.type} instead.`,
    );
  }

  const hashedMessageKeys = database.getSourceFileHashedKeys(sourcePath);
  const translationsPath = path.resolve(path.dirname(sourcePath), sourceFile.meta.translationsPath);
  const translationsLocaleMap = findAllTranslationFiles(translationsPath);

  if (processTranslations) {
    database.processAllTranslationFiles(translationsLocaleMap);
  }

  return {
    sourceFile,
    locale,
    hashedMessageKeys,
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
 *   outputFile?: string
 * }=} options
 * @returns {import('./types').ProcessTranslationsResult}
 */
function processTranslationsFile(sourcePath, sourceContent, options = {}) {
  const { locale = getLocaleFromTranslationsFileName(sourcePath), outputFile } = options;
  if (sourceContent) {
    database.processTranslationFileContent(sourcePath, locale, sourceContent);
  } else {
    database.processTranslationFile(sourcePath, locale);
  }

  const sourceFile = database.getSourceFile(sourcePath);
  if (sourceFile.type !== 'translation') {
    throw new Error(
      `Expected ${sourcePath} to be a message translations file, but it resulted in ${sourceFile.type} instead.`,
    );
  }

  return {
    sourceFile,
    locale,
    hashedMessageKeys: database.getSourceFileHashedKeys(sourcePath),
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
  const { format = IntlCompiledMessageFormat.Json, outputFile } = options;
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

  database.generateTypes(sourcePath, outputFile ?? sourcePath.replace(/\.[^.]+/, '.d.ts'));
  return true;
}

module.exports = {
  generateTypeDefinitions,
  precompileFileForLocale,
  processDefinitionsFile,
  processTranslationsFile,
};
