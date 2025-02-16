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
 * intl.format(someModuleMessages.THIS_IS_A_MESSAGE, {values: "I'm a value!"});
 * ```
 *
 * This transformer will only handle `SomeModule.messages.js`, and will output
 * something like:
 *
 * ```typescript
 * const {createLoader} = require('@discord/intl');
 * const _localeMap = {"en-US": () => require('./messages/en-US.messages.json')};
 * export const messagesLoader = createLoader(_localeMap);
 * export default {
 *   a9fn23(locale) => messagesLoader.get("a9fn23", locale)
 * };
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
 * intl.format(someModuleMessages["a9fn23"], {values: "i'm a value!"});
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
      // This assumes that the author has specified `importPath` as a properly-resolvable path for
      // the bundler, which we can't easily enforce, unfortunately.
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
   * When `option.proxyBinds` is set to true, this method is invoked to create it.
   *
   * The binds proxy is a plain `Proxy` object with configuration applied to make it act and
   * function like a complete object, but without having to instantiate potentially thousands of
   * binds during initialization. The proxy intentionally does _not_ support iteration nor `key in`
   * queries, as they require up-front initialization that is too costly when multiple thousands
   * of message keys are included.
   *
   * However, the proxy _does_ implement `ownKeys`, such that the returned list of keys represents
   * all of the messages that have been _accessed_ through this proxy so far. This is generally
   * more of a debugging utility than anything else, but can be useful for diagnosing when messages
   * are used in critical paths or otherwise.
   *
   * @returns {string[]}
   */
  createBindsProxy() {
    return [
      `const {makeMessagesProxy} = require('@discord/intl');`,
      `const binds = makeMessagesProxy(${this.loaderName});`,
    ];
  }

  /**
   * Return a map of key names to bound message getter functions. If `proxyBinds` is
   * configured to be `true`, the binds will be created as a constant object in the output.
   * Otherwise, the generation will be done at runtime through the `getBinds` method on the loader.
   *
   * @returns {string[]}
   */
  createLoaderAndBinds() {
    switch (this.options.bindMode ?? 'proxy') {
      case 'proxy':
        return [
          `const ${this.loaderName} = createLoader(_localeMap, _defaultLocale);`,
          ...this.createBindsProxy(),
        ];
      case 'literal': {
        const bindLines = Object.keys(this.options.messageKeys).map(
          (bind) => `"${bind}"(locale) { return ${this.loaderName}.get("${bind}", locale) }`,
        );
        return [
          `const binds = {${bindLines.join(',')}};`,
          `const ${this.loaderName} = createLoader(_localeMap, _defaultLocale);`,
        ];
      }
      default:
        throw new Error(
          `Unknown value for intl transformer option 'bindMode': ${this.options.bindMode}`,
        );
    }
  }

  /**
   * Return the lines to export fields from this module, as determined by the `exportMode` on this
   * transformer.
   *
   * @returns {string[]}
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
      `const _localeMap = ${this.getLocaleRequireMap()};`,
      `const _defaultLocale = ${JSON.stringify(this.options.defaultLocale)};`,
      ...this.createLoaderAndBinds(),
      ...this.debugModeSetup(),
      ...this.exportFields(),
    ].join('\n');
  }
}

module.exports = {
  MessageDefinitionsTransformer,
};
