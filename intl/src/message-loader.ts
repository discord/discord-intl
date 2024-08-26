import { InternalIntlMessage } from './message';

import type { MessageFormatElement } from '@formatjs/icu-messageformat-parser';

type AnyRawMessage = string | MessageFormatElement[];
type MessagesData = Record<string, AnyRawMessage>;
export interface IntlMessageGetterAdditions {
  onChange(callback: () => void): () => void;
}

export interface IntlMessageGetter extends IntlMessageGetterAdditions {
  (locale: LocaleId): InternalIntlMessage;
}

export type LocaleId = string;

export type LocaleImportMap = Record<LocaleId, () => Promise<{ default: MessagesData }>>;

export class MessageLoader {
  messageKeys: string[];
  /** Map of locale to string key to message content. */
  messages: Record<LocaleId, MessagesData>;
  localeImportMap: LocaleImportMap;
  supportedLocales: LocaleId[];
  /**
   * Fallback locale where messages will try to be looked up if they don't exist in the requested
   * locale in a given call. This locale will also be immediately loaded when this loader is
   * constructed to ensure the contents are available as soon as possible.
   */
  defaultLocale: LocaleId;

  /**
   * Promise representing the current locale being loaded. If this is non-null,
   * then a request has already been made to load the locale and it is waiting
   * to resolve.
   */
  _localeLoadingPromises: Record<LocaleId, Promise<{ default: MessagesData }> | undefined>;

  /**
   * Map of pre-parsed messages, keyed by the message key and locale
   */
  _parseCache: Map<string, InternalIntlMessage>;

  /**
   * List of subscribers listening for changes to the current locale and
   * when it has loaded.
   */
  _subscribers: Set<() => void>;

  /**
   * Message to show as a fallback when the requested message is unavailable.
   */
  fallbackMessage: InternalIntlMessage;

  ///
  // Debug mode values
  ///

  /**
   * Map from hashed message keys to their original values, to provide context in error messages
   * that would otherwise be obfuscated.
   *
   * Consumers should only provide this value in developer or debug environments.
   *
   * @private
   */
  _debugKeyMap?: Record<string, string>;
  /**
   * Map of locale names to source file names, used to provide context for where messages are
   * imported from during development.
   *
   * Consumers should only provide this value in developer or debug environments.
   *
   * @private
   */
  _localeFileMap?: Record<string, string>;

  constructor(messageKeys: string[], localeImportMap: LocaleImportMap, defaultLocale: LocaleId) {
    this.messageKeys = messageKeys;
    this.messages = {};
    this.localeImportMap = localeImportMap;
    this.supportedLocales = Object.keys(localeImportMap);
    this.defaultLocale = defaultLocale;

    this._localeLoadingPromises = {};
    this._parseCache = new Map();
    this._subscribers = new Set();

    this._loadLocale(this.defaultLocale);
    this.fallbackMessage = new InternalIntlMessage('THIS MESSAGE FAILED TO LOAD', 'en-US');

    // In cases where hot module replacement is available, set up cache clearing whenever the
    // targets change so that values are always replaced.
    // @ts-expect-error `hot` not defined in types.
    if (module.hot) {
      for (const [locale, file] of Object.entries(localeImportMap)) {
        // @ts-expect-error `hot` not defined in types.
        module.hot.accept(file, () => {
          this._parseCache.clear();
          this._loadLocale(locale);
        });
      }
    }
  }

  /**
   * Provide additional debug information to use during development, providing additional context
   * for console error messages.
   *
   * @param {Record<string, string>} keyMap
   * @param {Record<string, string>} localeFileMap
   */
  withDebugValues(keyMap: Record<string, string>, localeFileMap: Record<string, string>) {
    this._debugKeyMap = keyMap;
    this._localeFileMap = localeFileMap;
  }

