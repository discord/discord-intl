const path = require('node:path');

const chokidar = require('chokidar');
const debug = require('debug')('intl:loader-core:watcher');
const {
  isMessageDefinitionsFile,
  IntlCompiledMessageFormat,
} = require('@discord/intl-message-database');

const { database } = require('./database');
const {
  processDefinitionsFile,
  precompileFileForLocale,
  generateTypeDefinitions,
  findAllMessagesFiles,
  filterAllMessagesFiles,
  processAllMessagesFiles,
} = require('./processing');

const ALWAYS_IGNORE_PATTERNS = [
  // Ignore our own compiled message files, even though they shouldn't have a matching extension.
  '*.compiled.messages.*',
  '*.d.ts',
  '*.d.ts.map',
];
// TODO: This should come from the database extension? Or Utilities? Unsure, but the extensions
// should have some centralized location in general
const MESSAGES_FILE_PATTERNS = ['**/*.messages.*'];
const DEFAULT_LOCALE = 'en-US';

/**
 * Start a new filesystem watcher to discover and monitor messages files, allowing consumers to
 * react to changes in a single, consistent way.
 *
 * @param {string[]} watchedFolders List of folders to watch recursively.
 * @param {{
 *  ignore?: string[],
 *  skipInitial?: boolean,
 *  persistent?: boolean,
 * }} options
 */
async function watchMessagesFiles(
  watchedFolders,
  { ignore = [], skipInitial = false, persistent = true } = {},
) {
  debug('Pre-initializing database with all discoverable messages files');
  processAllMessagesFiles(findAllMessagesFiles(watchedFolders));

  const ignoredPatterns = ignore.concat(ALWAYS_IGNORE_PATTERNS);
  const globs = watchedFolders.flatMap((folder) =>
    MESSAGES_FILE_PATTERNS.map((pattern) => path.join(folder, pattern)),
  );

  debug(`Initializing watch for patterns:\n- ${globs.join('\n- ')}`);
  const watcher = chokidar.watch(globs, {
    ignored: ignoredPatterns,
    ignoreInitial: skipInitial,
    persistent,
  });
  watcher.on('all', (event, path) => {
    processAllMessagesFiles(filterAllMessagesFiles([path]));
    debug(`watcher got event: ${event}, for path ${path}`);
  });
  for (const signal of ['SIGINT', 'SIGTERM', 'SIGQUIT']) {
    process.on(signal, async () => {
      debug(`Stopping watch due to`, signal);
      await watcher.close();
      debug('Watch stopped');
    });
  }

  return watcher;
}

/**
 * Watch the file system for any changes to message files and regenerate their types.
 *
 * @param {chokidar.FSWatcher} watcher
 * @param {{
 *  allowNullability?: boolean
 * }} options
 */
async function generateMessageTypes(watcher, { allowNullability }) {
  watcher.on('all', (_, filePath) => {
    if (!isMessageDefinitionsFile(filePath)) return;
    debug(`Regenerating types for ${filePath}`);
    generateTypeDefinitions(filePath, undefined, allowNullability);
  });
}

/**
 * Watch the file system for changes to message _definitions_ files, and pre-process them into
 * compiled assets (with the extension `.compiled.messages.${assetExtension}`).
 *
 * @param {chokidar.FSWatcher} watcher
 * @param {{
 *  ignore?: string[],
 *  assetExtension?: string
 *  precompileOptions?: import('./processing').IntlPrecompileOptions,
 * }} options
 */
async function precompileMessageDefinitionsFiles(
  watcher,
  { assetExtension = 'json', precompileOptions = {} } = {},
) {
  watcher.on('all', (_, filePath) => {
    const { format = IntlCompiledMessageFormat.KeylessJson, bundleSecrets = false } =
      precompileOptions;
    if (!isMessageDefinitionsFile(filePath)) return;

    debug(`Processing file: ${filePath}`);
    try {
      // Convert the file name from `.messages.js` to `.compiled.messages.jsona` for output.
      const outputPath = filePath.replace(
        /\.messages\.js$/,
        `.compiled.messages.${assetExtension}`,
      );
      const result = processDefinitionsFile(filePath);
      precompileFileForLocale(filePath, result.locale, undefined, {
        format,
        bundleSecrets,
      });

      database.processDefinitionsFile(filePath);
      database.precompile(filePath, DEFAULT_LOCALE, outputPath, {
        format: IntlCompiledMessageFormat.KeylessJson,
        bundleSecrets,
      });
      debug(`Wrote definitions to: ${outputPath}`);
    } catch (e) {
      debug('[INTL Error] Failed to compile messages');
      console.error(e);
    }
  });
}

module.exports = {
  watchMessagesFiles,
  precompileMessageDefinitionsFiles,
  generateMessageTypes,
  ALWAYS_IGNORE_PATTERNS,
};
