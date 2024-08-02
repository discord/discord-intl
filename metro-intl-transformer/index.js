const {
  isMessageDefinitionsFile,
  isMessageTranslationsFile,
} = require('@discord/intl-message-database');
const fs = require('node:fs/promises');
const path = require('node:path');

const { database } = require('./src/database');
const { MessageDefinitionsTransformer } = require('./src/transformer');
const {
  ensureVirtualTranslationsModulesDirectory,
  getVirtualTranslationsModulePath,
} = require('./src/virtual-modules');

ensureVirtualTranslationsModulesDirectory();

/**
 * @param {{
 *  filename: string,
 *  src: string,
 *  getPrelude: () => string
 *  getTranslationAssetExtension: () => string,
 *  createAssetImport: (importPath: string) => string,
 * }} options
 * @returns
 */
async function transformToString({
  filename,
  src,
  getPrelude,
  getTranslationAssetExtension,
  createAssetImport,
}) {
  if (isMessageDefinitionsFile(filename)) {
    database.processDefinitionsFileContent(filename, src);
    const sourceFile = database.getSourceFile(filename);
    if (sourceFile.type !== 'definition') {
      throw new Error(
        'Expected an intl messages definition file, but found a translation file instead.',
      );
    }

    // TODO: This should be built inside the native extension instead.
    const locales = database.getKnownLocales();
    /** @type {Record<string, string>} */
    const localeMap = {};
    for (const locale of locales) {
      localeMap[locale] =
        `${sourceFile.meta.translationsPath}/${locale}.${getTranslationAssetExtension()}`;
    }

    // Metro's file map and worker model means that there's a disconnect
    // between a file getting written to the system and when Metro actually
    // becomes aware of it. This is _hideous_, but neither Watchman nor the
    // default FSEventsWatcher will have updated in time for the second part
    // of this transformation to be able to resolve the newly-created compiled
    // file. Waiting for a little bit (hopefully) gives it enough time to catch
    // the event and successfully compile the added dependency.
    const virtualModulePath = getVirtualTranslationsModulePath(
      `${filename}.compiled.${getTranslationAssetExtension()}`,
    );
    await fs.writeFile(virtualModulePath, database.precompileToBuffer('en-US'));
    await new Promise((resolve) => {
      setTimeout(resolve, 400);
    });
    localeMap['en-US'] = virtualModulePath;

    const transformer = new MessageDefinitionsTransformer(
      database.getSourceFileHashedKeys(filename),
      localeMap,
      createAssetImport,
      getPrelude,
    );

    return transformer.getOutput();
  } else if (isMessageTranslationsFile(filename)) {
    const localeName = path.basename(filename).split('.')[0];
    database.processTranslationFileContent(filename, localeName, src);
    return database.precompileToBuffer(localeName);
  }
}

module.exports = { transformToString };
