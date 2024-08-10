const path = require('node:path');
const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
} = require('@discord/intl-message-database');
const debug = require('debug')('intl:metro-intl-transformer');

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
    debug(`Processing ${filename} as a definitions file`);
    database.processDefinitionsFileContent(filename, src);
    const sourceFile = database.getSourceFile(filename);
    if (sourceFile.type !== 'definition') {
      throw new Error(
        'Expected an intl messages definition file, but found a translation file instead.',
      );
    }

    const locales = database.getKnownLocales();
    debug(`Creating locale map to translations at path: ${sourceFile.meta.translations_path}`);
    /** @type {Record<string, string>} */
    const localeMap = {};
    for (const locale of locales) {
      localeMap[locale] =
        `${sourceFile.meta.translations_path}/${locale}.${getTranslationAssetExtension()}`;
    }
    const compiledSourcePath = filename.replace(/\.messages\.js$/, '.compiled.messages.jsona');
    debug(`Resolving source file ${filename} to compiled translations file ${compiledSourcePath}`);
    localeMap['en-US'] = compiledSourcePath;

    const transformer = new MessageDefinitionsTransformer(
      database.getSourceFileHashedKeys(filename),
      localeMap,
      createAssetImport,
      getPrelude,
    );

    return transformer.getOutput();
  } else if (isMessageTranslationsFile(filename)) {
    debug(`Processing ${filename} as a translations file`);
    const localeName = path.basename(filename).split('.')[0];
    database.processTranslationFileContent(filename, localeName, src);
    return database.precompileToBuffer(filename, localeName);
  }

  return src;
}

module.exports = { transformToString };
