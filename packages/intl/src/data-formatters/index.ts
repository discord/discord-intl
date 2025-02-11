import {
  FormatConfigType,
  resolveFormatConfigOptions,
  TemporaryDurationFormatOptions,
  TemporaryDurationInput,
} from './config';
import { dataFormatterCache } from './cache';

export interface DataFormatters<FormatConfig extends FormatConfigType> {
  formatDate(
    value: number | Date,
    style?: Intl.DateTimeFormatOptions | { format?: keyof FormatConfig['date'] & string },
  ): string;
  formatDuration(
    value: TemporaryDurationInput,
    style?: TemporaryDurationFormatOptions | { format?: keyof FormatConfig['duration'] & string },
  ): string;
  formatNumber(
    value: number,
    style?: Intl.NumberFormatOptions | { format?: keyof FormatConfig['number'] & string },
  ): string;
  formatList(
    values: any[],
    style?: Intl.ListFormatOptions | { format?: keyof FormatConfig['list'] & string },
  ): string;
  formatListToParts<T>(
    values: T[],
    style?: Intl.ListFormatOptions | { format?: keyof FormatConfig['list'] & string },
  ): Array<T | string>;
  formatRelativeTime(
    value: number,
    unit: Intl.RelativeTimeFormatUnit,
    style?:
      | Intl.RelativeTimeFormatOptions
      | { format?: keyof FormatConfig['relativeTime'] & string },
  ): string;
  formatTime(
    value: number | Date,
    style?: Intl.DateTimeFormatOptions | { format?: keyof FormatConfig['time'] & string },
  ): string;
  getPluralRules(options: Intl.PluralRulesOptions): Intl.PluralRules;
}

export function makeDataFormatters<const FormatConfig extends FormatConfigType>(
  locales: string[],
  formatConfig: FormatConfig,
): DataFormatters<FormatConfig> {
  return {
    formatDate(value, style) {
      const options = resolveFormatConfigOptions(formatConfig.date, style);
      return dataFormatterCache.getDateTimeFormatter(locales, options).format(value);
    },

    formatDuration(value, style) {
      const options = resolveFormatConfigOptions(formatConfig.time, style);
      return dataFormatterCache.getDurationFormatter(locales, options).format(value);
    },

    formatNumber(value, style) {
      const options = resolveFormatConfigOptions(formatConfig.number, style);
      return dataFormatterCache.getNumberFormatter(locales, options).format(value);
    },

    formatList(values, style) {
      const options = resolveFormatConfigOptions(formatConfig.list, style);
      return dataFormatterCache.getListFormatter(locales, options).format(values);
    },

    formatListToParts(values, style) {
      const options = resolveFormatConfigOptions(formatConfig.list, style);
      // Intl.ListFormat only accepts string arguments, even for `formatToParts`,
      // but we want to support formatting of complex values like React nodes as
      // part of a list (think an "and more" link at the end of a list).
      //
      // To enable that, we make placeholders with hopefully-unusable sentinel
      // values that can be passed to `formatToParts`, then after the formatting
      // we map the original values back into the parts.
      const placeholders = {};
      for (const index in values) {
        placeholders['$+/-$placeholder.' + index] = values[index];
      }
      const parts = dataFormatterCache
        .getListFormatter(locales, options)
        .formatToParts(Object.keys(placeholders));

      return parts.map((part) => (part.value = placeholders[part.value] ?? part.value));
    },

    formatRelativeTime(value, unit, style) {
      const options = resolveFormatConfigOptions(formatConfig.relativeTime, style);
      return dataFormatterCache.getRelativeTimeFormatter(locales, options).format(value, unit);
    },

    formatTime(value, style) {
      const options = resolveFormatConfigOptions(formatConfig.time, style);
      return dataFormatterCache.getDateTimeFormatter(locales, options).format(value);
    },

    getPluralRules(options) {
      return dataFormatterCache.getPluralRules(locales, options);
    },
  };
}
