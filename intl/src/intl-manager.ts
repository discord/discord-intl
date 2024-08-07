import { IntlShape, createIntl } from '@formatjs/intl';
import {
  PART_TYPE as FormatPartType,
  Formats,
  Formatters,
  IntlMessageFormat,
} from 'intl-messageformat';

import { LocaleImportMap, MessageLoader } from './message-loader';

import type { TypedIntlMessageGetter, FormatValuesFor, RichTextElementMap } from './types';

/**
 * Fallback locale used for all internationalization when an operation in the
 * requested locale is not possible.
 */
export const DEFAULT_LOCALE: string = 'en-US';

export class IntlManager<
  DefaultElements extends RichTextElementMap,
  DefaultValues extends keyof DefaultElements,
> {
  defaultLocale: string;
  currentLocale: string;
  intl: IntlShape;

  /**
   * When formatting tag elements like `<b>value</b>`, these functions will
   * automatically be used by all messages, unless they are explicitly removed
   * for a specific call, either by overriding the value or by calling a
   * dedicated method like `formatToPlainString` which will remove all
   * rich elements from the string and return only the plain text result.
   */
  defaultRichTextElements: DefaultElements;

  _localeSubscriptions: Set<(locale: string) => void>;

  constructor(defaultLocale: string = DEFAULT_LOCALE, defaultRichTextElements: DefaultElements) {
    this.defaultLocale = defaultLocale;
    this.currentLocale = defaultLocale;
    this.intl = createIntl({
      formats: IntlMessageFormat.formats,
      defaultLocale,
      locale: defaultLocale,
    });

    this.defaultRichTextElements = defaultRichTextElements;
    this._localeSubscriptions = new Set();
  }

  /**
   * Set the locale for this package to use. This is a global change, and all
   * methods on this object will use this locale for formatting.
   */
  setLocale(locale: string) {
    this.currentLocale = locale;
    this.intl = createIntl({ defaultLocale: this.defaultLocale, locale });
    this.emitLocaleChange(locale);
  }

  emitLocaleChange(locale: string) {
    for (const callback of this._localeSubscriptions) {
      callback(locale);
    }
  }

  /**
   * Subscribe to changes in the current locale for this manager.
   */
  onLocaleChange = (callback: (locale: string) => void) => {
    this._localeSubscriptions.add(callback);
    return () => this._localeSubscriptions.delete(callback);
  };

  /**
   * Format the given message in the current locale with the provided values.
   * The returned values is _always_ an Array of parts, even if the message is
   * a simple string value.
   *
   * This function is the basis of how messages are normally formatted, and can
   * be used anywhere. However, it is not reactive and only functions on the
   * data that is currently loaded and known. For a reactive function that
   * automatically updates when the locale changes or when new data is loaded,
   * use `format`, which will wrap the formatting in a React component that
   * subscribes to the current locale and state of loaded messages.
   */
  formatToParts<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
  ): Array<string | any>;
  formatToParts<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values: Omit<FormatValuesFor<T>, DefaultValues>,
  ): Array<string | any>;
  formatToParts<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values?: Omit<FormatValuesFor<T>, DefaultValues>,
  ): Array<string | any> {
    if (typeof message === 'string') return [message];
    const resolvedMessage = typeof message === 'function' ? message(this.currentLocale) : message;
    if (typeof resolvedMessage === 'string') return [resolvedMessage];

    const resolvedValues =
      values != null
        ? { ...this.defaultRichTextElements, ...values }
        : this.defaultRichTextElements;
    const parts = resolvedMessage.formatToParts(
      this.intl.formatters as Formatters,
      this.intl.formats as Formats,
      resolvedValues,
    );

    const result = [];
    let inLiteral = false;
    for (const part of parts) {
      // This condition merges consecutive literal elements (static strings)
      // into single parts to reduce the number of nodes in the result. This
      // condition will never be true on the first loop, ensuring that the
      // array must at least have one entry before it attempts to concatenate
      // onto that value again.
      if (inLiteral && (inLiteral = part.type === FormatPartType.literal)) {
        result[result.length - 1] += part.value;
        continue;
      }

      inLiteral = part.type === FormatPartType.literal;
      result.push(part.value);
    }

    return result;
  }

  /**
   * Format the given message with the provided values, removing any styling
   * and non-textual content from the message, returning a plain string.
   */
  formatToPlainString<T extends TypedIntlMessageGetter<object | undefined>>(message: T): string;
  formatToPlainString<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values: Omit<FormatValuesFor<T>, DefaultValues>,
  ): string;
  formatToPlainString<T extends TypedIntlMessageGetter<object | undefined>>(
    message: T,
    values?: Omit<FormatValuesFor<T>, DefaultValues>,
  ) {
    if (typeof message === 'string') return message;
    const resolvedMessage = message(this.currentLocale);
    if (typeof resolvedMessage === 'string') return resolvedMessage;

    // No need to pass in `defaultRichTextElements`, since the stylistic tags
    // will be removed from the string anyway.
    return resolvedMessage.formatToPlainString(
      this.intl.formatters as Formatters,
      this.intl.formats as Formats,
      values,
    );
  }
}

/**
 * Create a new MessageLoader, which handles lazily loading messages for
 * different locales and sanity checks as needed to provide accessors for each
 * message defined in `messageKeys`.
 */
export function createLoader(messageKeys: string[], localeImportMap: LocaleImportMap) {
  return new MessageLoader(messageKeys, localeImportMap);
}
