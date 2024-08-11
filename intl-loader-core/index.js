const { database } = require('./src/database');
const { MessageDefinitionsTransformer } = require('./src/transformer');
const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  IntlCompiledMessageFormat,
} = require('@discord/intl-message-database');
const path = require('node:path');
const fs = require('node:fs');

const IGNORED_MESSAGE_PATTERNS = [/.*\.compiled.messages\..*/];

/**
 * Return the presumed locale for a translations file from it's name. The convention follows the
 * format: `some/path/to/<locale>.messages.jsona`, so the locale is determined by taking the content
 * of the basename up until the first `.`.
 *
 * @param {string} fileName
 * @returns {string}
 */
function getLocaleFromTranslationsFileName(fileName) {
  return path.basename(fileName).split('.')[0];
}

/**
 * Scan the given `translationsPath` to discover all translation files that exist, returning them
 * as a map from locale name to the path for importing.
 *
 * @param {string} translationsPath
 * @returns {Record<string, string>}
 */
function findAllTranslationFiles(translationsPath) {
  /** @type {Record<string, string>} */
  const localeMap = {};

  try {
    const translationFiles = fs.readdirSync(translationsPath, { encoding: 'utf-8' });
    for (const foundFile of translationFiles) {
      const filePath = path.join(translationsPath, foundFile);
      // Only include translation files, not definitions files.
      if (!isMessageTranslationsFile(filePath)) continue;
      // Some files are excluded, like pre-compiled artifacts.
      if (IGNORED_MESSAGE_PATTERNS.some((pattern) => pattern.test(filePath))) continue;

      const locale = getLocaleFromTranslationsFileName(filePath);
      localeMap[locale] = filePath;
    }
  } catch (e) {
    throw new Error(
      `The translations directory ${translationsPath} was not found. No translations will be loaded for these messages`,
    );
  }

  return localeMap;
}

/**
 * @param {string} sourcePath
 * @param {string=} sourceContent
 * @param {{
 *   processTranslations?: boolean
 * }=} options
 * @returns {import('./src/types').ProcessDefinitionsResult}
 */
function processDefinitionsFile(sourcePath, sourceContent, options = {}) {
  const { processTranslations = false } = options;
  if (sourceContent != null) {
    database.processDefinitionsFileContent(sourcePath, sourceContent);
  } else {
    database.processDefinitionsFile(sourcePath);
  }

  const sourceFile = database.getSourceFile(sourcePath);
  const hashedMessageKeys = database.getSourceFileHashedKeys(sourcePath);
  const translationsPath = path.resolve(path.dirname(sourcePath), sourceFile.meta.translationsPath);
  const translationsLocaleMap = findAllTranslationFiles(translationsPath);

  if (processTranslations) {
    database.processAllTranslationFiles(translationsLocaleMap);
  }

  return {
    sourceFile,
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
 * @returns {import('./src/types').ProcessTranslationsResult}
 */
function processTranslationsFile(sourcePath, sourceContent, options = {}) {
  const { locale = getLocaleFromTranslationsFileName(sourcePath), outputFile } = options;
  if (sourceContent) {
    database.processTranslationFileContent(sourcePath, locale, sourceContent);
  } else {
    database.processTranslationFile(sourcePath, locale);
  }

  return {
    sourceFile: database.getSourceFile(sourcePath),
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
  // @ts-expect-error This is a const enum, which TypeScript doesn't like letting you export, even
  // though it's a tangible object that can be accessed just fine from normal JS.
  IntlCompiledMessageFormat,
  MessageDefinitionsTransformer,
  database,
  findAllTranslationFiles,
  getLocaleFromTranslationsFileName,
  generateTypeDefinitions,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  processTranslationsFile,
  precompileFileForLocale,
};
