const path = require('node:path');

const chokidar = require('chokidar');
const fg = require('fast-glob');
const debug = require('debug')('intl:loader-core:watcher');
const {
  isMessageDefinitionsFile,
  IntlCompiledMessageFormat,
} = require('@discord/intl-message-database');

const { database } = require('./database');
const { processDefinitionsFile, precompileFileForLocale } = require('./processing');

const ALWAYS_IGNORE_PATTERNS = [
  // Ignore our own compiled message files, even though they shouldn't have a matching extension.
  '*.compiled.messages.*',
];
// TODO: This should come from the database extension? Or Utilities? Unsure, but the extensions
// should have some centralized location in general
const MESSAGE_DEFINITION_FILE_PATTERNS = ['**/*.messages.js'];
const DEFAULT_LOCALE = 'en-US';

/**
 * @param {string} filePath
 * @param {string} assetExtension
 * @param {import('./processing').IntlPrecompileOptions} [options]
 */
function processFile(filePath, assetExtension, options = {}) {
  const { format = IntlCompiledMessageFormat.KeylessJson, bundleSecrets = false } = options;
  debug(`Processing file: ${filePath}`);
  if (!isMessageDefinitionsFile(filePath)) {
    debug(`${filePath} is not a definitions file. Skipping processing`);
    return;
  }

  try {
    // Convert the file name from `.messages.js` to `.compiled.messages.jsona` for output.
    const outputPath = filePath.replace(/\.messages\.js$/, `.compiled.messages.${assetExtension}`);
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
}

/**
 * @param {string[]} watchedFolders
 * @param {{
 *  watch?: boolean,
 *  ignore?: string[],
 *  assetExtension?: string
 *  precompileOptions?: import('./processing').IntlPrecompileOptions,
 * }} options
 */
async function compileIntlMessageFiles(
  watchedFolders,
  { watch = true, ignore = [], assetExtension = 'json', precompileOptions = {} } = {},
) {
  const ignoredPatterns = ignore.concat(ALWAYS_IGNORE_PATTERNS);
  const globs = watchedFolders.flatMap((folder) =>
    MESSAGE_DEFINITION_FILE_PATTERNS.map((pattern) => path.join(folder, pattern)),
  );
  debug(`Configured message file patterns:\n- ${globs.join('\n- ')}`);

  // Perform one initial scan and compilation to ensure all files exist before Metro might try to
  // resolve them.
  debug('Scanning for initial messages files');
  for await (const filePath of fg.stream(globs, {
    ignore: ignoredPatterns,
    absolute: true,
    onlyFiles: true,
  })) {
    processFile(filePath.toString(), assetExtension, precompileOptions);
  }
  debug('Initial message scan completed.');

  if (watch) {
    debug(`Setting up file watching for configured paths`);
    chokidar
      .watch(globs, { ignored: ignoredPatterns, ignoreInitial: true })
      .on('all', (event, filePath) => {
        debug(`Got event ${event} for ${filePath}`);
        processFile(filePath, assetExtension, precompileOptions);
      });
  } else {
    debug('Not watching files because `watch` option was false');
  }
}

module.exports = { compileIntlMessageFiles, ALWAYS_IGNORE_PATTERNS };
