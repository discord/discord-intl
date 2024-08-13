const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
  processDefinitionsFile,
  MessageDefinitionsTransformer,
  processTranslationsFile,
  precompileFileForLocale,
} = require('@discord/intl-loader-core');
const debug = require('debug')('intl:metro-intl-transformer');

/**
 * @param {{
 *  filename: string,
 *  src: string,
 *  getPrelude: () => string
 *  getTranslationAssetExtension: () => string,
 *  getTranslationImport: (importPath: string) => string,
 * }} options
 * @returns {string | Buffer}
 */
function transformToString({
  filename,
  src,
  getPrelude,
  getTranslationAssetExtension,
  getTranslationImport,
}) {
  if (isMessageDefinitionsFile(filename)) {
    debug(`Processing ${filename} as a definitions file`);
    const result = processDefinitionsFile(filename, src, {
      // TODO: Make this more configurable
      locale: 'en-US',
    });
    const compiledSourcePath = filename.replace(
      /\.messages\.js$/,
      `.compiled.messages.${getTranslationAssetExtension()}`,
    );

    debug(`Resolving source file ${filename} to compiled translations file ${compiledSourcePath}`);
    result.translationsLocaleMap['en-US'] = compiledSourcePath;

    return new MessageDefinitionsTransformer({
      messageKeys: result.messageKeys,
      localeMap: result.translationsLocaleMap,
      defaultLocale: result.locale,
      getTranslationImport,
      getPrelude,
      debug: process.env.NODE_ENV === 'development',
    }).getOutput();
  } else if (isMessageTranslationsFile(filename)) {
    debug(`Processing ${filename} as a translations file`);
    const result = processTranslationsFile(filename, src);
    // @ts-expect-error Without the `outputFile` option, this always returns a Buffer, but the
    // option allows the function to return void instead.
    return precompileFileForLocale(filename, result.locale);
  }

  return src;
}

module.exports = { transformToString };
