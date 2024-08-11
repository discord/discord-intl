const path = require('node:path');
const fs = require('node:fs');
const { isMessageTranslationsFile } = require('@discord/intl-message-database');

const IGNORED_MESSAGE_FILE_PATTERNS = [/.*\.compiled.messages\..*/];

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
      if (IGNORED_MESSAGE_FILE_PATTERNS.some((pattern) => pattern.test(filePath))) continue;

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

module.exports = {
  IGNORED_MESSAGE_FILE_PATTERNS,
  findAllTranslationFiles,
  getLocaleFromTranslationsFileName,
};
