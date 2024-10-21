/**
 * Common class for parsing and transforming the content of a messages
 * definition file (e.g., "SomeFeature.messages.js") into a production-ready
 * version, with message keys obfuscated, loading harnesses configured, and
 * more.
 *
 * This transformation is intended to be used alongside the _consumer_
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
 * const _keys = ["a9fn23"];
 * const _locales = {"en-US": () => require('./messages/en-US.messages.json')};
 * export const messagesLoader = createLoader(_keys, _locales);
 * export default messagesLoader.getBinds();
 * ```
 *
 * Notice how the message keys have been hashed into short keys, and the
 * dynamic imports for each locale's data have been inserted automatically. In
 * this example, the only locale is the source locale.
 *
 * The SWC and Babel transformers then take care of the second file,
 * transforming the usage to use the hashed keys:
 *
 * ```typescript
 * import someModuleMessages from 'SomeModule.messages.js';
 * i18n.format(someModuleMessages["a9fn23"], {values: "i'm a value!"});
 * ```
 *
 * The transformed file also contains a named export for `messagesLoader`,
 * which consumers can use to query and update the loading state for the
 * messages managed by that loader, including waiting for a locale to be
 * loaded, kicking off new loads, and more:
 *
 * ```typescript
 * import {messagesLoader} from 'SomeModule.messages.js';
 * // Wait for the loader to be initialized with default messages
 * await messagesLoader.waitForDefaultLocaleLoaded();
 * // Wait for a specific locale to load, starting the load if it
 * // is not yet in progress.
 * await messagesLoader.waitForLocaleLoaded('fr');
 * // In hot-reloading environments, use the second `requireCurrent`
 * // parameter to wait for the latest data, even if a value already
 * // exists.
 * const loaded = messagesLoader.isLocaleLoaded('fr', true);
 * ```
 */
class MessageDefinitionsTransformer {
  /**
   * @param {import('../types.d.ts').MessageDefinitionsTransformerOptions} options
   */
  constructor(options) {
    this.options = options;
    this.loaderName = 'messagesLoader';
  }

  /**
   * Returns a compiled string for an object that maps locale names to a
   * dynamic require function for that locale, based on the supported locales
   * that were determined for this file. The shape ends up as:
   *
   * ```typescript
   * {
   *   "en-US": () => import("path/to/en-US.json"),
   * }
   * ```
   *
   * @returns {string}
   */
  getLocaleRequireMap() {
    const localeProperties = [];
    for (const [locale, importPath] of Object.entries(this.options.localeMap)) {
      // This assumes that the author has specified `importPath`
      // as a properly-resolvable path for the bundler, which we can't easily
      // enforce, unfortunately.
      localeProperties.push(`"${locale}": () => ${this.options.getTranslationImport(importPath)}`);
    }

    return `{${localeProperties.join(',')}}`;
  }

  /**
   * Return a map of key hashes to their original values, as well as a plain-text map of locales
   * to the file names that they import from.
   *
   * @returns {string[]}
   */
  debugModeSetup() {
    if (!this.options.debug) return [];

    return [
      `${this.loaderName}.withDebugValues(${JSON.stringify(this.options.messageKeys)}, ${JSON.stringify(this.options.localeMap)})`,
    ];
  }

  /**
   * Return the lines to export fields from this module, as determined by the `exportMode` on this
   * transformer.
   */
  exportFields() {
    switch (this.options.exportMode ?? 'esm') {
      case 'esm':
        return [`export {${this.loaderName}};`, `export default binds;`];
      case 'commonjs':
        return [`module.exports = { messagesLoader: ${this.loaderName}, default: binds };`];
      case 'transpiledEsModule':
        return [
          `Object.defineProperty(exports, "__esModule", { value: true });`,
          `exports["messageLoader"] = ${this.loaderName};`,
          `exports["default"] = binds;`,
        ];
    }
  }

  /**
   * Returns the reduced, transformed output for this file. Currently not
   * configurable, but could be told to include default messages or preserve
   * information as necessary.
   *
   * @returns {string}
   */
  getOutput() {
    return [
      this.options.getPrelude?.() ?? '// No additional prelude was configured.',
      `const {createLoader} = require('@discord/intl');`,
      `const _keys = ${JSON.stringify(Object.keys(this.options.messageKeys))};`,
      `const _locales = ${this.getLocaleRequireMap()};`,
      `const _defaultLocale = ${JSON.stringify(this.options.defaultLocale)};`,
      `const ${this.loaderName} = createLoader(_keys, _locales, _defaultLocale);`,
      ...this.debugModeSetup(),
      `const binds = ${this.loaderName}.getBinds();`,
      ...this.exportFields(),
    ].join('\n');
  }
}

module.exports = {
  MessageDefinitionsTransformer,
};
