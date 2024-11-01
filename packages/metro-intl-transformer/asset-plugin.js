const fs = require('node:fs');
const path = require('node:path');
const debug = require('debug')('intl:metro-intl-transformer:asset-plugin');
const metro = require('metro-config');

const {
  isMessageTranslationsFile,
  processTranslationsFile,
  precompileFileForLocale,
  IntlCompiledMessageFormat,
  findAllMessagesFiles,
  processAllMessagesFiles,
} = require('@discord/intl-loader-core');

/**
 * @typedef ResolvedIntlAssetPluginConfig
 *
 * @property {string} cacheDir
 * Directory to store compiled message assets while bundling. Defaults to `.cache/intl`. This
 * directory is resolved relative to project root and _must_ live somewhere within it.
 *
 * @property {string} assetExtension
 * The final extension to use when searching for and creating assets. Defaults to `json`. Note that
 * all compiled assets will include the initial extension `.compiled.messages.`, followed by this
 * configuration.
 *
 * @property {boolean} bundleSecrets
 * Whether messages marked as `secret` will be preserved in the bundled message assets. When false,
 * secret messages have their values replaced with obfuscated text to prevent leaking information.
 *
 * @property {IntlCompiledMessageFormat} format
 * The format to which messages should be compiled during bundling. `Json` will cause the messages
 * to be compiled to a FormatJS-like compatible format, while `KeylessJson` will use a much more
 * compressed, `@discord/intl`-specific format. `KeylessJson` is the default.
 *
 * @property {string[]} watchFolders
 * The folders Metro is configured to watch. A necessary argument to be able to preload all message
 * definitions and ensure that merged translations files handle secrets appropriately based on
 * matching definitions, which may otherwise be missed if the definitions files are handled by
 * different workers. If not specified directly in the configuration file (either through the
 * plugin's configuration or on Metro itself), this will default to `process.cwd()`.
 *
 * This configuration also automatically reads `projectRoot` and replaces `process.cwd()` when it
 * has been specified.
 */

/**
 * @typedef {Partial<ResolvedIntlAssetPluginConfig>} IntlAssetPluginConfig
 */

/** @type {ResolvedIntlAssetPluginConfig} */
const defaultConfig = {
  cacheDir: path.join('.cache', 'intl'),
  assetExtension: 'json',
  format: IntlCompiledMessageFormat.KeylessJson,
  bundleSecrets: false,
  watchFolders: [process.cwd()],
};

/**
 * Configure the asset-plugin for `@discord/metro-intl-transformer`. Necessary because Metro's
 * `assetPlugins` configuration only accepts a list of module paths for plugins without an option
 * for configuring them inline.
 *
 * This relies on extending `metro.config.js` with a new property, `intlAssetPlugin`, providing
 * the desired configuration for that run of the bundler. If the Metro configuration for the current
 * process is at a non-default location, this method will likely fail.
 *
 * @returns {Promise<ResolvedIntlAssetPluginConfig>}
 */
async function fetchConfig() {
  debug(`Loading Metro config`);
  const metroConfig = await metro.loadConfig();
  const defaults = {
    ...defaultConfig,
    watchFolders: [metroConfig.projectRoot, ...metroConfig.watchFolders],
  };

  // @ts-expect-error We're latching on `intlAssetPlugin` as a field in the transformer config.
  const config = metroConfig.transformer?.intlAssetPlugin;
  if (config == null) {
    debug('asset-plugin configuration was empty. Using defaults: %O', defaults);
    return { ...defaults };
  }
  debug(`discovered asset-plugin configuration: %O`, config);

  if (config.cacheDir != null && path.isAbsolute(config.cacheDir)) {
    throw new Error(
      '`cacheDir` configuration for the intl Metro asset-plugin must be a relative path.',
    );
  }

  return { ...defaults, ...config };
}

const pluginConfig = fetchConfig();

let hasInitializedAllDefinitions = false;

/**
 * @param {any} assetData
 */
async function transformAsset(assetData) {
  const { cacheDir, assetExtension, format, bundleSecrets, watchFolders } = await pluginConfig;

  if (!hasInitializedAllDefinitions) {
    debug('Initializing database with all messages files within watch folders: %O', watchFolders);
    processAllMessagesFiles(findAllMessagesFiles(watchFolders));
    hasInitializedAllDefinitions = true;
  }

  const filename = assetData.files[0] ?? '';
  // If this isn't a translations file or if it's already a compiled artifact, then we don't want
  // to do any more processing on it.
  if (filename === '' || !isMessageTranslationsFile(filename)) {
    return assetData;
  }
  if (/\.compiled.messages\./.test(filename)) {
    debug(`[${filename}] Compiled messages asset needs no further processing`);
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
  precompileFileForLocale(filename, result.locale, outputFile, {
    format,
    bundleSecrets,
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
