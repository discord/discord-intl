const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  MessageDefinitionsTransformer,
  processTranslationsFile,
  precompileFileForLocale,
  IntlCompiledMessageFormat,
} = require('@discord/intl-loader-core');
const debug = require('debug')('intl:metro-intl-transformer');

/**
 * @param {{
 *  filename: string,
 *  src: string,
 *  getPrelude: () => string
 *  getTranslationAssetExtension: () => string,
 *  getTranslationImport: (importPath: string) => string,
 *  format?: IntlCompiledMessageFormat,
 *  bundleSecrets?: boolean,
 *  preGenerateBinds?: boolean | 'proxy',
 * }} options
 * @returns {string | Buffer}
 */
function transformToString({
  filename,
  src,
  getPrelude,
  getTranslationAssetExtension,
  getTranslationImport,
  format = IntlCompiledMessageFormat.KeylessJson,
  bundleSecrets = false,
  preGenerateBinds = true,
}) {
  if (isMessageDefinitionsFile(filename)) {
    debug(`[${filename}] Processing as a definitions file`);
    const result = processDefinitionsFile(filename, src, {
      // TODO: Make this more configurable
      locale: 'en-US',
    });
    const compiledSourcePath = filename.replace(
      /\.messages\.js$/,
      `.compiled.messages.${getTranslationAssetExtension()}`,
    );

    debug(
      `[${filename}] Resolving source file to compiled translations file ${compiledSourcePath}`,
    );
    result.translationsLocaleMap[result.locale] = compiledSourcePath;
    debug('Locale map created: %O', result.translationsLocaleMap);

    return new MessageDefinitionsTransformer({
      messageKeys: result.messageKeys,
      localeMap: result.translationsLocaleMap,
      defaultLocale: result.locale,
      getTranslationImport,
      getPrelude,
      debug: process.env.NODE_ENV === 'development',
      preGenerateBinds,
    }).getOutput();
  } else if (isMessageTranslationsFile(filename)) {
    debug(`[${filename}] Processing as a translations file`);
    const result = processTranslationsFile(filename, src);
    // @ts-expect-error Without the `outputFile` option, this always returns a Buffer, but the
    // option allows the function to return void instead.
    return precompileFileForLocale(filename, result.locale, undefined, {
      format,
      bundleSecrets,
    });
  }

  return src;
}

module.exports = { transformToString };
