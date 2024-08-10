const path = require('node:path');
const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
} = require('@discord/intl-message-database');

const { database } = require('./src/database');
const { MessageDefinitionsTransformer } = require('./src/transformer');

/**
 * @param {{
 *  filename: string,
 *  src: string,
 *  getPrelude: () => string
 *  getTranslationAssetExtension: () => string,
 *  createAssetImport: (importPath: string) => string,
 * }} options
 * @returns {string | Buffer}
 */
function transformToString({
  filename,
  src,
  getPrelude,
  getTranslationAssetExtension,
  createAssetImport,
}) {
  if (isMessageDefinitionsFile(filename)) {
    database.processDefinitionsFileContent(filename, src);
    const sourceFile = database.getSourceFile(filename);
    if (sourceFile.type !== 'definition') {
      throw new Error(
        'Expected an intl messages definition file, but found a translation file instead.',
      );
    }

    const locales = database.getKnownLocales();
    /** @type {Record<string, string>} */
    const localeMap = {};
    for (const locale of locales) {
      localeMap[locale] =
        `${sourceFile.meta.translationsPath}/${locale}.${getTranslationAssetExtension()}`;
    }
    const fileBasename = filename.substring(0, filename.lastIndexOf('.messages.js'));
    localeMap['en-US'] = path.dirname(fileBasename + '.compiled.messages.jsona');

    const transformer = new MessageDefinitionsTransformer(
      database.getSourceFileHashedKeys(filename),
      localeMap,
      createAssetImport,
      getPrelude,
    );

    return transformer.getOutput();
  } else if (isMessageTranslationsFile(filename)) {
    const localeName = path.basename(filename).split('.')[0];
    database.processTranslationFileContent(filename, localeName, src);
    return database.precompileToBuffer(filename, localeName);
  }

  return src;
}

module.exports = { transformToString };
