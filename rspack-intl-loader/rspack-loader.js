const {
  IntlCompiledMessageFormat,
  MessageDefinitionsTransformer,
  database,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  getLocaleFromTranslationsFileName,
  processTranslationsFile,
  precompileFileForLocale,
} = require('@discord/intl-loader-core');

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
    const result = processDefinitionsFile(sourcePath, source);

    // Ensure that rspack knows to watch all of the translations files, even though they aren't
    // directly imported from a source. Without this, even though the compiled loader references the
    // transpiled message files, it won't trigger refreshes when those files change.
    for (const translationsFile of Object.values(result.translationsLocaleMap)) {
      this.addDependency(translationsFile);
    }

    result.translationsLocaleMap['en-US'] = sourcePath + '?forceTranslation';

    return new MessageDefinitionsTransformer({
      messageKeys: result.hashedMessageKeys,
      localeMap: result.translationsLocaleMap,
      getTranslationImport: (importPath) => `import("${importPath}")`,
    }).getOutput();
  } else {
    const locale = forceTranslation ? 'en-US' : getLocaleFromTranslationsFileName(sourcePath);
    if (isMessageTranslationsFile(sourcePath)) {
      processTranslationsFile(sourcePath, source, { locale });
    } else if (!forceTranslation) {
      throw new Error(
        'Expected a translation file or the `forceTranslation` query parameter on this import, but none was found',
      );
    }

    const compiledResult = precompileFileForLocale(sourcePath, locale, {
      format: IntlCompiledMessageFormat.Json,
    });

    // Translations are still treated as JS files that need to be pre-parsed.
    // Rspack will handle parsing for the actual JSON file requests.
    if (forceTranslation) {
      return 'export default JSON.parse(' + compiledResult + ')';
    } else {
      return compiledResult;
    }
  }
};

module.exports = intlLoader;
