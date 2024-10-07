const path = require('node:path');

const {
  IntlCompiledMessageFormat,
  MessageDefinitionsTransformer,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  getLocaleFromTranslationsFileName,
  processTranslationsFile,
  precompileFileForLocale,
} = require('@discord/intl-loader-core');
const debug = require('debug')('intl:rspack-intl-loader');

const FILE_PATH_SEPARATOR_MATCH = new RegExp(`[\\\\\\/]`, 'g');

/**
 * @param {string} source
 * @param {string} file
 * @returns {string}
 */
function makePosixRelativePath(source, file) {
  return (
    './' +
    path.relative(path.dirname(source), file).replace(FILE_PATH_SEPARATOR_MATCH, path.posix.sep)
  );
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

  debug(`[${sourcePath}] Processing intl messages file (forceTranslation=${forceTranslation})`);

  if (isMessageDefinitionsFile(sourcePath) && !forceTranslation) {
    debug(`[${sourcePath}] Determined to be a definitions file`);
    const result = processDefinitionsFile(sourcePath, source, {
      // TODO: Make this more configurable
      locale: 'en-US',
    });

    // Ensure that rspack knows to watch all of the translations files, even though they aren't
    // directly imported from a source. Without this, even though the compiled loader references the
    // transpiled message files, it won't trigger refreshes when those files change.
    for (const translationsFile of Object.values(result.translationsLocaleMap)) {
      this.addDependency(translationsFile);
    }

    // Normalize the path to a POSIX-like JS path. Using absolute paths on Windows, rspack currently
    // considers the leading drive name (like `C:\`) as a scheme, which is incorrect. It also
    // appears to process all paths as the posix style, meaning backlashes aren't preserved (e.g.,
    // the output of `C:\path\to\file` is interpreted as `C:\pathtofile`.
    result.translationsLocaleMap['en-US'] = sourcePath + '?forceTranslation';
    for (const locale in result.translationsLocaleMap) {
      result.translationsLocaleMap[locale] = makePosixRelativePath(
        sourcePath,
        result.translationsLocaleMap[locale],
      );
    }

    debug('Locale map created: %O', result.translationsLocaleMap);
    debug(
      `[${sourcePath}] Compiled translations file will be: ${result.translationsLocaleMap['en-US']}`,
    );

    return new MessageDefinitionsTransformer({
      messageKeys: result.messageKeys,
      localeMap: result.translationsLocaleMap,
      defaultLocale: result.locale,
      getTranslationImport: (importPath) => `import("${importPath}")`,
      debug: process.env.NODE_ENV === 'development',
    }).getOutput();
  } else {
    const locale = forceTranslation ? 'en-US' : getLocaleFromTranslationsFileName(sourcePath);
    if (isMessageTranslationsFile(sourcePath)) {
      debug(`[${sourcePath}] Determined to be a translations file`);
      processTranslationsFile(sourcePath, source, { locale });
    } else if (forceTranslation) {
      debug(`[${sourcePath}] Forcing processing as a translation file`);
    } else {
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
      debug(`[${sourcePath}] Emitting JS module because forceTranslation was true`);
      return 'export default JSON.parse(' + JSON.stringify(compiledResult?.toString()) + ')';
    } else {
      debug(`[${sourcePath}] Emitting plain JSON because forceTranslation was false`);
      return compiledResult;
    }
  }
};

module.exports = intlLoader;