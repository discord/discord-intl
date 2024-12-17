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
 *  getPrelude: () => string,
 *  format?: IntlCompiledMessageFormat,
 *  bundleSecrets?: boolean,
 *  preGenerateBinds?: boolean,
 * }} options
 * @returns {string | Buffer}
 */
function transformToString({
  filename,
  src,
  getPrelude,
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
    result.translationsLocaleMap[result.locale] = 'intl$sentinel-use-local';
    debug('Locale map created: %O', result.translationsLocaleMap);

    const sourceMessages = /** @type {Buffer} */ (
      precompileFileForLocale(filename, result.locale, undefined, { format, bundleSecrets })
    ).toString();

    return new MessageDefinitionsTransformer({
      messageKeys: result.messageKeys,
      localeMap: result.translationsLocaleMap,
      defaultLocale: result.locale,
      getTranslationImport(importPath) {
        if (importPath === 'intl$sentinel-use-local') {
          return `Promise.resolve({default: ${sourceMessages}})`;
        }
        return `import("${importPath}")`;
      },
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
