const path = require('node:path');
const {isMessageDefinitionsFile, isMessageTranslationsFile} = require('@discord/intl-message-database');

const {database} = require('./src/database');
const {MessageDefinitionsTransformer} = require('./src/transformer');

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
      throw new Error('Expected an intl messages definition file, but found a translation file instead.');
    }

    this.data.contentFileName = sourcePath;
    // TODO: This should be built inside the native extension instead.
    const locales = database.getKnownLocales();
    /** @type {Record<string, string>} */
    const localeMap = {};
    for (const locale of locales) {
      localeMap[locale] = `${sourceFile.meta.translationsPath}/${locale}.jsona`;
    }
    localeMap['en-US'] = this.data.contentFileName + '?forceTranslation';

    const transformer = new MessageDefinitionsTransformer(database.getSourceFileHashedKeys(sourcePath), localeMap);
    return transformer.getOutput();
  } else {
    const localeName = forceTranslation ? 'en-US' : path.basename(sourcePath).split('.')[0];
    if (isMessageTranslationsFile(sourcePath)) {
      database.processTranslationFileContent(sourcePath, localeName, source);
    } else if (!forceTranslation) {
      throw new Error(
        'Expected a translation file or the `forceTranslation` query parameter on this import, but none was found',
      );
    }
    return 'export default' + database.precompileToBuffer(localeName);
  }
};

module.exports = intlLoader;
