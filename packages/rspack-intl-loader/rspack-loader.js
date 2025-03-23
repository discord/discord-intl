const path = require('node:path');

const {
  MessageDefinitionsTransformer,
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  getLocaleFromTranslationsFileName,
  processTranslationsFile,
  precompileFileForLocale,
  IntlCompiledMessageFormat,
  processAllMessagesFiles,
  findAllMessagesFiles,
  findAllDefinitionsFilesForTranslations,
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

let hasInitializedAllDefinitions = false;

/**
 * Take in a file that contains message definitions (e.g., calls to
 * `defineMessages`), extract the pieces necessary for the runtime i18n code
 * to use (only the string id), and transform the module to only include a
 * default export of that content.
 *
 * @param {string} source
 * @this {import('webpack').LoaderContext<{
 *   format: IntlCompiledMessageFormat,
 *   bundleSecrets: boolean,
 *   jsonExportMode?: 'rspack' | 'webpack',
 *   bindMode?: 'proxy' | 'literal',
 *   watchFolders?: string[],
 *   sourceLocale?: string,
 * }>}
 */
const intlLoader = function intlLoader(source) {
  const sourcePath = this.resourcePath;
  const forceTranslation = this.resourceQuery === '?forceTranslation';
  const {
    bundleSecrets = false,
    format = IntlCompiledMessageFormat.KeylessJson,
    jsonExportMode = 'rspack',
    bindMode = 'proxy',
    watchFolders = [this._compiler?.context ?? process.cwd()],
    sourceLocale = 'en-US',
  } = this.getOptions();

  if (!hasInitializedAllDefinitions) {
    debug('Initializing database with all messages files within watch folders: %O', watchFolders);
    const result = processAllMessagesFiles(findAllMessagesFiles(watchFolders));
    debug('Finished processing all discovered messages files: %O', result);
    hasInitializedAllDefinitions = true;
  }

  debug(`[${sourcePath}] Processing intl messages file (forceTranslation=${forceTranslation})`);

  /**
   * @param {import('@discord/intl-loader-core/types').IntlProcessingError} result
   * @returns {never}
   */
  const failFromProcessingErrors = (result) => {
    debug(`[${sourcePath}] Failed to process definitions: %O`, result.errors);
    for (const error of result.errors) {
      this.emitError(new Error(error.message));
    }
    throw new Error(result.errors[0].message);
  };

  if (isMessageDefinitionsFile(sourcePath) && !forceTranslation) {
    debug(`[${sourcePath}] Determined to be a definitions file`);
    const result = processDefinitionsFile(sourcePath, source, { locale: sourceLocale });
    if (!result.succeeded) failFromProcessingErrors(result);

    // Ensure that rspack knows to watch all of the translations files, even though they aren't
    // directly imported from a source. Without this, even though the compiled loader references the
    // transpiled message files, it won't trigger refreshes when those files change.
    //
    // Note that this creates a semi-intentional circular dependency between translations and
    // definitions due to the reverse dependence of translations on definitions for their content.
    for (const translationsFile of Object.values(result.translationsLocaleMap)) {
      this.addDependency(translationsFile);
    }

    // Normalize the path to a POSIX-like JS path. Using absolute paths on Windows, rspack currently
    // considers the leading drive name (like `C:\`) as a scheme, which is incorrect. It also
    // appears to process all paths as the posix style, meaning backlashes aren't preserved (e.g.,
    // the output of `C:\path\to\file` is interpreted as `C:\pathtofile`.
    result.translationsLocaleMap[result.locale] = sourcePath + '?forceTranslation';
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
      bindMode,
    }).getOutput();
  } else {
    const locale = forceTranslation ? 'en-US' : getLocaleFromTranslationsFileName(sourcePath);
    if (isMessageTranslationsFile(sourcePath)) {
      debug(`[${sourcePath}] Determined to be a translations file`);
      const result = processTranslationsFile(sourcePath, source, { locale });
      if (!result.succeeded) failFromProcessingErrors(result);

      // Translations file content is affected by the content of the definitions file (e.g., the
      // `secret` meta value), so it can only be cached safely by adding a loader dependency on the
      // definitions file.
      const owningDefinitions = findAllDefinitionsFilesForTranslations(sourcePath);
      for (const file in owningDefinitions) {
        this.addBuildDependency(file);
      }
    } else if (forceTranslation) {
      debug(`[${sourcePath}] Forcing processing as a translation file`);
    } else {
      throw new Error(
        'Expected a translation file or the `forceTranslation` query parameter on this import, but none was found',
      );
    }

    const compiledResult = precompileFileForLocale(sourcePath, locale, undefined, {
      format,
      bundleSecrets,
    });

    // Translations are still treated as JS files that need to be pre-parsed.
    // Rspack will handle parsing for the actual JSON file requests, but
    // Webpack won't, so there's also the configurable override to force a
    // certain compilation method as needed.
    switch (jsonExportMode) {
      case 'rspack':
        if (forceTranslation) {
          debug(`[${sourcePath}] Emitting ESM JS compiled module`);
          return 'export default JSON.parse(' + JSON.stringify(compiledResult?.toString()) + ')';
        } else {
          debug(`[${sourcePath}] Emitting plain JSON because forceTranslation was false`);
          return compiledResult;
        }
      case 'webpack':
        debug(`[${sourcePath}] Emitting CommonJS compiled module`);
        return (
          'exports["default"] = JSON.parse(' + JSON.stringify(compiledResult?.toString()) + ')'
        );
    }
  }
};

module.exports = intlLoader;
