import {
  FormatConfigType,
  resolveFormatConfigOptions,
  TemporaryDurationFormatOptions,
} from './config';
import { dataFormatterCache } from './cache';

export interface DataFormatters<FormatConfig extends FormatConfigType> {
  formatDate(
    value: number | Date,
    style?: (keyof FormatConfig['date'] & string) | Intl.DateTimeFormatOptions,
  ): string;

  formatDuration(
    value: number,
    style?: (keyof FormatConfig['duration'] & string) | TemporaryDurationFormatOptions,
  ): string;

  formatNumber(
    value: number,
    style?: (keyof FormatConfig['number'] & string) | Intl.DateTimeFormatOptions,
  ): string;

  formatList(
    values: any[],
    style?: (keyof FormatConfig['list'] & string) | Intl.ListFormatOptions,
  ): string;
  formatListToParts<T>(
    values: T[],
    style?: (keyof FormatConfig['list'] & string) | Intl.ListFormatOptions,
  ): Array<T | string>;

  formatRelativeTime(
    value: number,
    unit: Intl.RelativeTimeFormatUnit,
    style?: (keyof FormatConfig['relativeTime'] & string) | Intl.RelativeTimeFormatOptions,
  ): string;
  formatTime(
    value: number | Date,
    style?: (keyof FormatConfig['time'] & string) | Intl.DateTimeFormatOptions,
  ): string;

  getPluralRules(options: Intl.PluralRulesOptions): Intl.PluralRules;
}

export function makeDataFormatters<const FormatConfig extends FormatConfigType>(
  locales: string[],
): DataFormatters<FormatConfig> {
  return {
    formatDate(
      value: number | Date,
      style?: (keyof FormatConfig['date'] & string) | Intl.DateTimeFormatOptions,
    ): string {
      const options = resolveFormatConfigOptions(this.formatConfig.date, style);
      return dataFormatterCache.getDateTimeFormatter(locales, options).format(value);
    },

    formatDuration(
      value: number,
      style?: (keyof FormatConfig['duration'] & string) | TemporaryDurationFormatOptions,
    ): string {
      const options = resolveFormatConfigOptions(this.formatConfig.time, style);
      return dataFormatterCache.getDurationFormatter(locales, options).format(value);
    },

    formatNumber(
      value: number,
      style?: (keyof FormatConfig['number'] & string) | Intl.DateTimeFormatOptions,
    ): string {
      const options = resolveFormatConfigOptions(this.formatConfig.number, style);
      return dataFormatterCache.getNumberFormatter(locales, options).format(value);
    },

    formatList(
      values: any[],
      style?: (keyof FormatConfig['list'] & string) | Intl.ListFormatOptions,
    ): string {
      const options = resolveFormatConfigOptions(this.formatConfig.list, style);
      return dataFormatterCache.getListFormatter(locales, options).format(values);
    },

    formatListToParts<T>(
      values: T[],
      style?: (keyof FormatConfig['list'] & string) | Intl.ListFormatOptions,
    ): Array<T | string> {
      const options = resolveFormatConfigOptions(this.formatConfig.list, style);
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

    formatRelativeTime(
      value: number,
      unit: Intl.RelativeTimeFormatUnit,
      style?: (keyof FormatConfig['relativeTime'] & string) | Intl.RelativeTimeFormatOptions,
    ): string {
      const options = resolveFormatConfigOptions(this.formatConfig.relativeTime, style);
      return dataFormatterCache.getRelativeTimeFormatter(locales, options).format(value, unit);
    },

    formatTime(
      value: number | Date,
      style?: (keyof FormatConfig['time'] & string) | Intl.DateTimeFormatOptions,
    ): string {
      const options = resolveFormatConfigOptions(this.formatConfig.time, style);
      return dataFormatterCache.getDateTimeFormatter(locales, options).format(value);
    },

    getPluralRules(options: Intl.PluralRulesOptions): Intl.PluralRules {
      return dataFormatterCache.getPluralRules(locales, options);
    },
  };
}
