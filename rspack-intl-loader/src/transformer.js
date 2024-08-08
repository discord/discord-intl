/**
 * Common class for parsing and transforming the content of a messages
 * definition file (e.g., "SomeFeature.messages.js") into a production-ready
 * version, with string keys obfuscated, loading harnesses configured, and
 * more.
 *
 * This transformation is intended to be used alongside the consumer
 * transforms implemented as SWC and Babel plugins, which transform the
 * callsites for messages into matching formats. Consider this input example:
 *
 * ```typescript
 * // SomeModule.messages.js
 * import {defineMessages} from '@discord/intl';
 *
 * export default defineMessages({
 *   THIS_IS_A_MESSAGE: 'it has some content with {values}',
 * });
 *
 * // SomeConsumer.tsx
 * import someModuleMessages from 'SomeModule.messages.js';
 * i18n.format(someModuleMessages.THIS_IS_A_MESSAGE, {values: "I'm a value!"});
 * ```
 *
 * This transformer will only handle `SomeModule.messages.js`, and will output
 * something like:
 *
 * ```typescript
 * const {i18n} = require('@discord/intl');
 * const MESSAGE_KEYS = ["a9fn23"]
 * const loader = createLoader(MESSAGE_KEYS, {"en-US": () => require('./messages/en-US.messages.jsona')});
 * export default MESSAGE_KEYS.reduce((acc, k) => {acc[k] = loader.bindFor(k); return acc}, {});
 * ```
 *
 * Notice how the message keys have been hashed into short keys, and the
 * dynamic imports for each locale's data have been inserted automatically.
 *
 * The SWC and Babel transformers then take care of the second file,
 * transforming the usage to use the hashed keys:
 *
 * ```typescript
 * import someModuleMessages from 'SomeModule.messages.js';
 * i18n.format(someModuleMessages["a9fn23"], {values: "i'm a value!"});
 * ```
 */
class MessageDefinitionsTransformer {
  /**
   * @param {string[]} messageKeys The list of hashed message keys that this file manages.
   * @param {Record<string, string>} localeMap A map of locale names to paths used for importing translations.
   */
  constructor(messageKeys, localeMap) {
    this.messageKeys = messageKeys;
    this.localeMap = localeMap;
  }

  /**
   * Returns a compiled string for an object that maps locale names to a
   * dynamic require function for that locale, based on the supported locales
   * that were determined for this file. The shape ends up as:
   *
   * ```typescript
   * {
   *   "en-US": () => import("path/to/en-US.messages.json"),
   * }
   * ```
   *
   * @returns {string}
   */
  getLocaleRequireMap() {
    const localeProperties = [];
    for (const [locale, importPath] of Object.entries(this.localeMap)) {
      // leading `./`, which is necessary for the bundler to be able to
      // NOTE: This doesn't use `path.join` because it will remove any
      // resolve the import as a relative path.
      // This also assumes that the author has specified `translationsPath`
      // as a properly-resolvable path for the bundler, which we can't easily
      // enforce, unfortunately.
      localeProperties.push(`"${locale}": () => import("${importPath}")`);
    }

    return `{${localeProperties.join(',')}}`;
  }

  /**
   * Returns the reduced, transformed output for this file. Currently not
   * configurable, but could be told to include default messages or preserve
   * information as necessary.
   */
  getOutput() {
    const messageKeys = this.messageKeys;

    const i18nImport = `const {createLoader} = require('@discord/intl');`;
    const keysDefinition = `const _keys = ${JSON.stringify(messageKeys)};`;
    const localeMapDefinition = `const _locales = ${this.getLocaleRequireMap()};`;
    const loaderDefinition = 'const loader = createLoader(_keys, _locales);';
    const messagesExport = 'export default loader.getBinds();';

    return [i18nImport, keysDefinition, localeMapDefinition, loaderDefinition, messagesExport].join(
      '\n',
    );
  }
}

module.exports = {
  MessageDefinitionsTransformer,
};
