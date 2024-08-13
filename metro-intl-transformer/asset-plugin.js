const path = require('node:path');
const debug = require('debug')('intl:metro-intl-transformer:asset-plugin');

const {
  isMessageTranslationsFile,
  processTranslationsFile,
  precompileFileForLocale,
  IntlCompiledMessageFormat,
} = require('@discord/intl-loader-core');

/**
 * @param {any} assetData
 */
function transformAsset(assetData) {
  const filename = assetData.files[0] ?? '';
  if (!isMessageTranslationsFile(filename) || /\.compiled.messages\./.test(filename))
    return assetData;

  const outputFile = filename.replace(/\.messages\.js(ona?)?$/, `.compiled.messages.jsona`);

  debug(`[${filename}] Processing translation asset`);
  const result = processTranslationsFile(filename);
  precompileFileForLocale(filename, result.locale, {
    format: IntlCompiledMessageFormat.Json,
    outputFile,
  });

  return {
    ...assetData,
    files: [outputFile],
    name: path.basename(outputFile).replace(/\.jsona$/, ''),
    type: 'jsona',
  };
}

module.exports = transformAsset;
