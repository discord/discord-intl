import { AstNode, FullFormatJsNode } from '@discord/intl-ast';
import { InternalIntlMessage } from './message';

/**
 * Type representing the serialized content of a translations file, which is a record of hashed
 * message keys to their AST structure. This type represents both compressed, keyless AstNodes and
 * fully-typed, object FullFormatJsNodes as the message content, since either can be given depending
 * on the configuration of the bundler/compiler.
 */
type MessagesData = Record<string, AstNode[] | FullFormatJsNode[]>;

export type IntlMessageGetter = (this: MessageLoader, locale: LocaleId) => InternalIntlMessage;

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
  _localeLoadingPromises: Record<
    LocaleId,
    {
      /**`true` if this locale has been loaded at least once. */
      initialized: boolean;
      /**
       * If the locale is currently being loaded, this Promise will exist and
       * represent that loading status. Otherwise, this value is undefined.
       */
      current?: Promise<{ default: MessagesData }> | undefined;
    }
  >;

  /**
   * Map of pre-parsed messages, keyed by the message key and locale
   */
  _parseCache: Record<LocaleId, { [name: string]: InternalIntlMessage }>;

  /**
   * List of subscribers listening for changes to the current locale and
   * when it has loaded.
   */
  _subscribers: Set<() => void>;

  /**
   * If a message is not present in this loader, try looking it up in this
   * loader instead before resorting to a default placeholder message.
   *
   * The order of lookups will be:
   * - this loader, requested locale
   * - this loader, default locale
   * - fallback loader, requested locale
   * - fallback loader, default locale
   * - fallback message
   */
  fallbackLoader?: MessageLoader;
  /**
   * Message to show as a fallback when the requested message is unavailable
   * from both this loader and the fallback loader.
   */
  fallbackMessage: InternalIntlMessage;

  /**
   * When `fallbackLoader` is set, this value will be set on the fallback
   * loader to reference this loader. Setting this ensures that loaders won't
   * ever form an infinite circular chain.
   */
  _parentLoader?: MessageLoader;

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

  constructor(localeImportMap: LocaleImportMap, defaultLocale: LocaleId) {
    this.messages = {};
    this.localeImportMap = localeImportMap;
    this.supportedLocales = Object.keys(localeImportMap);
    this.defaultLocale = defaultLocale;

    this._localeLoadingPromises = {};
    this._parseCache = {};
    this._subscribers = new Set();

    // TODO: Bring this back as an option to pre-load the locale.
    // this._loadLocale(this.defaultLocale);
    this.fallbackMessage = new InternalIntlMessage([], this.defaultLocale);

    // In cases where hot module replacement is available, set up cache clearing whenever the
    // targets change so that values are always replaced.
    // @ts-expect-error `hot` not defined in types.
    if (module.hot) {
      for (const [locale, file] of Object.entries(localeImportMap)) {
        // @ts-expect-error `hot` not defined in types.
        module.hot.accept(file, async () => {
          await this._loadLocale(locale);
          this._parseCache = {};
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

  /**
   * Configure a fallback loader to search for any message requests that are not resolved by this
   * loader directly. For example, if a request for a message "abcdef" in locale `pt-BR` is not
   * found in this loader's managed content, the request gets forwarded on to this fallback instead.
   *
   * The lookup first exhausts this loader entirely before checking the fallback, in the order:
   * - This loader's requested locale.
   * - This loader's default locale.
   * - Fallback loader's requested locale.
   * - Fallback loader's default locale.
   * - Fallback loader's final fallback value.
   * - This loader's final fallback value.
   *
   * Note that the final fallback value (the `fallbackMessage` property on this class) will first
   * be retrieved from the fallback loader rather than this instance.
   */
  fallbackWith(fallback: MessageLoader) {
    let parent: MessageLoader = this;
    while (parent != null) {
      parent = parent._parentLoader;
      if (parent === this) {
        throw new Error(
          'Setting `fallbackWith` on MessageLoader created a circular chain that would never resolve',
        );
      }
    }
    this.fallbackLoader = fallback;
    fallback._parentLoader = this;
  }

  get(key: string, locale: LocaleId): InternalIntlMessage {
    const expectedValue = this.getMessageValue(key, locale);
    if (expectedValue != null) return expectedValue;
    // If the expected locale is still in progress with loading and the default locale _has not_
    // started loading already, then return the fallback message instead of starting a load of the
    // default locale when it may not be necessary.
    if (this.isLocaleLoading(locale) && !this.isLocaleLoaded(this.defaultLocale)) {
      return this.fallbackMessage;
    }

    const defaultLocaleValue = this.getMessageValue(key, this.defaultLocale);
    if (defaultLocaleValue != null) return defaultLocaleValue;

    const fallbackLoaderValue = this.fallbackLoader?.get(key, locale);
    if (fallbackLoaderValue != null) {
      return fallbackLoaderValue;
    }

    // If the message couldn't be found in either the requested nor the default locale, then
    // nothing can be done.
    const errorKey = this._debugKeyMap != null ? `"${this._debugKeyMap[key]}" (${key})` : key;
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
    const parsed = this._parseCache[locale]?.[key];
    if (parsed) return parsed;

    // Check whether the locale exists in memory. If not, start a request to
    // load the locale if one has not already started. It's impossible for
    // the request to finish within the same tick, so return undefined in any
    // case then.
    if (this.messages[locale] == null) {
      if (this.supportedLocales.includes(locale)) {
        this._loadLocale(locale);
      }
      return undefined;
    }

    // Then try to return the loaded message. Previously, this used a `key in ...` check to quickly
    // verify whether the locale object contains the requested message, but in the case where the
    // target is a Proxy or uses some other internalized method for providing gettable values, `in`
    // queries aren't guaranteed to work. Instead, since this is realistically amortized to just be
    // a property access on every call, we can just the returned value as the determining factor. If
    // the locale object returns `undefined`, then it "does not contain" the message, whether it's
    // still loading or otherwise. Future calls will then be able to pick up any changed value,
    // since it won't be cached in the `_parseCache`.
    const content = this.messages[locale][key];
    if (content != null) {
      const message = new InternalIntlMessage(content, locale);
      (this._parseCache[locale] ??= {})[key] = message;
      return message;
    }

    // Otherwise just assume it doesn't exist.
    return undefined;
  }

  async _loadLocale(locale: LocaleId) {
    // If the locale is already set in `messages`, then it doesn't need to be loaded again.
    if (this.messages[locale] != null) return;

    // Don't re-load a locale that's already in progress.
    if (this._localeLoadingPromises[locale]?.current != null) {
      await this._localeLoadingPromises[locale]?.current;
      return;
    }

    // Safety check in case the locale map doesn't include a require for the
    // requested locale. Shouldn't happen, but throwing here is much more
    // contextual than whatever error would result otherwise.
    if (this.localeImportMap[locale] == null) {
      if (this.supportedLocales.includes(locale)) {
        // `supportedLocales` is determined by the `localeImportMap`, so if it's present but
        // nullish, it's almost definitely a configuration error and deserves to be reported loudly.
        throw new Error(
          `Requested to load locale ${locale}, which should be supported, but no source for translation data was provided.`,
        );
      } else {
        return;
      }
    }

    const current = this.localeImportMap[locale]();
    const isAlreadyInitialized = this._localeLoadingPromises[locale]?.initialized ?? false;
    this._localeLoadingPromises[locale] = { initialized: isAlreadyInitialized, current };
    this.messages[locale] = (await current).default;
    this._localeLoadingPromises[locale] = { initialized: true, current: undefined };
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

  isLocaleLoading(locale: LocaleId): boolean {
    return this._localeLoadingPromises[locale]?.current != null;
  }

  isLocaleLoaded(locale: LocaleId, requireCurrent: boolean = false): boolean {
    const ref = this._localeLoadingPromises[locale];
    // Ensure the locale at least exists and is initialized.
    if (ref == null || ref.initialized == false) return false;
    // `current` will be deleted once the locale has loaded, so if it is null,
    // then the locale data is current.
    if (requireCurrent) return ref.current == null;
    // Otherwise, initialization is enough.
    return true;
  }

  async waitForLocaleLoaded(locale: LocaleId, requireCurrent = false): Promise<void> {
    const ref = this._localeLoadingPromises[locale];
    // If the locale hasn't been loaded at all, kick off the load and return
    // that Promise directly.
    if (ref == null) return this._loadLocale(locale);
    // If initialization is enough for this request, just resolve immediately.
    if (ref.initialized && !requireCurrent) return;
    // Otherwise, check the `current` loading state on the object, and if that
    // doesn't exist, then the locale data is already finished loading, and the
    // containing Promise will just resolve with `undefined`.
    await ref.current;
  }

  /**
   * Returns true if this loader's default locale has finished loading. When
   * true, all messages managed by this loader are guaranteed to have _a_
   * render-able value, even if one does not exist in the current locale.
   */
  async waitForDefaultLocale(requireCurrent = false): Promise<void> {
    return this.waitForLocaleLoaded(this.defaultLocale, requireCurrent);
  }
}

const LOADER_REGISTRY: MessageLoader[] = [];

/**
 * Kick off a load request for the given locale across all currently-registered
 * message loaders, returning a Promise that resolves when all loaders have
 * finished loading that locale.
 */
export async function loadAllMessagesInLocale(locale: LocaleId): Promise<void> {
  await Promise.all(LOADER_REGISTRY.map((loader) => loader._loadLocale(locale)));
}

/**
 * Returns a new Promise that resolves after all currently-registered message
 * loaders have finished successfully loading their default locale content.
 * Once this Promise has resolved, all messages that currently exist in the
 * application (i.e., within all modules that have been imported or required in
 * the current session) can be guaranteed to have _a_ render-able value.
 *
 * This function can be called at any point in an applications life cycle and
 * will always ensure that _all_ loaders that currently exist are initialized.
 */
export async function waitForAllDefaultIntlMessagesLoaded(): Promise<void> {
  await Promise.all(LOADER_REGISTRY.map((loader) => loader.waitForDefaultLocale()));
}

/**
 * Create a new MessageLoader, which handles lazily loading messages for different locales and
 * sanity checks as needed to provide accessors for each message contained by the import map.
 *
 * Notably, this does _not_ verify whether the locale objects managed by this loader actually
 * contain a given key. It is the responsibility of the caller to verify this (easily enforced with
 * typescript, eslint rules, and others).
 */
export function createLoader(localeImportMap: LocaleImportMap, defaultLocale: LocaleId) {
  const loader = new MessageLoader(localeImportMap, defaultLocale);
  LOADER_REGISTRY.push(loader);
  return loader;
}
