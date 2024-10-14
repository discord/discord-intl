const {
  processDefinitionsFile,
  processTranslationsFile,
  isMessageDefinitionsFile,
  MessageDefinitionsTransformer,
  isMessageTranslationsFile,
  getLocaleFromTranslationsFileName,
  precompileFileForLocale,
  IntlCompiledMessageFormat,
} = require('@discord/intl-loader-core');

module.exports = {
  INTL_MESSAGES_FILE_PATTERN: '\\.messages\\.(js|json|jsona)(\\?forceTranslation)?$',
  /**
   * @param {string} source
   * @param {string} filePath
   * @param {object} _config
   * @returns {string}
   */
  process(source, filePath, _config) {
    const precompileOptions = {
      format: IntlCompiledMessageFormat.KeylessJson,
      // Always bundle secrets when running tests
      bundleSecrets: true,
    };

    const forceTranslation = filePath === '?forceTranslation';
    if (isMessageDefinitionsFile(filePath) && !forceTranslation) {
      const result = processDefinitionsFile(filePath, source, { processTranslations: false });

      const sourceMessages = /** @type {Buffer} */ (
        precompileFileForLocale(filePath, result.locale, undefined, precompileOptions)
      ).toString();

      result.translationsLocaleMap[result.locale] = 'intl$sentinel-use-local';

      return new MessageDefinitionsTransformer({
        messageKeys: result.messageKeys,
        localeMap: result.translationsLocaleMap,
        defaultLocale: result.locale,
        getTranslationImport: (importPath) => {
          // Embed source translations directly into the loader, because Jest doesn't have a loader
          // mechanism to create new dependencies while transforming a file.
          if (importPath === 'intl$sentinel-use-local') {
            return `Promise.resolve({default: ${sourceMessages}})`;
          }
          return `Promise.resolve(require("${importPath}"))`;
        },
        debug: true,
        // Jest won't pass the result of this transpilation through another processor, and expects
        // babel-like transpilation to have occurred, so this processor exports transpiled-
        // compatible values as a safe default.
        exportMode: 'transpiledEsModule',
      }).getOutput();
    } else {
      const locale = getLocaleFromTranslationsFileName(filePath);
      if (isMessageTranslationsFile(filePath)) {
        processTranslationsFile(filePath, source, { locale });
      } else if (!forceTranslation) {
        throw new Error(
          'Expected a translation file or the `forceTranslation` query parameter on this import, but none was found',
        );
      }

      const compiledResult = /** @type {Buffer} */ (
        precompileFileForLocale(filePath, locale, undefined, precompileOptions)
      );

      if (forceTranslation) {
        return (
          'module.exports.default = JSON.parse(' + JSON.stringify(compiledResult?.toString()) + ')'
        );
      } else {
        return compiledResult.toString();
      }
    }
  },
};
