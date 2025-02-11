import { DEFAULT_FORMAT_CONFIG, type FormatConfigType } from './data-formatters/config';
import { bindFormatValues, FormatBuilderConstructor } from './format';
import type {
  FormatterImplementation,
  IntlMessageGetter,
  RequiredFormatValues,
  TypedIntlMessageGetter,
} from './types';
import { InternalIntlMessage } from './message';
import { DataFormatters, makeDataFormatters } from './data-formatters';

/**
 * Fallback locale used for all internationalization when an operation in the
 * requested locale is not possible.
 */
export const DEFAULT_LOCALE: string = 'en-US';

type FormatterReturnType<F extends FormatterImplementation<any, any>> = ReturnType<F['format']>;

type FormatterFunctionTypes<F extends FormatterImplementation<any, any>> =
  F extends FormatterImplementation<infer Functions, any> ? Functions : never;

export type FormatFunction<F extends FormatterImplementation<any, any>> = <
  T extends IntlMessageGetter,
>(
  this: IntlManager,
  message: T,
  values: RequiredFormatValues<T, FormatterFunctionTypes<F>>,
) => FormatterReturnType<F>;

type ThisWithFormatters<
  This,
  T extends Record<string, FormatterImplementation<any, any>>,
> = This & {
  [K in keyof T]: FormatFunction<T[K]>;
};

export interface IntlManagerOptions<FormatConfig> {
  /**
   * The locale to initially have this manager use. Useful to set when information about the user's
   * likely locale is available sooner than when that information is definitely known (at which
   * point `setLocale` should be called instead).
   *
   * @default DEFAULT_LOCALE
   */
  initialLocale?: string;
  /**
   * The locale in which most messages are _defined_, used as an indication of where fallbacks
   * should be loaded from when they can't be found in the requested `currentLocale`.
   *
   * @default DEFAULT_LOCALE
   */
  defaultLocale?: string;
  /**
   * Configuration for the different kinds of data dataFormatters that can be used both inside of
   * messages and in their own functions (like `intl.formatDate`) as shorthands for common sets of
   * options. For example, with the default config, this enables `intl.formatDate(now, 'short')`
   * rather than having to specify the exact properties of each style every time.
   *
   * @default DEFAULT_FORMATTER_CONFIG
   */
  formatConfig?: FormatConfig;
}

export class IntlManager<
  const FormatConfig extends FormatConfigType = typeof DEFAULT_FORMAT_CONFIG,
> {
  defaultLocale: string;
  currentLocale: string;
  formatConfig: FormatConfig;

  data: DataFormatters<FormatConfig>;

  _localeSubscriptions: Set<(locale: string) => void>;

  constructor({
    initialLocale = DEFAULT_LOCALE,
    defaultLocale = DEFAULT_LOCALE,
    // @ts-expect-error If it's not given, formatConfig defaults to the same
    // value as the type parameters, but typescript things this should be
    // overwritten
    formatConfig = DEFAULT_FORMAT_CONFIG,
  }: IntlManagerOptions<FormatConfig>) {
    this.currentLocale = initialLocale;
    this.defaultLocale = defaultLocale;
    this.formatConfig = formatConfig;
    this.data = makeDataFormatters([this.currentLocale, this.defaultLocale], this.formatConfig);

    this._localeSubscriptions = new Set();
  }

  /**
   * Add a set of formatter implementations to this manager, making each available as a direct
   * property
   */
  withFormatters<const T extends Record<string, FormatterImplementation<any, any>>>(
    dataFormatters: T,
  ): ThisWithFormatters<this, T> {
    for (const [name, formatter] of Object.entries(dataFormatters)) {
      this[name] = this.makeFormatFunction(formatter);
    }

    return this as ThisWithFormatters<this, T>;
  }

  /**
   * Return a new function bound to this manager that uses the given `FormatterImplementation` to
   * format a message after it has been resolved for the current locale and potentially processed in
   * other ways by the manager.
   */
  makeFormatFunction<F extends FormatterImplementation<any, any>>({
    format,
    builder,
  }: F): FormatFunction<FormatterReturnType<F>> {
    const formatter = format.bind(this);
    return (message, values) => {
      if (message == null) return null;
      return formatter(message(this.currentLocale), values, builder);
    };
  }

  /**
   * Set the locale for this package to use. This is a global change, and all
   * methods on this object will use this locale for formatting.
   */
  setLocale(locale: string) {
    this.currentLocale = locale;
    this.data = makeDataFormatters([this.currentLocale, this.defaultLocale], this.formatConfig);
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
   * For static messages with no rich text and no dynamic placeholders, use this method to
   * immediately return the plain string value of the message in the current locale.
   */
  string<T extends TypedIntlMessageGetter<{}>>(message: T): string {
    if (message == null) return '';
    return message(this.currentLocale).reserialize();
  }

  /**
   * Return a raw string representing the syntax of the original message, as
   * authored, with no values replaced. The result of this function could be
   * written back to the definition file for the message and re-parsed to
   * create an identical message to the original.
   *
   * This should rarely be necessary outside of sending raw messages to other
   * applications that do their own message parsing.
   */
  reserialize<T extends IntlMessageGetter>(message: T): string {
    if (message == null) return '';

    const resolved = message(this.currentLocale);
    if (typeof resolved === 'string') return resolved;
    return resolved.reserialize();
  }

  bindFormatValues<T>(
    Builder: FormatBuilderConstructor<T>,
    message: InternalIntlMessage,
    values: Record<string, any>,
  ): T[] {
    return bindFormatValues(
      Builder,
      message.ast,
      [this.currentLocale, this.defaultLocale],
      this.data,
      this.formatConfig,
      values,
    );
  }
}