  get(key: string, locale: LocaleId): InternalIntlMessage {
    const value =
      this.getMessageValue(key, locale) ?? this.getMessageValue(key, this.defaultLocale);
    if (value != null) {
      return value;
    }

    // If the message couldn't be found in either the requested nor the default locale, then
    // nothing can be done.
    const errorKey = this._debugKeyMap != null ? `"${this._debugKeyMap[key]}" (${key})` : undefined;
    const requestedLocale =
      this._localeFileMap != null ? `${locale} (${this._localeFileMap[locale]})` : locale;
    const defaultLocale =
      this._localeFileMap != null
        ? `${this.defaultLocale} (${this._localeFileMap[this.defaultLocale]})`
        : this.defaultLocale;
    console.warn(
      `Requested message ${errorKey} does not have a value in the requested locale ${requestedLocale} nor the default locale ${defaultLocale}`,
    );
    return this.fallbackMessage;
  }

  /**
   * Return the value of the message with the given `key` in the given `locale`. If the message has
   * no value in that locale, this function returns `undefined`.
   *
   * This function will first check the cache to see if it has already been loaded, and will later
   * set the cache after parsing if not. If the requested locale has not yet been loaded, this
   * function will trigger a load for that locale, but will not wait for it to resolve before
   * returning.
   */
  getMessageValue(key: string, locale: LocaleId): InternalIntlMessage | undefined {
    const cacheKey = key + '@' + locale;
    const cachedValue = this._parseCache.get(cacheKey);
    if (cachedValue != null) return cachedValue;

    // Return early if this locale is still in the process of being loaded.
    if (this._localeLoadingPromises[locale] != null) {
      return undefined;
    }

    // If not, check whether it's been loaded and trigger a load if not.
    if (this.messages[locale] == null) {
      // Ensure the locale is loaded, if it is supported
      if (this.supportedLocales.includes(locale)) {
        this._loadLocale(locale);
      }

      return undefined;
    }

    // Then try to return the loaded message.
    if (key in this.messages[locale]) {
      const content = this.messages[locale][key];
      const message = new InternalIntlMessage(content, locale);
      this._parseCache.set(cacheKey, message);
      return message;
    }

    // Otherwise just assume it doesn't exist.
    return undefined;
  }

  /**
   * Returns a record mapping the keys this object manages to bound functions
   * for `get` with the that key as the first argument, allowing consumers to
   * just call the function with a locale to retrieve the translated message
   * for that key.
   */
  getBinds(): Record<string, IntlMessageGetter> {
    const onChange = this.onChange.bind(this);
    return Object.keys(this.messageKeys).reduce(
      (acc, key) => {
        const bound = this.get.bind(this, key) as IntlMessageGetter;
        bound.onChange = onChange;
        acc[key] = bound;
        return acc;
      },
      {} as Record<string, IntlMessageGetter>,
    );
  }

  async _loadLocale(locale: LocaleId) {
    // Don't re-load a locale that's already in progress.
    if (this._localeLoadingPromises[locale] != null) {
      return;
    }

    // Safety check in case the locale map doesn't include a require for the
    // requested locale. Shouldn't happen, but throwing here is much more
    // contextual than whatever error would result otherwise.
    if (this.localeImportMap[locale] == null) {
      throw new Error(
        `Requested to load locale ${locale}, which should be supported, but no source for translation data was provided.`,
      );
    }

    // If the locale is already set in `messages`, then it doesn't need to be loaded again.
    if (this.messages[locale] != null) return;

    const loadingPromise = this.localeImportMap[locale]();
    this._localeLoadingPromises[locale] = loadingPromise;
    this.messages[locale] = (await loadingPromise).default;
    delete this._localeLoadingPromises[locale];
    this.emitChange();
  }

  /**
   * Inform subscribers that the loader state has changed and they should
   * potentially update to get new values for messages.
   */
  emitChange() {
    for (const callback of this._subscribers.values()) {
      callback();
    }
  }

  /**
   * Subscribe to events about when a locale has loaded.
   */
  onChange(callback: () => void): () => void {
    this._subscribers.add(callback);

    return () => this._subscribers.delete(callback);
  }
}
