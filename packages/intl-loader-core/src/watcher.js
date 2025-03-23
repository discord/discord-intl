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
 * @typedef {{
 *   on(event: string, callback: (event: string, path: string) => void): void,
 * }} BasicWatcher
 */

/**
 * Start a new filesystem watcher to discover and monitor messages files, allowing consumers to
 * react to changes in a single, consistent way.
 *
 * @param {string[]} watchedFolders List of folders to watch recursively.
 * @param {{
 *  ignore?: string[],
 *  skipInitial?: boolean,
 *  persistent?: boolean,
 *  once?: boolean,
 * }} options
 * @returns {Promise<BasicWatcher>}
 */
async function watchMessagesFiles(
  watchedFolders,
  { ignore = [], skipInitial = false, once = true } = {}
) {
  debug('Pre-initializing database with all discoverable messages files');
  const files = findAllMessagesFiles(watchedFolders);
  const result = processAllMessagesFiles(files);
  const reprocessQueue = result
    .filter((data) => data.errors.length > 0)
    .map((data) => data.fileKey);

  if (once) {
    debug('Performing one-shot watch because `once` was true');
    // This is a weird hack around file watching just not working well on
    // containers/CI. `add` events won't fire consistently from chokidar when
    // it's not set to be persistent. So for a `once` run, we can just manually
    // invoke the event for every file that was discovered on the initial scan.
    return {
      on(_, callback) {
        for (const file of files) {
          callback('add', file.filePath);
        }
      },
    };
  }

  const ignoredPatterns = ignore.concat(ALWAYS_IGNORE_PATTERNS);
  const globs = watchedFolders.flatMap((folder) =>
    MESSAGES_FILE_PATTERNS.map((pattern) => path.join(folder, pattern))
  );

  debug(`Initializing watch for patterns:\n- ${globs.join('\n- ')}`);
  const watcher = chokidar.watch(globs, {
    ignored: ignoredPatterns,
    ignoreInitial: skipInitial,
    persistent: !once,
  });

  watcher.on('all', (event, path) => {
    const files = filterAllMessagesFiles([path]);
    if (files.length === 0) return;

    // Process the file that changed first, then reprocess all of the queued files from previous
    // failures to hopefully get them to process successfully again.
    const [result] = processAllMessagesFiles(files);
    const failed = result.errors.length > 0 ? [result.fileKey] : [];
    const secondResult = processAllMessagesFiles(
      filterAllMessagesFiles([...reprocessQueue, ...failed])
    );
    // For any files that are still failing, keep them in the queue for next time.
    const secondFailures = secondResult
      .filter((data) => data.errors.length > 0)
      .map((data) => data.fileKey);
    if (secondFailures.length > 0) {
      debug('Failed to re-process files: %O. Next update will try again.', secondFailures);
    }
    reprocessQueue.splice(0, Infinity, ...secondFailures);

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
 * @param {BasicWatcher} watcher
 */
async function generateMessageTypes(watcher) {
  watcher.on('all', (_, filePath) => {
    if (!isMessageDefinitionsFile(filePath)) return;
    debug(`Regenerating types for ${filePath}`);
    generateTypeDefinitions(filePath, undefined);
  });
}

/**
 * Watch the file system for changes to message _definitions_ files, and pre-process them into
 * compiled assets (with the extension `.compiled.messages.${assetExtension}`).
 *
 * @param {BasicWatcher} watcher
 * @param {{
 *  ignore?: string[],
 *  assetExtension?: string
 *  precompileOptions?: import('./processing').IntlPrecompileOptions,
 * }} options
 */
async function precompileMessageDefinitionsFiles(
  watcher,
  { assetExtension = 'json', precompileOptions = {} } = {}
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
        `.compiled.messages.${assetExtension}`
      );
      const result = processDefinitionsFile(filePath);
      if (!result.succeeded) {
        debug('[INTL Error] Failed to compile messages');
        console.error(result.errors);
        return;
      }

      precompileFileForLocale(filePath, result.locale, undefined, {
        format,
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
