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
 *
 * @property {string} publicPath
 * The path where assets are served from on Metro's HTTP server, used during development. Defaults
 * to `/assets`, which is the default path Metro assigns internally. Any overwritten path should
 * also include `/assets` as the base path for Metro to understand how to serve it.
 *
 * When rewriting an asset after compilation, this path will be prepended to the cache directory
 * where messages were compiled to, creating a full path for Metro to access the asset at runtime.
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
  publicPath: '/assets',
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
 * @returns {Promise<[ResolvedIntlAssetPluginConfig, import('metro-config').ConfigT]>}
 */
async function fetchConfig() {
  debug(`Loading Metro config`);
  const metroConfig = await metro.loadConfig();
  const watchFolders = [metroConfig.projectRoot, ...metroConfig.watchFolders];
  const defaults = {
    ...defaultConfig,
    watchFolders,
    publicPath: metroConfig.transformer?.publicPath ?? defaultConfig.publicPath,
  };

  /** @type {IntlAssetPluginConfig} */
  const config =
    // @ts-expect-error We're latching on `intlAssetPlugin` as a field in the transformer config.
    metroConfig.transformer?.intlAssetPlugin;
  if (config == null) {
    debug('asset-plugin configuration was empty. Using defaults: %O', defaults);
    return [{ ...defaults }, metroConfig];
  }
  debug(`discovered asset-plugin configuration: %O`, config);
  const resolved = {
    ...defaults,
    ...config,
  };

  if (resolved.cacheDir == null) {
    throw new Error(
      'metro-intl-transformer asset-plugin cacheDir configuration resolved to null. Assets cannot be compiled',
    );
  }

  const cacheDir = path.resolve(metroConfig.projectRoot, resolved.cacheDir);
  const cacheDirIsWatched = watchFolders.some((folder) => {
    const relative = path.relative(folder, resolved.cacheDir);
    return !relative.startsWith('..');
  });
  if (!cacheDirIsWatched) {
    throw new Error(
      "metro-intl-transformer cacheDir is not visible to metro's file watching. Assets cannot be used after compilation",
    );
  }

  resolved.cacheDir = cacheDir;
  debug(`Resolved cache directory to ${resolved.cacheDir}`);
  return [{ ...defaults, ...config }, metroConfig];
}

const pluginConfig = fetchConfig();

let hasInitializedAllDefinitions = false;

/**
 * @param {any} assetData
 */
async function transformAsset(assetData) {
  const [config, metroConfig] = await pluginConfig;
  const { cacheDir, assetExtension, format, bundleSecrets, watchFolders } = config;

  if (!hasInitializedAllDefinitions) {
    debug('Initializing database with all messages files within watch folders: %O', watchFolders);
    const result = processAllMessagesFiles(findAllMessagesFiles(watchFolders));
    debug('Finished processing all discovered messages files: %O', result);
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

  const relativeLocation = path.relative(metroConfig.projectRoot, assetData.fileSystemLocation);
  const dirHash = btoa(path.dirname(relativeLocation));
  const relativeOutputDir = path.join(cacheDir, dirHash);
  const outputDir = path.resolve(relativeOutputDir);
  const outputName = `${assetData.name}.${assetData.hash}.compiled.messages`;
  const outputFile = path.join(outputDir, outputName) + '.' + assetExtension;
  debug(`[${filename}] Output file path: ${outputFile}`);

  if (!fs.existsSync(outputDir)) {
    debug(`[${filename}] Cache dir ${outputDir} did not exist. Creating it now`);
    fs.mkdirSync(outputDir, { recursive: true });
  }

  debug(`[${filename}] processing file`);
  debug(`[${filename}] ${metroConfig.transformer?.publicPath}`);

  const result = processTranslationsFile(filename);
  debug(`[${filename}] precompiling file`);
  precompileFileForLocale(filename, result.locale, outputFile, {
    format,
    bundleSecrets,
  });
  debug(`[${filename}] finished precompiling`);

  const finalAssetData = {
    ...assetData,
    fileSystemLocation: outputDir,
    httpServerLocation: path.join(config.publicPath, relativeOutputDir),
    files: [outputFile],
    name: outputName,
    type: assetExtension,
  };

  debug(`[${filename}] Asset data: %O`, finalAssetData);
  return finalAssetData;
}

module.exports = transformAsset;
