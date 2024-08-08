const fs = require('node:fs');
const path = require('node:path');
const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
} = require('@discord/intl-message-database');

const { database } = require('./src/database');
const { MessageDefinitionsTransformer } = require('./src/transformer');

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
 * Take in a file that contains message definitions (e.g., calls to
 * `defineMessages`), extract the pieces necessary for the runtime i18n code
 * to use (only the string id), and transform the module to only include a
 * default export of that content.
 *
 * @param {string} source
 * @this {import('webpack').LoaderContext<{}>}
 */
const intlLoader = function intlLoader(source) {
  const sourcePath = this.resourcePath;
  const forceTranslation = this.resourceQuery === '?forceTranslation';

  if (isMessageDefinitionsFile(sourcePath) && !forceTranslation) {
    database.processDefinitionsFileContent(sourcePath, source);
    const sourceFile = database.getSourceFile(sourcePath);
    if (sourceFile.type !== 'definition') {
      throw new Error(
        'Expected an intl messages definition file, but found a translation file instead.',
      );
    }

    const translationsPath = path.resolve(
      path.dirname(sourcePath),
      sourceFile.meta.translations_path,
    );

    /** @type {Record<string, string>} */
    const localeMap = {};
    try {
      const translationFiles = fs.readdirSync(translationsPath, { encoding: 'utf-8' });
      for (const foundFile of translationFiles) {
        const filePath = path.join(translationsPath, foundFile);
        if (!isMessageTranslationsFile(filePath)) continue;

        const locale = getLocaleFromTranslationsFileName(filePath);
        localeMap[locale] = filePath;
        this.addDependency(filePath);
      }
    } catch (e) {
      this.emitWarning(
        new Error(
          `The translations directory ${translationsPath} was not found. No translations will be loaded for these messages`,
        ),
      );
    }

    delete localeMap['en-US'];
    database.processAllTranslationFiles(localeMap);
    localeMap['en-US'] = sourcePath + '?forceTranslation';

    const transformer = new MessageDefinitionsTransformer(
      database.getSourceFileHashedKeys(sourcePath),
      localeMap,
    );
    return transformer.getOutput();
  } else {
    const localeName = forceTranslation ? 'en-US' : getLocaleFromTranslationsFileName(sourcePath);
    if (isMessageTranslationsFile(sourcePath)) {
      database.processTranslationFileContent(sourcePath, localeName, source);
    } else if (!forceTranslation) {
      throw new Error(
        'Expected a translation file or the `forceTranslation` query parameter on this import, but none was found',
      );
    }
    // Translations are still treated as JS files that need to be pre-parsed.
    // Rspack will handle parsing for the actual JSON file requests.
    if (forceTranslation) {
      return (
        'export default JSON.parse(' +
        JSON.stringify(database.precompileToBuffer(localeName).toString()) +
        ')'
      );
    } else {
      return database.precompileToBuffer(localeName);
    }
  }
};

module.exports = intlLoader;
