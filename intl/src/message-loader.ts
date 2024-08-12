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

type LocaleId = string;

export type LocaleImportMap = Record<LocaleId, () => Promise<{ default: MessagesData }>>;

export class MessageLoader {
  messageKeys: string[];
  /** Map of locale to string key to message content. */
  messages: Record<LocaleId, MessagesData>;
  localeImportMap: LocaleImportMap;
  supportedLocales: LocaleId[];

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

  fallbackMessage: InternalIntlMessage;

  constructor(messageKeys: string[], localeImportMap: LocaleImportMap) {
    this.messageKeys = messageKeys;
    this.messages = {};
    this.localeImportMap = localeImportMap;
    this.supportedLocales = Object.keys(localeImportMap);

    this._localeLoadingPromises = {};
    this._parseCache = new Map();
    this._subscribers = new Set();

    this.fallbackMessage = new InternalIntlMessage('THIS MESSAGE FAILED TO LOAD', 'en-US');
  }

  get(key: string, locale: LocaleId): InternalIntlMessage {
    const cacheKey = key + '@' + locale;
    const cachedValue = this._parseCache.get(cacheKey);
    if (cachedValue != null) return cachedValue;

    if (this.messages[locale] == null) {
      // Ensure the locale is loaded, if it is supported
      if (this.supportedLocales.includes(locale)) {
        this._loadLocale(locale);
      }

      // Otherwise, fallback to just the key value in the meantime.
      return this.fallbackMessage;
    }

    // Then try to return the loaded message
    if (key in this.messages[locale]) {
      const content = this.messages[locale][key];
      const message = new InternalIntlMessage(content, locale);
      this._parseCache.set(cacheKey, message);
      return message;
    }

    // And finally throw if there's no match whatsoever
    throw new Error(
      `Requested message ${key} does not have a value in the requested locale nor the default locale`,
    );
  }

  /**
   * Returns a record mapping the keys this object manages to bound functions
   * for `get` with the that key as the first argument, allowing consumers to
   * just call the function with a locale to retrieve the translated message
   * for that key.
   */
  getBinds(): Record<string, IntlMessageGetter> {
    const onChange = this.onChange.bind(this);
    return this.messageKeys.reduce(
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

    const loadStart = performance.now();
    const loadingPromise = this.localeImportMap[locale]();
    this._localeLoadingPromises[locale] = loadingPromise;
    this.messages[locale] = (await loadingPromise).default;
    delete this._localeLoadingPromises[locale];
    const loadEnd = performance.now();
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
