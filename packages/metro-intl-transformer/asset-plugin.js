const fs = require('node:fs');
const path = require('node:path');
const debug = require('debug')('intl:metro-intl-transformer:asset-plugin');

const {
  isMessageTranslationsFile,
  processTranslationsFile,
  precompileFileForLocale,
  IntlCompiledMessageFormat,
} = require('@discord/intl-loader-core');

/**
 * @typedef ResolvedIntlAssetPluginConfig
 *
 * @property {string=} cacheDir
 * Directory to store compiled message assets while bundling. Defaults to `.cache/intl`. This
 * directory is resolved relative to project root and _must_ live somewhere within it.
 *
 * @property {string=} assetExtension
 * The final extension to use when searching for and creating assets. Defaults to `json`. Note that
 * all compiled assets will include the initial extension `.compiled.messages.`, followed by this
 * configuration.
 */

/**
 * @typedef {Partial<ResolvedIntlAssetPluginConfig>} IntlAssetPluginConfig
 */

/** @type {ResolvedIntlAssetPluginConfig} */
const defaultConfig = {
  cacheDir: path.join('.cache', 'intl'),
  assetExtension: 'json',
};

/**
 * Configure the asset-plugin for `@discord/metro-intl-transformer`. Necessary because Metro's
 * `assetPlugins` configuration only accepts a list of module paths for plugins without an option
 * for configuring them inline.
 *
 * This relies on extending `metro.config.js` with a new property, `intlAssetPlugin`, providing
 * the desired configuration for that run of the bundler. If the Metro configuration for the current
 * process is at a non-default location, this method will likely fail.
 */
async function fetchConfig() {
  const metroConfigPath = path.join(process.cwd(), 'metro.config.js');

  debug(`Loading Metro config from ${metroConfigPath}`);
  let metroConfig;
  try {
    metroConfig = require(metroConfigPath);
    debug(`Successfully loaded config: %o`, metroConfig);
  } catch {
    debug(`Failed to load Metro configuration. Using defaults instead.`);
    return defaultConfig;
  }

  const config = metroConfig.transformer?.intlAssetPlugin;
  if (config == null) {
    return defaultConfig;
  }

  if (config.cacheDir != null && path.isAbsolute(config.cacheDir)) {
    throw new Error(
      '`cacheDir` configuration for the intl Metro asset-plugin must be a relative path.',
    );
  }

  return { ...defaultConfig, ...config };
}

const pluginConfig = fetchConfig();

/**
 * @param {any} assetData
 */
async function transformAsset(assetData) {
  const { cacheDir, assetExtension } = await pluginConfig;

  const filename = assetData.files[0] ?? '';
  // If this isn't a translations file or if it's already a compiled artifact, then we don't want
  // to do any more processing on it.
  if (
    filename === '' ||
    !isMessageTranslationsFile(filename) ||
    /\.compiled.messages\./.test(filename)
  ) {
    return assetData;
  }

  debug(`[${filename}] Processing translation asset`);

  const outputDir = path.resolve(assetData.fileSystemLocation, cacheDir);
  const outputName = `${assetData.name}.${assetData.hash}.compiled.messages`;
  const outputFile = path.join(outputDir, outputName) + '.' + assetExtension;
  debug(`[${filename}] Output file path: ${outputFile}`);

  if (!fs.existsSync(outputDir)) {
    debug(`[${filename}] Cache dir ${outputDir} did not exist. Creating it now`);
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const result = processTranslationsFile(filename);
  precompileFileForLocale(filename, result.locale, {
    format: IntlCompiledMessageFormat.KeylessJson,
    outputFile,
  });

  return {
    ...assetData,
    fileSystemLocation: outputDir,
    httpServerLocation: `${assetData.httpServerLocation}/${cacheDir}`,
    files: [outputFile],
    name: outputName,
    type: assetExtension,
  };
}

module.exports = transformAsset;
